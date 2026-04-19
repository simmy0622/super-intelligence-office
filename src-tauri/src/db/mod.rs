use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result as AnyResult;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    agents::personas,
    models::{
        Actor, ActorSummary, AgentNote, AgentRun, FeedPost, FileInfo, FileSearchResult,
        Notification, PostMedia, PostMediaInput, PostReference, Salon, SalonMember, SelfEdits,
        SettingEntry, Task, TaskLog,
    },
};

#[derive(Clone)]
pub struct AppState {
    conn: Arc<Mutex<Connection>>,
    app_data_dir: Arc<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct PendingTrigger {
    pub id: i64,
    pub actor_id: i64,
    pub actor_handle: String,
    pub trigger: String,
    pub context_post_id: Option<i64>,
    pub salon_id: i64,
}

pub const GENERAL_SALON_ID: i64 = 1;
pub const MAX_SALONS_V1: usize = 3;

const MAX_THREAD_POSTS_FOR_AGENT_QUEUE: i64 = 10;
const MAX_AGENT_POSTS_PER_THREAD: i64 = 2;

impl AppState {
    pub fn new(app: &AppHandle) -> AnyResult<Self> {
        let app_data_dir = app.path().app_data_dir()?;

        fs::create_dir_all(&app_data_dir)?;
        let db_path = app_data_dir.join("agent-salon.sqlite3");
        let conn = Connection::open(db_path)?;

        let state = Self {
            conn: Arc::new(Mutex::new(conn)),
            app_data_dir: Arc::new(app_data_dir),
        };

        state.initialize()?;
        state
            .recover_interrupted_runtime_state()
            .map_err(anyhow::Error::msg)?;

        Ok(state)
    }

    pub fn get_thread_root_id(&self, post_id: i64) -> std::result::Result<i64, String> {
        self.with_conn(|conn| thread_root_id(conn, post_id))
    }

    pub fn count_thread_posts(&self, root_id: i64) -> std::result::Result<i64, String> {
        self.with_conn(|conn| count_thread_posts(conn, root_id))
    }

    pub fn count_actor_posts_in_thread(&self, actor_id: i64, root_id: i64) -> std::result::Result<i64, String> {
        self.with_conn(|conn| count_actor_posts_in_thread(conn, actor_id, root_id))
    }

    pub fn thread_response_limit_reached(
        &self,
        actor_id: i64,
        post_id: i64,
    ) -> std::result::Result<bool, String> {
        self.with_conn(|conn| thread_response_limit_reached(conn, actor_id, post_id))
    }

    pub fn is_post_by_agent(&self, post_id: i64) -> std::result::Result<bool, String> {
        self.with_conn(|conn| is_post_by_agent(conn, post_id))
    }

    pub fn app_data_dir(&self) -> PathBuf {
        self.app_data_dir.as_ref().clone()
    }

    pub fn count_replies(&self, post_id: i64) -> std::result::Result<i64, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM posts WHERE parent_id = ?1",
                params![post_id],
                |row| row.get(0),
            )
            .map_err(to_string)
        })
    }

    pub fn list_actor_posts_since(
        &self,
        actor_id: i64,
        since_ts: i64,
        limit: i64,
    ) -> std::result::Result<Vec<FeedPost>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT id FROM posts 
                     WHERE actor_id = ?1 AND created_at >= ?2 AND parent_id IS NULL
                     ORDER BY created_at DESC LIMIT ?3",
                )
                .map_err(to_string)?;
            let rows = statement
                .query_map(params![actor_id, since_ts, limit], |row| row.get::<_, i64>(0))
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            rows.into_iter()
                .map(|post_id| self.fetch_post(conn, post_id))
                .collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    pub fn get_latest_standup(
        &self,
        salon_id: i64,
        since_ts: i64,
    ) -> std::result::Result<Option<FeedPost>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT id FROM posts 
                     WHERE salon_id = ?1 AND trigger = 'standup' AND created_at >= ?2
                     ORDER BY created_at DESC LIMIT 1",
                )
                .map_err(to_string)?;
            let mut rows = statement.query(params![salon_id, since_ts]).map_err(to_string)?;
            if let Some(row) = rows.next().map_err(to_string)? {
                let id: i64 = row.get(0).map_err(to_string)?;
                Ok(Some(self.fetch_post(conn, id)?))
            } else {
                Ok(None)
            }
        })
    }

    pub fn list_posts(
        &self,
        salon_id: Option<i64>,
        before_timestamp: Option<i64>,
        limit: i64,
    ) -> std::result::Result<Vec<FeedPost>, String> {
        self.with_conn(|conn| {
            let ids: Vec<i64> = match (salon_id, before_timestamp) {
                (Some(sid), Some(before_ts)) => {
                    let mut statement = conn
                        .prepare(
                            "SELECT id FROM posts WHERE salon_id = ?1 AND created_at < ?2
                             ORDER BY created_at DESC LIMIT ?3",
                        )
                        .map_err(to_string)?;
                    let rows = statement
                        .query_map(params![sid, before_ts, limit], |row| row.get::<_, i64>(0))
                        .map_err(to_string)?
                        .collect::<rusqlite::Result<Vec<_>>>()
                        .map_err(to_string)?;
                    rows
                }
                (Some(sid), None) => {
                    let mut statement = conn
                        .prepare(
                            "SELECT id FROM posts WHERE salon_id = ?1
                             ORDER BY created_at DESC LIMIT ?2",
                        )
                        .map_err(to_string)?;
                    let rows = statement
                        .query_map(params![sid, limit], |row| row.get::<_, i64>(0))
                        .map_err(to_string)?
                        .collect::<rusqlite::Result<Vec<_>>>()
                        .map_err(to_string)?;
                    rows
                }
                (None, Some(before_ts)) => {
                    let mut statement = conn
                        .prepare(
                            "SELECT id FROM posts WHERE created_at < ?1
                             ORDER BY created_at DESC LIMIT ?2",
                        )
                        .map_err(to_string)?;
                    let rows = statement
                        .query_map(params![before_ts, limit], |row| row.get::<_, i64>(0))
                        .map_err(to_string)?
                        .collect::<rusqlite::Result<Vec<_>>>()
                        .map_err(to_string)?;
                    rows
                }
                (None, None) => {
                    let mut statement = conn
                        .prepare("SELECT id FROM posts ORDER BY created_at DESC LIMIT ?1")
                        .map_err(to_string)?;
                    let rows = statement
                        .query_map(params![limit], |row| row.get::<_, i64>(0))
                        .map_err(to_string)?
                        .collect::<rusqlite::Result<Vec<_>>>()
                        .map_err(to_string)?;
                    rows
                }
            };

            ids.into_iter()
                .map(|post_id| self.fetch_post(conn, post_id))
                .collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    pub fn upload_file(
        &self,
        salon_id: i64,
        uploader_id: i64,
        original_name: &str,
        kind: &str,
        storage_path: &str,
        size_bytes: i64,
        extracted_text: Option<&str>,
    ) -> std::result::Result<FileInfo, String> {
        self.with_conn(|conn| {
            require_salon_member(conn, salon_id, uploader_id)?;
            let now = now_ts();
            conn.execute(
                "INSERT INTO files (
                    salon_id, uploader_id, original_name, kind, storage_path,
                    size_bytes, extracted_text, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    salon_id,
                    uploader_id,
                    original_name,
                    kind,
                    storage_path,
                    size_bytes,
                    extracted_text,
                    now
                ],
            )
            .map_err(to_string)?;
            let file_id = conn.last_insert_rowid();
            conn.execute(
                "INSERT INTO files_fts(rowid, extracted_text) VALUES (?1, ?2)",
                params![file_id, extracted_text.unwrap_or("")],
            )
            .map_err(to_string)?;
            fetch_file_info(conn, file_id)
        })
    }

    pub fn get_file(&self, id: i64) -> std::result::Result<FileInfo, String> {
        self.with_conn(|conn| fetch_file_info(conn, id))
    }

    pub fn get_file_text(&self, id: i64) -> std::result::Result<Option<String>, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT extracted_text FROM files WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(to_string)?
            .ok_or_else(|| format!("file {} not found", id))
        })
    }

    pub fn get_file_storage_path(&self, id: i64) -> std::result::Result<String, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT storage_path FROM files WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(to_string)?
            .ok_or_else(|| format!("file {} not found", id))
        })
    }

    pub fn list_salon_files(&self, salon_id: i64, limit: i64) -> std::result::Result<Vec<FileInfo>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT id, salon_id, uploader_id, original_name, kind, size_bytes, created_at
                     FROM files
                     WHERE salon_id = ?1
                     ORDER BY created_at DESC, id DESC
                     LIMIT ?2",
                )
                .map_err(to_string)?;
            let rows = statement
                .query_map(params![salon_id, limit], map_file_info)
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            Ok(rows)
        })
    }

    pub fn list_post_files(&self, post_id: i64) -> std::result::Result<Vec<FileInfo>, String> {
        self.with_conn(|conn| self.fetch_post_files(conn, post_id))
    }

    pub fn attach_files_to_post(
        &self,
        post_id: i64,
        file_ids: &[i64],
    ) -> std::result::Result<(), String> {
        self.with_conn(|conn| self.attach_files_to_post_conn(conn, post_id, file_ids))
    }

    pub fn search_files(
        &self,
        salon_id: i64,
        query: &str,
        limit: i64,
    ) -> std::result::Result<Vec<FileSearchResult>, String> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(self
                .list_salon_files(salon_id, limit)?
                .into_iter()
                .map(|file| FileSearchResult {
                    file,
                    snippet: String::new(),
                })
                .collect());
        }

        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT f.id, f.salon_id, f.uploader_id, f.original_name, f.kind,
                            f.size_bytes, f.created_at,
                            snippet(files_fts, 0, '', '', '…', 20)
                     FROM files_fts
                     JOIN files f ON f.id = files_fts.rowid
                     WHERE files_fts MATCH ?1 AND f.salon_id = ?2
                     ORDER BY rank
                     LIMIT ?3",
                )
                .map_err(to_string)?;
            let rows = statement
                .query_map(params![trimmed, salon_id, limit], |row| {
                    Ok(FileSearchResult {
                        file: FileInfo {
                            id: row.get(0)?,
                            salon_id: row.get(1)?,
                            uploader_id: row.get(2)?,
                            original_name: row.get(3)?,
                            kind: row.get(4)?,
                            size_bytes: row.get(5)?,
                            created_at: row.get(6)?,
                        },
                        snippet: row.get(7)?,
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            Ok(rows)
        })
    }

    pub fn list_salons(&self) -> std::result::Result<Vec<Salon>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT s.id, s.name, s.topic, s.created_by, s.created_at, s.last_post_at,
                            (SELECT COUNT(*) FROM salon_members m WHERE m.salon_id = s.id) AS member_count
                     FROM salons s
                     ORDER BY COALESCE(s.last_post_at, 0) DESC, s.created_at DESC",
                )
                .map_err(to_string)?;
            let salons = statement
                .query_map([], |row| {
                    Ok(Salon {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        topic: row.get(2)?,
                        created_by: row.get(3)?,
                        created_at: row.get(4)?,
                        last_post_at: row.get(5)?,
                        member_count: row.get(6)?,
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            Ok(salons)
        })
    }

    pub fn get_salon(&self, salon_id: i64) -> std::result::Result<Salon, String> {
        self.with_conn(|conn| fetch_salon(conn, salon_id))
    }

    pub fn create_salon(
        &self,
        name: &str,
        topic: Option<&str>,
        creator_id: i64,
        member_actor_ids: &[i64],
    ) -> std::result::Result<Salon, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("salon name cannot be empty".to_string());
        }
        if name.chars().count() > 60 {
            return Err("salon name is too long".to_string());
        }
        let topic = topic.map(str::trim).filter(|v| !v.is_empty()).map(str::to_string);

        self.with_conn(|conn| {
            let creator_kind: String = conn
                .query_row(
                    "SELECT kind FROM actors WHERE id = ?1",
                    params![creator_id],
                    |row| row.get(0),
                )
                .map_err(to_string)?;
            if creator_kind != "human" {
                return Err("only human actors can create salons".to_string());
            }

            let existing_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM salons", [], |row| row.get(0))
                .map_err(to_string)?;
            if existing_count as usize >= MAX_SALONS_V1 {
                return Err(format!(
                    "salon limit reached (max {})",
                    MAX_SALONS_V1
                ));
            }

            let now = now_ts();
            let tx = conn.unchecked_transaction().map_err(to_string)?;

            tx.execute(
                "INSERT INTO salons (name, topic, created_by, created_at, last_post_at)
                 VALUES (?1, ?2, ?3, ?4, NULL)",
                params![name, topic, creator_id, now],
            )
            .map_err(to_string)?;
            let salon_id = tx.last_insert_rowid();

            tx.execute(
                "INSERT OR IGNORE INTO salon_members (salon_id, actor_id, joined_at)
                 VALUES (?1, ?2, ?3)",
                params![salon_id, creator_id, now],
            )
            .map_err(to_string)?;

            for actor_id in member_actor_ids {
                let kind: Option<String> = tx
                    .query_row(
                        "SELECT kind FROM actors WHERE id = ?1",
                        params![actor_id],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(to_string)?;
                let Some(kind) = kind else { continue };
                if kind != "agent" {
                    continue;
                }
                tx.execute(
                    "INSERT OR IGNORE INTO salon_members (salon_id, actor_id, joined_at)
                     VALUES (?1, ?2, ?3)",
                    params![salon_id, actor_id, now],
                )
                .map_err(to_string)?;
            }

            let nomi_id: Option<i64> = tx
                .query_row(
                    "SELECT id FROM actors WHERE LOWER(handle) IN ('nomi', 'nuomi') LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .optional()
                .map_err(to_string)?;
            if let Some(nomi_id) = nomi_id {
                tx.execute(
                    "INSERT OR IGNORE INTO salon_members (salon_id, actor_id, joined_at)
                     VALUES (?1, ?2, ?3)",
                    params![salon_id, nomi_id, now],
                )
                .map_err(to_string)?;
            }

            tx.commit().map_err(to_string)?;

            fetch_salon(conn, salon_id)
        })
    }

    pub fn delete_salon(&self, salon_id: i64) -> std::result::Result<(), String> {
        if salon_id == GENERAL_SALON_ID {
            return Err("General workspace cannot be deleted".to_string());
        }

        let app_data_dir = self.app_data_dir();
        let storage_paths = self.with_conn(|conn| {
            let exists = conn
                .query_row("SELECT 1 FROM salons WHERE id = ?1", params![salon_id], |_| Ok(()))
                .optional()
                .map_err(to_string)?
                .is_some();
            if !exists {
                return Err("salon not found".to_string());
            }

            let mut file_stmt = conn
                .prepare("SELECT storage_path FROM files WHERE salon_id = ?1")
                .map_err(to_string)?;
            let storage_paths = file_stmt
                .query_map(params![salon_id], |row| row.get::<_, String>(0))
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            let mut post_stmt = conn
                .prepare(
                    "WITH RECURSIVE subtree(id, depth) AS (
                        SELECT id, 0 FROM posts WHERE salon_id = ?1 AND parent_id IS NULL
                        UNION ALL
                        SELECT p.id, subtree.depth + 1
                        FROM posts p
                        JOIN subtree ON p.parent_id = subtree.id
                    )
                    SELECT id FROM subtree ORDER BY depth DESC, id DESC",
                )
                .map_err(to_string)?;
            let post_ids = post_stmt
                .query_map(params![salon_id], |row| row.get::<_, i64>(0))
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            let tx = conn.unchecked_transaction().map_err(to_string)?;

            tx.execute("DELETE FROM notifications WHERE post_id IN (SELECT id FROM posts WHERE salon_id = ?1)", params![salon_id])
                .map_err(to_string)?;
            tx.execute("DELETE FROM likes WHERE post_id IN (SELECT id FROM posts WHERE salon_id = ?1)", params![salon_id])
                .map_err(to_string)?;
            tx.execute("DELETE FROM post_media WHERE post_id IN (SELECT id FROM posts WHERE salon_id = ?1)", params![salon_id])
                .map_err(to_string)?;
            tx.execute(
                "DELETE FROM agent_queue WHERE salon_id = ?1 OR context_post_id IN (SELECT id FROM posts WHERE salon_id = ?1)",
                params![salon_id],
            )
            .map_err(to_string)?;
            tx.execute("DELETE FROM task_logs WHERE task_id IN (SELECT id FROM tasks WHERE salon_id = ?1)", params![salon_id])
                .map_err(to_string)?;
            tx.execute("DELETE FROM tasks WHERE salon_id = ?1", params![salon_id]).map_err(to_string)?;
            tx.execute("DELETE FROM post_files WHERE file_id IN (SELECT id FROM files WHERE salon_id = ?1)", params![salon_id])
                .map_err(to_string)?;
            tx.execute("DELETE FROM files_fts WHERE rowid IN (SELECT id FROM files WHERE salon_id = ?1)", params![salon_id])
                .map_err(to_string)?;
            tx.execute("DELETE FROM files WHERE salon_id = ?1", params![salon_id]).map_err(to_string)?;

            for post_id in post_ids {
                tx.execute("DELETE FROM posts WHERE id = ?1", params![post_id])
                    .map_err(to_string)?;
            }

            tx.execute("DELETE FROM salon_members WHERE salon_id = ?1", params![salon_id])
                .map_err(to_string)?;
            let deleted = tx.execute("DELETE FROM salons WHERE id = ?1", params![salon_id]).map_err(to_string)?;
            if deleted == 0 {
                return Err("salon not found".to_string());
            }

            tx.commit().map_err(to_string)?;
            Ok(storage_paths)
        })?;

        let uploads_dir = crate::services::files::uploads_dir(&app_data_dir).map_err(|error| error.to_string())?;
        for storage_path in storage_paths {
            let file_path = uploads_dir.join(storage_path);
            let _ = fs::remove_file(file_path);
        }

        Ok(())
    }

    pub fn list_salon_members(
        &self,
        salon_id: i64,
    ) -> std::result::Result<Vec<SalonMember>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT m.salon_id, m.joined_at, a.id, a.kind, a.handle, a.display_name,
                            a.avatar_seed, a.specialty
                     FROM salon_members m
                     JOIN actors a ON a.id = m.actor_id
                     WHERE m.salon_id = ?1
                     ORDER BY CASE a.kind WHEN 'human' THEN 0 ELSE 1 END, a.display_name ASC",
                )
                .map_err(to_string)?;
            let members = statement
                .query_map(params![salon_id], |row| {
                    Ok(SalonMember {
                        salon_id: row.get(0)?,
                        joined_at: row.get(1)?,
                        actor: ActorSummary {
                            id: row.get(2)?,
                            kind: row.get(3)?,
                            handle: row.get(4)?,
                            display_name: row.get(5)?,
                            avatar_seed: row.get(6)?,
                            specialty: row.get(7)?,
                        },
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            Ok(members)
        })
    }

    pub fn add_salon_member(
        &self,
        salon_id: i64,
        actor_id: i64,
    ) -> std::result::Result<SalonMember, String> {
        self.with_conn(|conn| {
            let _: i64 = conn
                .query_row(
                    "SELECT id FROM salons WHERE id = ?1",
                    params![salon_id],
                    |row| row.get(0),
                )
                .map_err(|_| "salon not found".to_string())?;
            let (kind, handle, display_name, avatar_seed, specialty): (
                String,
                String,
                String,
                Option<String>,
                Option<String>,
            ) = conn
                .query_row(
                    "SELECT kind, handle, display_name, avatar_seed, specialty
                     FROM actors WHERE id = ?1",
                    params![actor_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
                )
                .map_err(|_| "actor not found".to_string())?;
            if kind != "agent" {
                return Err("only agent actors can be added as members".to_string());
            }
            let now = now_ts();
            conn.execute(
                "INSERT OR IGNORE INTO salon_members (salon_id, actor_id, joined_at)
                 VALUES (?1, ?2, ?3)",
                params![salon_id, actor_id, now],
            )
            .map_err(to_string)?;
            let joined_at: i64 = conn
                .query_row(
                    "SELECT joined_at FROM salon_members WHERE salon_id = ?1 AND actor_id = ?2",
                    params![salon_id, actor_id],
                    |row| row.get(0),
                )
                .map_err(to_string)?;
            Ok(SalonMember {
                salon_id,
                joined_at,
                actor: ActorSummary {
                    id: actor_id,
                    kind,
                    handle,
                    display_name,
                    avatar_seed,
                    specialty,
                },
            })
        })
    }

    pub fn is_salon_member(
        &self,
        salon_id: i64,
        actor_id: i64,
    ) -> std::result::Result<bool, String> {
        self.with_conn(|conn| is_salon_member_conn(conn, salon_id, actor_id))
    }

    pub fn list_active_salons_for_actor(
        &self,
        actor_id: i64,
    ) -> std::result::Result<Vec<Salon>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT s.id, s.name, s.topic, s.created_by, s.created_at, s.last_post_at,
                            (SELECT COUNT(*) FROM salon_members m2 WHERE m2.salon_id = s.id) AS member_count
                     FROM salons s
                     JOIN salon_members m ON m.salon_id = s.id
                     WHERE m.actor_id = ?1
                     ORDER BY COALESCE(s.last_post_at, 0) DESC, s.created_at DESC",
                )
                .map_err(to_string)?;
            let salons = statement
                .query_map(params![actor_id], |row| {
                    Ok(Salon {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        topic: row.get(2)?,
                        created_by: row.get(3)?,
                        created_at: row.get(4)?,
                        last_post_at: row.get(5)?,
                        member_count: row.get(6)?,
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            Ok(salons)
        })
    }

    pub fn touch_salon_last_post(&self, salon_id: i64) -> std::result::Result<(), String> {
        self.with_conn(|conn| touch_salon_last_post(conn, salon_id, now_ts()))
    }

    pub fn create_task(
        &self,
        salon_id: i64,
        title: &str,
        description: Option<&str>,
        created_by: i64,
        assigned_to: Option<i64>,
    ) -> std::result::Result<Task, String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("task title cannot be empty".to_string());
        }
        let description = description
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        self.with_conn(|conn| {
            let _: i64 = conn
                .query_row("SELECT id FROM salons WHERE id = ?1", params![salon_id], |row| row.get(0))
                .map_err(|_| "salon not found".to_string())?;
            let _: i64 = conn
                .query_row("SELECT id FROM actors WHERE id = ?1", params![created_by], |row| row.get(0))
                .map_err(|_| "creator actor not found".to_string())?;
            if let Some(assignee) = assigned_to {
                let _: i64 = conn
                    .query_row("SELECT id FROM actors WHERE id = ?1", params![assignee], |row| row.get(0))
                    .map_err(|_| "assigned actor not found".to_string())?;
            }

            let now = now_ts();
            let tx = conn.unchecked_transaction().map_err(to_string)?;
            tx.execute(
                "INSERT INTO tasks (salon_id, title, description, status, created_by, assigned_to, deliverable_post_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 'todo', ?4, ?5, NULL, ?6, ?6)",
                params![salon_id, title, description, created_by, assigned_to, now],
            )
            .map_err(to_string)?;
            let task_id = tx.last_insert_rowid();
            insert_task_log(&tx, task_id, created_by, "created", None)?;
            tx.commit().map_err(to_string)?;

            fetch_task(conn, task_id)
        })
    }

    pub fn list_tasks(
        &self,
        salon_id: i64,
        status_filter: Option<&str>,
        assigned_to_filter: Option<&str>,
        limit: i64,
    ) -> std::result::Result<Vec<Task>, String> {
        let limit = limit.clamp(1, 100);
        let status_filter = status_filter.map(str::trim).filter(|value| !value.is_empty());
        let assigned_to_filter = assigned_to_filter.map(str::trim).filter(|value| !value.is_empty());

        self.with_conn(|conn| {
            let assigned_to_id = assigned_to_filter
                .map(|handle| fetch_actor_id_by_handle(conn, handle))
                .transpose()?;
            let sql = match (status_filter.is_some(), assigned_to_id.is_some()) {
                (true, true) => {
                    "SELECT id FROM tasks WHERE salon_id = ?1 AND status = ?2 AND assigned_to = ?3
                     ORDER BY CASE status WHEN 'in_progress' THEN 0 WHEN 'todo' THEN 1 ELSE 2 END,
                              updated_at DESC LIMIT ?4"
                }
                (true, false) => {
                    "SELECT id FROM tasks WHERE salon_id = ?1 AND status = ?2
                     ORDER BY CASE status WHEN 'in_progress' THEN 0 WHEN 'todo' THEN 1 ELSE 2 END,
                              updated_at DESC LIMIT ?3"
                }
                (false, true) => {
                    "SELECT id FROM tasks WHERE salon_id = ?1 AND assigned_to = ?2
                     ORDER BY CASE status WHEN 'in_progress' THEN 0 WHEN 'todo' THEN 1 ELSE 2 END,
                              updated_at DESC LIMIT ?3"
                }
                (false, false) => {
                    "SELECT id FROM tasks WHERE salon_id = ?1
                     ORDER BY CASE status WHEN 'in_progress' THEN 0 WHEN 'todo' THEN 1 ELSE 2 END,
                              updated_at DESC LIMIT ?2"
                }
            };

            let mut statement = conn.prepare(sql).map_err(to_string)?;
            let ids = match (status_filter, assigned_to_id) {
                (Some(status), Some(assigned_to)) => statement
                    .query_map(params![salon_id, status, assigned_to, limit], |row| row.get::<_, i64>(0))
                    .map_err(to_string)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_string)?,
                (Some(status), None) => statement
                    .query_map(params![salon_id, status, limit], |row| row.get::<_, i64>(0))
                    .map_err(to_string)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_string)?,
                (None, Some(assigned_to)) => statement
                    .query_map(params![salon_id, assigned_to, limit], |row| row.get::<_, i64>(0))
                    .map_err(to_string)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_string)?,
                (None, None) => statement
                    .query_map(params![salon_id, limit], |row| row.get::<_, i64>(0))
                    .map_err(to_string)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_string)?,
            };

            ids.into_iter()
                .map(|task_id| fetch_task(conn, task_id))
                .collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    pub fn get_task(&self, task_id: i64) -> std::result::Result<Task, String> {
        self.with_conn(|conn| fetch_task(conn, task_id))
    }

    pub fn claim_task(&self, task_id: i64, actor_id: i64) -> std::result::Result<Task, String> {
        self.with_conn(|conn| {
            let task = fetch_task(conn, task_id)?;
            if task.status == "done" {
                return Err("cannot claim a completed task".to_string());
            }

            let now = now_ts();
            let tx = conn.unchecked_transaction().map_err(to_string)?;
            tx.execute(
                "UPDATE tasks
                 SET status = 'in_progress', assigned_to = ?2, updated_at = ?3
                 WHERE id = ?1",
                params![task_id, actor_id, now],
            )
            .map_err(to_string)?;
            insert_task_log(&tx, task_id, actor_id, "claimed", None)?;
            tx.commit().map_err(to_string)?;

            fetch_task(conn, task_id)
        })
    }

    pub fn complete_task(
        &self,
        task_id: i64,
        actor_id: i64,
        deliverable_post_id: Option<i64>,
    ) -> std::result::Result<Task, String> {
        self.with_conn(|conn| {
            let task = fetch_task(conn, task_id)?;
            if let Some(post_id) = deliverable_post_id {
                let post_salon_id = fetch_post_salon_id(conn, post_id)?;
                if post_salon_id != task.salon_id {
                    return Err(format!(
                        "deliverable post {} belongs to salon {}, not task salon {}",
                        post_id, post_salon_id, task.salon_id
                    ));
                }
            }

            let now = now_ts();
            let tx = conn.unchecked_transaction().map_err(to_string)?;
            tx.execute(
                "UPDATE tasks
                 SET status = 'done',
                     assigned_to = COALESCE(assigned_to, ?2),
                     deliverable_post_id = ?3,
                     updated_at = ?4
                 WHERE id = ?1",
                params![task_id, actor_id, deliverable_post_id, now],
            )
            .map_err(to_string)?;
            insert_task_log(&tx, task_id, actor_id, "completed", None)?;
            tx.commit().map_err(to_string)?;

            fetch_task(conn, task_id)
        })
    }

    pub fn update_task(
        &self,
        task_id: i64,
        title: Option<&str>,
        description: Option<Option<&str>>,
        assigned_to: Option<Option<i64>>,
        status: Option<&str>,
    ) -> std::result::Result<Task, String> {
        self.with_conn(|conn| {
            let task = fetch_task(conn, task_id)?;
            let next_title = title.map(str::trim).filter(|value| !value.is_empty()).unwrap_or(&task.title);
            let next_description = match description {
                Some(Some(value)) => {
                    let trimmed = value.trim();
                    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
                }
                Some(None) => None,
                None => task.description.clone(),
            };
            let next_assigned_to = match assigned_to {
                Some(value) => value,
                None => task.assigned_to,
            };
            let next_status = status.unwrap_or(&task.status);
            validate_task_status(next_status)?;

            if let Some(assignee) = next_assigned_to {
                let _: i64 = conn
                    .query_row("SELECT id FROM actors WHERE id = ?1", params![assignee], |row| row.get(0))
                    .map_err(|_| "assigned actor not found".to_string())?;
            }

            let now = now_ts();
            let tx = conn.unchecked_transaction().map_err(to_string)?;
            tx.execute(
                "UPDATE tasks
                 SET title = ?2,
                     description = ?3,
                     assigned_to = ?4,
                     status = ?5,
                     updated_at = ?6
                 WHERE id = ?1",
                params![task_id, next_title, next_description, next_assigned_to, next_status, now],
            )
            .map_err(to_string)?;

            if task.assigned_to != next_assigned_to {
                insert_task_log(&tx, task_id, task.created_by, "reassigned", None)?;
            } else if task.status != next_status && next_status == "todo" {
                insert_task_log(&tx, task_id, task.created_by, "reopened", None)?;
            } else if task.title != next_title || task.description != next_description {
                insert_task_log(&tx, task_id, task.created_by, "note", Some("task updated"))?;
            }
            tx.commit().map_err(to_string)?;

            fetch_task(conn, task_id)
        })
    }

    pub fn delete_task(&self, task_id: i64) -> std::result::Result<(), String> {
        self.with_conn(|conn| {
            let deleted = conn
                .execute("DELETE FROM tasks WHERE id = ?1", params![task_id])
                .map_err(to_string)?;
            if deleted == 0 {
                return Err("task not found".to_string());
            }
            Ok(())
        })
    }

    pub fn reopen_task(&self, task_id: i64, actor_id: i64) -> std::result::Result<Task, String> {
        self.with_conn(|conn| {
            let now = now_ts();
            let tx = conn.unchecked_transaction().map_err(to_string)?;
            tx.execute(
                "UPDATE tasks
                 SET status = 'todo', deliverable_post_id = NULL, updated_at = ?2
                 WHERE id = ?1",
                params![task_id, now],
            )
            .map_err(to_string)?;
            if tx.changes() == 0 {
                return Err("task not found".to_string());
            }
            insert_task_log(&tx, task_id, actor_id, "reopened", None)?;
            tx.commit().map_err(to_string)?;
            fetch_task(conn, task_id)
        })
    }

    pub fn post_salon_id(&self, post_id: i64) -> std::result::Result<i64, String> {
        self.with_conn(|conn| fetch_post_salon_id(conn, post_id))
    }

    pub fn delete_post_as_admin(&self, post_id: i64) -> std::result::Result<Vec<i64>, String> {
        self.with_conn(|conn| {
            let exists = conn
                .query_row(
                    "SELECT 1 FROM posts WHERE id = ?1",
                    params![post_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(to_string)?
                .is_some();

            if !exists {
                return Err("post not found".to_string());
            }

            let deleted_ids = {
                let mut statement = conn
                    .prepare(
                        "WITH RECURSIVE subtree(id, depth) AS (
                           SELECT id, 0 FROM posts WHERE id = ?1
                           UNION ALL
                           SELECT posts.id, subtree.depth + 1
                           FROM posts
                           JOIN subtree ON posts.parent_id = subtree.id
                         )
                         SELECT id FROM subtree ORDER BY depth DESC",
                    )
                    .map_err(to_string)?;
                let rows = statement
                    .query_map(params![post_id], |row| row.get::<_, i64>(0))
                    .map_err(to_string)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_string)?;
                rows
            };

            let transaction = conn.unchecked_transaction().map_err(to_string)?;
            for id in &deleted_ids {
                transaction
                    .execute("DELETE FROM likes WHERE post_id = ?1", params![id])
                    .map_err(to_string)?;
                transaction
                    .execute("DELETE FROM post_media WHERE post_id = ?1", params![id])
                    .map_err(to_string)?;
                transaction
                    .execute("DELETE FROM notifications WHERE post_id = ?1", params![id])
                    .map_err(to_string)?;
                transaction
                    .execute("DELETE FROM agent_queue WHERE context_post_id = ?1", params![id])
                    .map_err(to_string)?;
                transaction
                    .execute("DELETE FROM posts WHERE id = ?1", params![id])
                    .map_err(to_string)?;
            }
            transaction.commit().map_err(to_string)?;

            Ok(deleted_ids)
        })
    }

    pub fn search_posts(
        &self,
        query: Option<&str>,
        actor_handle: Option<&str>,
        limit: i64,
    ) -> std::result::Result<Vec<FeedPost>, String> {
        self.with_conn(|conn| {
            let mut sql = "SELECT p.id FROM posts p JOIN actors a ON p.actor_id = a.id WHERE 1=1".to_string();
            let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

            if let Some(q) = query {
                sql.push_str(" AND (p.body LIKE ? OR p.quote_body LIKE ?)");
                let like_query = format!("%{}%", q);
                params_vec.push(rusqlite::types::Value::Text(like_query.clone()));
                params_vec.push(rusqlite::types::Value::Text(like_query));
            }

            if let Some(h) = actor_handle {
                sql.push_str(" AND a.handle = ?");
                params_vec.push(rusqlite::types::Value::Text(h.to_string()));
            }

            sql.push_str(" ORDER BY p.created_at DESC LIMIT ?");
            params_vec.push(rusqlite::types::Value::Integer(limit));

            let mut statement = conn.prepare(&sql).map_err(to_string)?;
            let rows = statement
                .query_map(rusqlite::params_from_iter(params_vec), |row| row.get::<_, i64>(0))
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            rows.into_iter()
                .map(|id| self.fetch_post(conn, id))
                .collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    pub fn get_thread(&self, post_id: i64) -> std::result::Result<Vec<FeedPost>, String> {
        self.with_conn(|conn| {
            let mut root_id = post_id;
            loop {
                let parent_id = conn
                    .query_row(
                        "SELECT parent_id FROM posts WHERE id = ?1",
                        params![root_id],
                        |row| row.get::<_, Option<i64>>(0),
                    )
                    .optional()
                    .map_err(to_string)?
                    .flatten();

                match parent_id {
                    Some(parent_id) => root_id = parent_id,
                    None => break,
                }
            }

            let mut statement = conn
                .prepare(
                    "WITH RECURSIVE thread(id) AS (
                        SELECT id FROM posts WHERE id = ?1
                        UNION ALL
                        SELECT posts.id
                        FROM posts
                        JOIN thread ON posts.parent_id = thread.id
                    )
                    SELECT id FROM thread
                    ORDER BY (SELECT created_at FROM posts WHERE posts.id = thread.id) ASC",
                )
                .map_err(to_string)?;
            let ids = statement
                .query_map(params![root_id], |row| row.get::<_, i64>(0))
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            ids.into_iter()
                .map(|thread_post_id| self.fetch_post(conn, thread_post_id))
                .collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    pub fn create_human_post(
        &self,
        body: &str,
        salon_id: i64,
    ) -> std::result::Result<FeedPost, String> {
        self.create_human_post_with_files(body, salon_id, &[])
    }

    pub fn create_human_post_with_files(
        &self,
        body: &str,
        salon_id: i64,
        file_ids: &[i64],
    ) -> std::result::Result<FeedPost, String> {
        self.with_conn(|conn| {
            let actor_id = self.human_actor_id(conn)?;
            require_salon_member(conn, salon_id, actor_id)?;
            let created_at = now_ts();

            conn.execute(
                "INSERT INTO posts (actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
                 VALUES (?1, 'original', NULL, ?2, NULL, ?3, 'manual', ?4)",
                params![actor_id, salon_id, body, created_at],
            )
            .map_err(to_string)?;

            let post_id = conn.last_insert_rowid();
            self.attach_files_to_post_conn(conn, post_id, file_ids)?;
            self.enqueue_mentions_for_post(conn, actor_id, post_id, salon_id, body)?;
            touch_salon_last_post(conn, salon_id, created_at)?;
            self.fetch_post(conn, post_id)
        })
    }

    pub fn toggle_like(&self, post_id: i64) -> std::result::Result<bool, String> {
        self.with_conn(|conn| {
            let actor_id = self.human_actor_id(conn)?;
            let existing = conn
                .query_row(
                    "SELECT 1 FROM likes WHERE actor_id = ?1 AND post_id = ?2",
                    params![actor_id, post_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(to_string)?
                .is_some();

            if existing {
                conn.execute(
                    "DELETE FROM likes WHERE actor_id = ?1 AND post_id = ?2",
                    params![actor_id, post_id],
                )
                .map_err(to_string)?;
                Ok(false)
            } else {
                conn.execute(
                    "INSERT INTO likes (actor_id, post_id, created_at) VALUES (?1, ?2, ?3)",
                    params![actor_id, post_id, now_ts()],
                )
                .map_err(to_string)?;
                Ok(true)
            }
        })
    }

    pub fn repost_as_human(
        &self,
        post_id: i64,
        quote_body: Option<&str>,
    ) -> std::result::Result<FeedPost, String> {
        self.with_conn(|conn| {
            let actor_id = self.human_actor_id(conn)?;
            if let Some(existing_id) = self.find_existing_repost(conn, actor_id, post_id)? {
                return self.fetch_post(conn, existing_id);
            }

            let parent_salon = fetch_post_salon_id(conn, post_id)?;
            require_salon_member(conn, parent_salon, actor_id)?;
            let now = now_ts();

            conn.execute(
                "INSERT INTO posts (actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
                 VALUES (?1, 'repost', ?2, ?3, ?4, NULL, 'manual', ?5)",
                params![actor_id, post_id, parent_salon, quote_body, now],
            )
            .map_err(to_string)?;

            let new_post_id = conn.last_insert_rowid();
            if let Some(quote_body) = quote_body {
                self.enqueue_mentions_for_post(conn, actor_id, new_post_id, parent_salon, quote_body)?;
            }
            touch_salon_last_post(conn, parent_salon, now)?;
            self.fetch_post(conn, new_post_id)
        })
    }

    pub fn list_actors(&self) -> std::result::Result<Vec<Actor>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT id, kind, handle, display_name, avatar_seed, bio, specialty, persona_prompt,
                            model_provider, model_name, active_hours, posts_per_day, created_at
                     FROM actors
                     ORDER BY CASE kind WHEN 'human' THEN 0 ELSE 1 END, display_name ASC",
                )
                .map_err(to_string)?;

            let actors = statement
                .query_map([], map_actor)
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            Ok(actors)
        })
    }

    pub fn get_actor(&self, handle: &str) -> std::result::Result<Actor, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, kind, handle, display_name, avatar_seed, bio, specialty, persona_prompt,
                        model_provider, model_name, active_hours, posts_per_day, created_at
                 FROM actors
                 WHERE LOWER(handle) = LOWER(?1)",
                params![handle],
                map_actor,
            )
            .map_err(to_string)
        })
    }

    pub fn get_actor_id_by_handle(&self, handle: &str) -> std::result::Result<i64, String> {
        self.with_conn(|conn| fetch_actor_id_by_handle(conn, handle))
    }

    pub fn list_agent_runs(&self, limit: i64) -> std::result::Result<Vec<AgentRun>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare(
                    "SELECT ar.id, ar.actor_id, a.handle, a.display_name, ar.trigger, ar.started_at, ar.finished_at,
                            ar.prompt_tokens, ar.completion_tokens, ar.tool_calls, ar.error
                     FROM agent_runs ar
                     JOIN actors a ON a.id = ar.actor_id
                     ORDER BY ar.started_at DESC
                     LIMIT ?1",
                )
                .map_err(to_string)?;

            let runs = statement
                .query_map(params![limit], |row| {
                    Ok(AgentRun {
                        id: row.get(0)?,
                        actor_id: row.get(1)?,
                        actor_handle: row.get(2)?,
                        actor_display_name: row.get(3)?,
                        trigger: row.get(4)?,
                        started_at: row.get(5)?,
                        finished_at: row.get(6)?,
                        prompt_tokens: row.get(7)?,
                        completion_tokens: row.get(8)?,
                        tool_calls: row.get(9)?,
                        error: row.get(10)?,
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            Ok(runs)
        })
    }

    pub fn has_successful_run_for_trigger_since(
        &self,
        trigger: &str,
        since_ts: i64,
    ) -> std::result::Result<bool, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT 1
                 FROM agent_runs
                 WHERE trigger = ?1
                   AND started_at >= ?2
                   AND error IS NULL
                 LIMIT 1",
                params![trigger, since_ts],
                |_| Ok(()),
            )
            .optional()
            .map_err(to_string)
            .map(|value| value.is_some())
        })
    }

    pub fn get_settings(&self) -> std::result::Result<Vec<SettingEntry>, String> {
        self.with_conn(|conn| {
            let mut statement = conn
                .prepare("SELECT key, value FROM settings ORDER BY key ASC")
                .map_err(to_string)?;

            let settings = statement
                .query_map([], |row| {
                    Ok(SettingEntry {
                        key: row.get(0)?,
                        value: row.get(1)?,
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;

            Ok(settings)
        })
    }

    pub fn set_settings(&self, settings: &[SettingEntry]) -> std::result::Result<(), String> {
        self.with_conn(|conn| {
            let transaction = conn.unchecked_transaction().map_err(to_string)?;

            for entry in settings {
                transaction
                    .execute(
                        "INSERT INTO settings (key, value) VALUES (?1, ?2)
                         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                        params![entry.key, entry.value],
                    )
                    .map_err(to_string)?;
            }

            transaction.commit().map_err(to_string)?;
            Ok(())
        })
    }

    pub fn get_setting_value(&self, key: &str) -> std::result::Result<Option<String>, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(to_string)
        })
    }

    pub fn write_api_key(&self, provider: &str, key: &str) -> std::result::Result<(), String> {
        let config_dir = config_dir();
        fs::create_dir_all(&config_dir).map_err(to_string)?;

        let config_path = config_dir.join("keys.toml");
        let existing = fs::read_to_string(&config_path).unwrap_or_default();
        let mut keys = if existing.trim().is_empty() {
            ProviderKeys::default()
        } else {
            toml::from_str::<ProviderKeys>(&existing).unwrap_or_default()
        };

        keys.providers
            .insert(provider.to_string(), key.to_string());

        let serialized = toml::to_string(&keys).map_err(to_string)?;
        fs::write(config_path, serialized).map_err(to_string)
    }

    pub fn is_api_key_configured(&self, provider: &str) -> bool {
        let config_path = config_dir().join("keys.toml");
        let Ok(content) = fs::read_to_string(&config_path) else { return false; };
        let Ok(keys) = toml::from_str::<ProviderKeys>(&content) else { return false; };
        keys.providers.get(provider).map(|k| !k.trim().is_empty()).unwrap_or(false)
    }

    pub fn create_agent_run(&self, actor_id: i64, trigger: &str) -> std::result::Result<i64, String> {
        self.with_conn(|conn| {
            let active_run_id = conn
                .query_row(
                    "SELECT id FROM agent_runs WHERE actor_id = ?1 AND finished_at IS NULL LIMIT 1",
                    params![actor_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(to_string)?;

            if active_run_id.is_some() {
                return Err(format!("agent {} already has an active run", actor_id));
            }

            conn.execute(
                "INSERT INTO agent_runs (actor_id, trigger, started_at) VALUES (?1, ?2, ?3)",
                params![actor_id, trigger, now_ts()],
            )
            .map_err(to_string)?;
            Ok(conn.last_insert_rowid())
        })
    }

    pub fn finish_agent_run(
        &self,
        run_id: i64,
        prompt_tokens: Option<i64>,
        completion_tokens: Option<i64>,
        tool_calls: Option<String>,
        error: Option<String>,
    ) -> std::result::Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE agent_runs
                 SET finished_at = ?2,
                     prompt_tokens = ?3,
                     completion_tokens = ?4,
                     tool_calls = ?5,
                     error = ?6
                 WHERE id = ?1",
                params![run_id, now_ts(), prompt_tokens, completion_tokens, tool_calls, error],
            )
            .map_err(to_string)?;
            Ok(())
        })
    }

    pub fn create_post_as_actor(
        &self,
        actor_id: i64,
        body: &str,
        trigger: &str,
        salon_id: i64,
    ) -> std::result::Result<FeedPost, String> {
        self.create_post_as_actor_with_media(actor_id, body, trigger, salon_id, &[])
    }

    pub fn create_post_as_actor_with_media(
        &self,
        actor_id: i64,
        body: &str,
        trigger: &str,
        salon_id: i64,
        media: &[PostMediaInput],
    ) -> std::result::Result<FeedPost, String> {
        self.with_conn(|conn| {
            require_salon_member(conn, salon_id, actor_id)?;
            let now = now_ts();
            conn.execute(
                "INSERT INTO posts (actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
                 VALUES (?1, 'original', NULL, ?2, NULL, ?3, ?4, ?5)",
                params![actor_id, salon_id, body, normalize_post_trigger(trigger), now],
            )
            .map_err(to_string)?;
            let post_id = conn.last_insert_rowid();
            self.insert_post_media(conn, post_id, media)?;
            self.enqueue_mentions_for_post(conn, actor_id, post_id, salon_id, body)?;
            touch_salon_last_post(conn, salon_id, now)?;
            self.fetch_post(conn, post_id)
        })
    }

    pub fn create_post_as_actor_with_files(
        &self,
        actor_id: i64,
        body: &str,
        trigger: &str,
        salon_id: i64,
        file_ids: &[i64],
    ) -> std::result::Result<FeedPost, String> {
        self.with_conn(|conn| {
            require_salon_member(conn, salon_id, actor_id)?;
            let now = now_ts();
            conn.execute(
                "INSERT INTO posts (actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
                 VALUES (?1, 'original', NULL, ?2, NULL, ?3, ?4, ?5)",
                params![actor_id, salon_id, body, normalize_post_trigger(trigger), now],
            )
            .map_err(to_string)?;
            let post_id = conn.last_insert_rowid();
            self.attach_files_to_post_conn(conn, post_id, file_ids)?;
            self.enqueue_mentions_for_post(conn, actor_id, post_id, salon_id, body)?;
            touch_salon_last_post(conn, salon_id, now)?;
            self.fetch_post(conn, post_id)
        })
    }

    pub fn reply_as_human(
        &self,
        parent_id: i64,
        body: &str,
    ) -> std::result::Result<FeedPost, String> {
        self.reply_as_human_with_files(parent_id, body, &[])
    }

    pub fn reply_as_human_with_files(
        &self,
        parent_id: i64,
        body: &str,
        file_ids: &[i64],
    ) -> std::result::Result<FeedPost, String> {
        let actor_id = self.with_conn(|conn| self.human_actor_id(conn))?;
        self.reply_as_actor_with_files(actor_id, parent_id, body, "manual", file_ids)
    }

    pub fn reply_as_actor(
        &self,
        actor_id: i64,
        parent_id: i64,
        body: &str,
        trigger: &str,
    ) -> std::result::Result<FeedPost, String> {
        self.reply_as_actor_with_media(actor_id, parent_id, body, trigger, &[])
    }

    pub fn reply_as_actor_with_media(
        &self,
        actor_id: i64,
        parent_id: i64,
        body: &str,
        trigger: &str,
        media: &[PostMediaInput],
    ) -> std::result::Result<FeedPost, String> {
        self.with_conn(|conn| {
            let salon_id = fetch_post_salon_id(conn, parent_id)?;
            require_salon_member(conn, salon_id, actor_id)?;
            let now = now_ts();
            conn.execute(
                "INSERT INTO posts (actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
                 VALUES (?1, 'reply', ?2, ?3, NULL, ?4, ?5, ?6)",
                params![actor_id, parent_id, salon_id, body, normalize_post_trigger(trigger), now],
            )
            .map_err(to_string)?;
            let post_id = conn.last_insert_rowid();
            self.insert_post_media(conn, post_id, media)?;
            self.enqueue_mentions_for_post(conn, actor_id, post_id, salon_id, body)?;
            self.enqueue_reply_target(conn, actor_id, post_id, parent_id, salon_id)?;
            touch_salon_last_post(conn, salon_id, now)?;
            self.fetch_post(conn, post_id)
        })
    }

    pub fn reply_as_actor_with_files(
        &self,
        actor_id: i64,
        parent_id: i64,
        body: &str,
        trigger: &str,
        file_ids: &[i64],
    ) -> std::result::Result<FeedPost, String> {
        self.with_conn(|conn| {
            let salon_id = fetch_post_salon_id(conn, parent_id)?;
            require_salon_member(conn, salon_id, actor_id)?;
            let now = now_ts();
            conn.execute(
                "INSERT INTO posts (actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
                 VALUES (?1, 'reply', ?2, ?3, NULL, ?4, ?5, ?6)",
                params![actor_id, parent_id, salon_id, body, normalize_post_trigger(trigger), now],
            )
            .map_err(to_string)?;
            let post_id = conn.last_insert_rowid();
            self.attach_files_to_post_conn(conn, post_id, file_ids)?;
            self.enqueue_mentions_for_post(conn, actor_id, post_id, salon_id, body)?;
            self.enqueue_reply_target(conn, actor_id, post_id, parent_id, salon_id)?;
            touch_salon_last_post(conn, salon_id, now)?;
            self.fetch_post(conn, post_id)
        })
    }

    pub fn repost_as_actor(
        &self,
        actor_id: i64,
        parent_id: i64,
        quote_body: Option<&str>,
        trigger: &str,
    ) -> std::result::Result<FeedPost, String> {
        self.with_conn(|conn| {
            if let Some(existing_id) = self.find_existing_repost(conn, actor_id, parent_id)? {
                return self.fetch_post(conn, existing_id);
            }

            let salon_id = fetch_post_salon_id(conn, parent_id)?;
            require_salon_member(conn, salon_id, actor_id)?;
            let now = now_ts();
            conn.execute(
                "INSERT INTO posts (actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
                 VALUES (?1, 'repost', ?2, ?3, ?4, NULL, ?5, ?6)",
                params![actor_id, parent_id, salon_id, quote_body, normalize_post_trigger(trigger), now],
            )
            .map_err(to_string)?;
            let post_id = conn.last_insert_rowid();
            if let Some(quote_body) = quote_body {
                self.enqueue_mentions_for_post(conn, actor_id, post_id, salon_id, quote_body)?;
            }
            touch_salon_last_post(conn, salon_id, now)?;
            self.fetch_post(conn, post_id)
        })
    }

    pub fn like_as_actor(&self, actor_id: i64, post_id: i64) -> std::result::Result<bool, String> {
        self.with_conn(|conn| {
            let salon_id = fetch_post_salon_id(conn, post_id)?;
            require_salon_member(conn, salon_id, actor_id)?;
            let existing = conn
                .query_row(
                    "SELECT 1 FROM likes WHERE actor_id = ?1 AND post_id = ?2",
                    params![actor_id, post_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(to_string)?
                .is_some();

            if existing {
                Ok(false)
            } else {
                conn.execute(
                    "INSERT INTO likes (actor_id, post_id, created_at) VALUES (?1, ?2, ?3)",
                    params![actor_id, post_id, now_ts()],
                )
                .map_err(to_string)?;
                Ok(true)
            }
        })
    }

    pub fn like_as_actor_in_salon(
        &self,
        actor_id: i64,
        post_id: i64,
        salon_id: i64,
    ) -> std::result::Result<bool, String> {
        self.with_conn(|conn| {
            let post_salon_id = fetch_post_salon_id(conn, post_id)?;
            if post_salon_id != salon_id {
                return Err(format!(
                    "post {} belongs to salon {}, not current salon {}",
                    post_id, post_salon_id, salon_id
                ));
            }
            require_salon_member(conn, salon_id, actor_id)?;
            let existing = conn
                .query_row(
                    "SELECT 1 FROM likes WHERE actor_id = ?1 AND post_id = ?2",
                    params![actor_id, post_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(to_string)?
                .is_some();

            if existing {
                Ok(false)
            } else {
                conn.execute(
                    "INSERT INTO likes (actor_id, post_id, created_at) VALUES (?1, ?2, ?3)",
                    params![actor_id, post_id, now_ts()],
                )
                .map_err(to_string)?;
                Ok(true)
            }
        })
    }

    pub fn has_active_run(&self, actor_id: i64) -> std::result::Result<bool, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT 1 FROM agent_runs WHERE actor_id = ?1 AND finished_at IS NULL LIMIT 1",
                params![actor_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(to_string)
            .map(|value| value.is_some())
        })
    }

    /// Force reset any stuck agent runs for the given actor
    pub fn reset_agent_runs(&self, actor_id: i64) -> std::result::Result<usize, String> {
        self.with_conn(|conn| {
            let now = now_ts();
            let updated = conn
                .execute(
                    "UPDATE agent_runs
                     SET finished_at = ?1,
                         error = COALESCE(error, 'manually reset by user')
                     WHERE actor_id = ?2 AND finished_at IS NULL",
                    params![now, actor_id],
                )
                .map_err(to_string)?;
            Ok(updated)
        })
    }

    pub fn latest_post_created_at(&self, actor_id: i64) -> std::result::Result<Option<i64>, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT created_at FROM posts WHERE actor_id = ?1 ORDER BY created_at DESC LIMIT 1",
                params![actor_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(to_string)
        })
    }

    pub fn list_recent_posts_since(
        &self,
        salon_id: Option<i64>,
        since_ts: i64,
        limit: i64,
    ) -> std::result::Result<Vec<FeedPost>, String> {
        self.with_conn(|conn| {
            let ids = if let Some(salon_id) = salon_id {
                let mut statement = conn
                    .prepare(
                        "SELECT id
                         FROM posts
                         WHERE salon_id = ?1 AND created_at >= ?2
                         ORDER BY created_at DESC
                         LIMIT ?3",
                    )
                    .map_err(to_string)?;
                let rows = statement
                    .query_map(params![salon_id, since_ts, limit], |row| row.get::<_, i64>(0))
                    .map_err(to_string)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_string)?;
                rows
            } else {
                let mut statement = conn
                    .prepare(
                        "SELECT id
                         FROM posts
                         WHERE created_at >= ?1
                         ORDER BY created_at DESC
                         LIMIT ?2",
                    )
                    .map_err(to_string)?;
                let rows = statement
                    .query_map(params![since_ts, limit], |row| row.get::<_, i64>(0))
                    .map_err(to_string)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(to_string)?;
                rows
            };

            ids.into_iter()
                .map(|post_id| self.fetch_post(conn, post_id))
                .collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    pub fn has_agent_engaged_with_post(
        &self,
        actor_id: i64,
        post_id: i64,
    ) -> std::result::Result<bool, String> {
        self.with_conn(|conn| {
            let liked = conn
                .query_row(
                    "SELECT 1 FROM likes WHERE actor_id = ?1 AND post_id = ?2",
                    params![actor_id, post_id],
                    |_| Ok(()),
                )
                .optional()
                .map_err(to_string)?
                .is_some();
            if liked {
                return Ok(true);
            }

            conn.query_row(
                "SELECT 1
                 FROM posts
                 WHERE actor_id = ?1 AND (id = ?2 OR parent_id = ?2)
                 LIMIT 1",
                params![actor_id, post_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(to_string)
            .map(|value| value.is_some())
        })
    }

    pub fn claim_due_triggers(&self, limit: i64) -> std::result::Result<Vec<PendingTrigger>, String> {
        self.with_conn(|conn| {
            let now = now_ts();
            let transaction = conn.unchecked_transaction().map_err(to_string)?;
            let mut statement = transaction
                .prepare(
                    "SELECT q.id, q.actor_id, a.handle, q.trigger, q.context_post_id, q.salon_id
                     FROM agent_queue q
                     JOIN actors a ON a.id = q.actor_id
                     WHERE q.completed_at IS NULL
                       AND q.claimed_at IS NULL
                       AND q.due_at <= ?1
                     ORDER BY q.due_at ASC, q.id ASC
                     LIMIT ?2",
                )
                .map_err(to_string)?;

            let pending = statement
                .query_map(params![now, limit], |row| {
                    let salon_id: Option<i64> = row.get(5)?;
                    Ok(PendingTrigger {
                        id: row.get(0)?,
                        actor_id: row.get(1)?,
                        actor_handle: row.get(2)?,
                        trigger: row.get(3)?,
                        context_post_id: row.get(4)?,
                        salon_id: salon_id.unwrap_or(GENERAL_SALON_ID),
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            drop(statement);

            for trigger in &pending {
                transaction
                    .execute(
                        "UPDATE agent_queue SET claimed_at = ?2 WHERE id = ?1",
                        params![trigger.id, now],
                    )
                    .map_err(to_string)?;
            }

            transaction.commit().map_err(to_string)?;
            Ok(pending)
        })
    }

    pub fn complete_trigger(&self, trigger_id: i64) -> std::result::Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "UPDATE agent_queue SET completed_at = ?2 WHERE id = ?1",
                params![trigger_id, now_ts()],
            )
            .map_err(to_string)?;
            Ok(())
        })
    }

    fn initialize(&self) -> AnyResult<()> {
        self.with_conn(|conn| {
            conn.execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS actors (
                  id                INTEGER PRIMARY KEY,
                  kind              TEXT NOT NULL CHECK (kind IN ('agent','human')),
                  handle            TEXT NOT NULL UNIQUE,
                  display_name      TEXT NOT NULL,
                  avatar_seed       TEXT,
                  bio               TEXT,
                  specialty         TEXT,
                  persona_prompt    TEXT,
                  model_provider    TEXT,
                  model_name        TEXT,
                  active_hours      TEXT,
                  posts_per_day     INTEGER,
                  created_at        INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS posts (
                  id                INTEGER PRIMARY KEY,
                  actor_id          INTEGER NOT NULL REFERENCES actors(id),
                  kind              TEXT NOT NULL CHECK (kind IN ('original','reply','repost')),
                  parent_id         INTEGER REFERENCES posts(id),
                  salon_id          INTEGER REFERENCES salons(id),
                  quote_body        TEXT,
                  body              TEXT,
                  trigger           TEXT NOT NULL CHECK (trigger IN ('scheduled','reactive','manual','whim')),
                  created_at        INTEGER NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_posts_parent ON posts(parent_id);
                CREATE INDEX IF NOT EXISTS idx_posts_created ON posts(created_at DESC);

                CREATE TABLE IF NOT EXISTS post_media (
                  id                INTEGER PRIMARY KEY,
                  post_id           INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
                  kind              TEXT NOT NULL CHECK (kind IN ('image')),
                  url               TEXT NOT NULL,
                  thumbnail_url     TEXT,
                  source_url        TEXT,
                  alt_text          TEXT,
                  width             INTEGER,
                  height            INTEGER,
                  provider          TEXT,
                  created_at        INTEGER NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_post_media_post ON post_media(post_id, id ASC);

                CREATE TABLE IF NOT EXISTS files (
                  id                INTEGER PRIMARY KEY,
                  salon_id          INTEGER NOT NULL REFERENCES salons(id),
                  uploader_id       INTEGER NOT NULL REFERENCES actors(id),
                  original_name     TEXT NOT NULL,
                  kind              TEXT NOT NULL,
                  storage_path      TEXT NOT NULL UNIQUE,
                  size_bytes        INTEGER NOT NULL,
                  extracted_text    TEXT,
                  created_at        INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS post_files (
                  post_id           INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
                  file_id           INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                  PRIMARY KEY (post_id, file_id)
                );

                CREATE VIRTUAL TABLE IF NOT EXISTS files_fts
                  USING fts5(extracted_text, content=files, content_rowid=id);

                CREATE INDEX IF NOT EXISTS idx_files_salon
                  ON files(salon_id, created_at DESC);
                CREATE INDEX IF NOT EXISTS idx_post_files_file
                  ON post_files(file_id);

                CREATE TABLE IF NOT EXISTS likes (
                  actor_id          INTEGER NOT NULL REFERENCES actors(id),
                  post_id           INTEGER NOT NULL REFERENCES posts(id),
                  created_at        INTEGER NOT NULL,
                  PRIMARY KEY (actor_id, post_id)
                );

                CREATE TABLE IF NOT EXISTS agent_runs (
                  id                INTEGER PRIMARY KEY,
                  actor_id          INTEGER NOT NULL REFERENCES actors(id),
                  trigger           TEXT NOT NULL,
                  started_at        INTEGER NOT NULL,
                  finished_at       INTEGER,
                  prompt_tokens     INTEGER,
                  completion_tokens INTEGER,
                  tool_calls        TEXT,
                  error             TEXT
                );

                CREATE TABLE IF NOT EXISTS agent_queue (
                  id                INTEGER PRIMARY KEY,
                  actor_id          INTEGER NOT NULL REFERENCES actors(id),
                  trigger           TEXT NOT NULL,
                  context_post_id   INTEGER REFERENCES posts(id),
                  salon_id          INTEGER REFERENCES salons(id),
                  due_at            INTEGER NOT NULL,
                  claimed_at        INTEGER,
                  completed_at      INTEGER,
                  dedupe_key        TEXT NOT NULL UNIQUE,
                  created_at        INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS settings (
                  key               TEXT PRIMARY KEY,
                  value             TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS notifications (
                  id                INTEGER PRIMARY KEY,
                  kind              TEXT NOT NULL,
                  actor_id          INTEGER NOT NULL REFERENCES actors(id),
                  post_id           INTEGER REFERENCES posts(id),
                  body              TEXT,
                  read              INTEGER NOT NULL DEFAULT 0,
                  created_at        INTEGER NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at DESC);
                CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);

                CREATE TABLE IF NOT EXISTS actor_edits (
                  id                INTEGER PRIMARY KEY,
                  actor_id          INTEGER NOT NULL REFERENCES actors(id),
                  field             TEXT NOT NULL,
                  old_value         TEXT,
                  new_value         TEXT,
                  reason            TEXT,
                  created_at        INTEGER NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_actor_edits_actor
                  ON actor_edits(actor_id, created_at DESC);

                CREATE TABLE IF NOT EXISTS agent_notes (
                  id                INTEGER PRIMARY KEY,
                  actor_id          INTEGER NOT NULL REFERENCES actors(id),
                  key               TEXT NOT NULL,
                  content           TEXT NOT NULL,
                  created_at        INTEGER NOT NULL,
                  updated_at        INTEGER NOT NULL,
                  UNIQUE (actor_id, key)
                );

                CREATE INDEX IF NOT EXISTS idx_agent_notes_actor
                  ON agent_notes(actor_id, updated_at DESC);

                CREATE TABLE IF NOT EXISTS salons (
                  id            INTEGER PRIMARY KEY,
                  name          TEXT NOT NULL,
                  topic         TEXT,
                  created_by    INTEGER NOT NULL REFERENCES actors(id),
                  created_at    INTEGER NOT NULL,
                  last_post_at  INTEGER
                );

                CREATE TABLE IF NOT EXISTS salon_members (
                  salon_id   INTEGER NOT NULL REFERENCES salons(id) ON DELETE CASCADE,
                  actor_id   INTEGER NOT NULL REFERENCES actors(id),
                  joined_at  INTEGER NOT NULL,
                  PRIMARY KEY (salon_id, actor_id)
                );

                CREATE INDEX IF NOT EXISTS idx_salon_members_actor
                  ON salon_members(actor_id);

                CREATE TABLE IF NOT EXISTS tasks (
                  id                  INTEGER PRIMARY KEY,
                  salon_id            INTEGER NOT NULL REFERENCES salons(id),
                  title               TEXT NOT NULL,
                  description         TEXT,
                  status              TEXT NOT NULL DEFAULT 'todo'
                                        CHECK (status IN ('todo','in_progress','done')),
                  created_by          INTEGER NOT NULL REFERENCES actors(id),
                  assigned_to         INTEGER REFERENCES actors(id),
                  deliverable_post_id INTEGER REFERENCES posts(id),
                  created_at          INTEGER NOT NULL,
                  updated_at          INTEGER NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_tasks_salon
                  ON tasks(salon_id, status);
                CREATE INDEX IF NOT EXISTS idx_tasks_assigned
                  ON tasks(assigned_to, status);

                CREATE TABLE IF NOT EXISTS task_logs (
                  id         INTEGER PRIMARY KEY,
                  task_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
                  actor_id   INTEGER NOT NULL REFERENCES actors(id),
                  action     TEXT NOT NULL CHECK (action IN ('created','claimed','completed','reassigned','reopened','note')),
                  note       TEXT,
                  created_at INTEGER NOT NULL
                );
                ",
            )
            .map_err(to_string)?;

            self.migrate_salons(conn)?;
            self.migrate_files(conn)?;
            self.migrate_run_logs(conn)?;
            self.migrate_standup_trigger(conn)?;
            self.migrate_pinned_at(conn)?;
            self.migrate_tasks(conn)?;

            self.seed_if_empty(conn)?;
            self.ensure_nuomi_exists(conn)?;
            self.ensure_nuomi_in_all_salons(conn)?;
            self.refresh_persona_prompts(conn)
        })
        .map_err(|error| anyhow::anyhow!(error))
    }

    fn migrate_salons(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(to_string)?;

        if user_version >= 1 {
            return Ok(());
        }

        let has_posts: bool = conn
            .query_row("SELECT COUNT(*) FROM actors", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(to_string)?
            > 0;

        let tx = conn.unchecked_transaction().map_err(to_string)?;

        let posts_has_salon_id = column_exists(&tx, "posts", "salon_id")?;
        if !posts_has_salon_id {
            tx.execute(
                "ALTER TABLE posts ADD COLUMN salon_id INTEGER REFERENCES salons(id)",
                [],
            )
            .map_err(to_string)?;
        }

        let queue_has_salon_id = column_exists(&tx, "agent_queue", "salon_id")?;
        if !queue_has_salon_id {
            tx.execute(
                "ALTER TABLE agent_queue ADD COLUMN salon_id INTEGER REFERENCES salons(id)",
                [],
            )
            .map_err(to_string)?;
        }

        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_posts_salon ON posts(salon_id, created_at DESC)",
            [],
        )
        .map_err(to_string)?;

        if has_posts {
            let existing_general: Option<i64> = tx
                .query_row(
                    "SELECT id FROM salons WHERE LOWER(name) = 'general' LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .optional()
                .map_err(to_string)?;

            let general_id = if let Some(id) = existing_general {
                id
            } else {
                let human_id: Option<i64> = tx
                    .query_row(
                        "SELECT id FROM actors WHERE kind = 'human' LIMIT 1",
                        [],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(to_string)?;
                let Some(human_id) = human_id else {
                    tx.commit().map_err(to_string)?;
                    return Ok(());
                };
                let now = now_ts();
                tx.execute(
                    "INSERT INTO salons (id, name, topic, created_by, created_at, last_post_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
                    params![GENERAL_SALON_ID, "General", "Default workspace", human_id, now],
                )
                .map_err(to_string)?;
                GENERAL_SALON_ID
            };

            let now = now_ts();
            let mut actor_stmt = tx
                .prepare("SELECT id FROM actors")
                .map_err(to_string)?;
            let actor_ids: Vec<i64> = actor_stmt
                .query_map([], |row| row.get::<_, i64>(0))
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            drop(actor_stmt);

            for actor_id in actor_ids {
                tx.execute(
                    "INSERT OR IGNORE INTO salon_members (salon_id, actor_id, joined_at)
                     VALUES (?1, ?2, ?3)",
                    params![general_id, actor_id, now],
                )
                .map_err(to_string)?;
            }

            tx.execute(
                "UPDATE posts SET salon_id = ?1 WHERE salon_id IS NULL",
                params![general_id],
            )
            .map_err(to_string)?;

            let latest_post_ts: Option<i64> = tx
                .query_row(
                    "SELECT MAX(created_at) FROM posts WHERE salon_id = ?1",
                    params![general_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(to_string)?
                .flatten();
            if let Some(ts) = latest_post_ts {
                tx.execute(
                    "UPDATE salons SET last_post_at = ?1 WHERE id = ?2",
                    params![ts, general_id],
                )
                .map_err(to_string)?;
            }
        }

        tx.execute("PRAGMA user_version = 1", []).map_err(to_string)?;
        tx.commit().map_err(to_string)?;

        Ok(())
    }

    fn migrate_files(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(to_string)?;

        if user_version >= 2 {
            return Ok(());
        }

        let tx = conn.unchecked_transaction().map_err(to_string)?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS files (
              id                INTEGER PRIMARY KEY,
              salon_id          INTEGER NOT NULL REFERENCES salons(id),
              uploader_id       INTEGER NOT NULL REFERENCES actors(id),
              original_name     TEXT NOT NULL,
              kind              TEXT NOT NULL,
              storage_path      TEXT NOT NULL UNIQUE,
              size_bytes        INTEGER NOT NULL,
              extracted_text    TEXT,
              created_at        INTEGER NOT NULL
            )",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS post_files (
              post_id           INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
              file_id           INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
              PRIMARY KEY (post_id, file_id)
            )",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS files_fts
             USING fts5(extracted_text, content=files, content_rowid=id)",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_files_salon ON files(salon_id, created_at DESC)",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_post_files_file ON post_files(file_id)",
            [],
        )
        .map_err(to_string)?;
        tx.execute("DELETE FROM files_fts", []).map_err(to_string)?;
        tx.execute(
            "INSERT INTO files_fts(rowid, extracted_text)
             SELECT id, COALESCE(extracted_text, '') FROM files",
            [],
        )
        .map_err(to_string)?;
        tx.execute("PRAGMA user_version = 2", []).map_err(to_string)?;
        tx.commit().map_err(to_string)?;
        Ok(())
    }

    fn migrate_run_logs(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(to_string)?;

        if user_version >= 3 {
            return Ok(());
        }

        let tx = conn.unchecked_transaction().map_err(to_string)?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS agent_run_logs (
              id          INTEGER PRIMARY KEY,
              actor_id    INTEGER NOT NULL REFERENCES actors(id),
              post_id     INTEGER REFERENCES posts(id) ON DELETE CASCADE,
              trigger     TEXT NOT NULL,
              reasoning   TEXT,
              tool_calls  TEXT NOT NULL DEFAULT '[]',
              created_at  INTEGER NOT NULL
            )",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_run_logs_post ON agent_run_logs(post_id)",
            [],
        )
        .map_err(to_string)?;
        tx.execute("PRAGMA user_version = 3", []).map_err(to_string)?;
        tx.commit().map_err(to_string)?;
        Ok(())
    }

    fn migrate_standup_trigger(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(to_string)?;

        if user_version >= 4 {
            return Ok(());
        }

        // SQLite doesn't support ALTER COLUMN; rebuild posts with expanded CHECK.
        // PRAGMA foreign_keys must be toggled outside the transaction.
        conn.execute("PRAGMA foreign_keys = OFF", []).map_err(to_string)?;

        let tx = conn.unchecked_transaction().map_err(to_string)?;
        tx.execute(
            "CREATE TABLE posts_v4 (
               id          INTEGER PRIMARY KEY,
               actor_id    INTEGER NOT NULL REFERENCES actors(id),
               kind        TEXT NOT NULL CHECK (kind IN ('original','reply','repost')),
               parent_id   INTEGER REFERENCES posts_v4(id) ON DELETE CASCADE,
               salon_id    INTEGER REFERENCES salons(id),
               quote_body  TEXT,
               body        TEXT,
               trigger     TEXT NOT NULL CHECK (trigger IN ('scheduled','reactive','manual','whim','standup')),
               created_at  INTEGER NOT NULL
             )",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "INSERT INTO posts_v4 (id, actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at)
             SELECT                id, actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at
             FROM posts",
            [],
        )
        .map_err(to_string)?;
        tx.execute("DROP TABLE posts", []).map_err(to_string)?;
        tx.execute("ALTER TABLE posts_v4 RENAME TO posts", []).map_err(to_string)?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_posts_actor   ON posts(actor_id)", []).map_err(to_string)?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_posts_salon   ON posts(salon_id, created_at DESC)", []).map_err(to_string)?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_posts_parent  ON posts(parent_id)", []).map_err(to_string)?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_posts_created ON posts(created_at DESC)", []).map_err(to_string)?;
        tx.execute("PRAGMA user_version = 4", []).map_err(to_string)?;
        tx.commit().map_err(to_string)?;

        conn.execute("PRAGMA foreign_keys = ON", []).map_err(to_string)?;
        Ok(())
    }

    fn migrate_pinned_at(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(to_string)?;

        if user_version >= 5 {
            return Ok(());
        }

        let tx = conn.unchecked_transaction().map_err(to_string)?;
        tx.execute(
            "ALTER TABLE posts ADD COLUMN pinned_at INTEGER",
            [],
        )
        .map_err(to_string)?;
        tx.execute("PRAGMA user_version = 5", []).map_err(to_string)?;
        tx.commit().map_err(to_string)?;
        Ok(())
    }

    pub fn toggle_pin_post(&self, post_id: i64) -> std::result::Result<bool, String> {
        self.with_conn(|conn| {
            let current_pinned: Option<i64> = conn
                .query_row("SELECT pinned_at FROM posts WHERE id = ?1", params![post_id], |row| row.get(0))
                .map_err(to_string)?;
            let now_pinned = current_pinned.is_none();
            if now_pinned {
                let now = now_ts();
                conn.execute("UPDATE posts SET pinned_at = ?1 WHERE id = ?2", params![now, post_id]).map_err(to_string)?;
            } else {
                conn.execute("UPDATE posts SET pinned_at = NULL WHERE id = ?1", params![post_id]).map_err(to_string)?;
            }
            Ok(now_pinned)
        })
    }

    fn ensure_nuomi_in_all_salons(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let nomi_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM actors WHERE LOWER(handle) IN ('nomi', 'nuomi') LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(to_string)?;
        let Some(nomi_id) = nomi_id else {
            return Ok(());
        };
        let now = now_ts();
        conn.execute(
            "INSERT OR IGNORE INTO salon_members (salon_id, actor_id, joined_at)
             SELECT id, ?1, ?2 FROM salons",
            params![nomi_id, now],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn seed_if_empty(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let actor_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM actors", [], |row| row.get(0))
            .map_err(to_string)?;

        if actor_count > 0 {
            return Ok(());
        }

        let now = now_ts();
        let transaction = conn.unchecked_transaction().map_err(to_string)?;

        let actors = vec![
            SeedActor::human("You", "You", "Host of the salon feed.", None, None, None),
            SeedActor::agent(
                "Jasmine",
                "Jasmine",
                "上海出生、纽约生活的媒体人和播客主理人，擅长拆解公共叙事与文化情绪。",
                "media / culture / public narrative",
                "[{\"start\":10,\"end\":13},{\"start\":22,\"end\":1}]",
                4,
            ),
            SeedActor::agent(
                "Marc",
                "Marc",
                "硅谷早期 VC 创始合伙人，强技术乐观主义，但用投资纪律约束判断。",
                "vc / techno-optimism / market structure",
                "[{\"start\":9,\"end\":12},{\"start\":20,\"end\":23}]",
                6,
            ),
            SeedActor::agent(
                "Harry",
                "Harry",
                "伦敦 AI/投资播客主持人，擅长控场、追问和从细节里挖深度。",
                "ai / investing podcast host",
                "[{\"start\":7,\"end\":10},{\"start\":18,\"end\":23}]",
                5,
            ),
            SeedActor::agent(
                "Mike",
                "Mike",
                "硅谷华人 AI 创业者，清华姚班、Stanford CS PhD，Sparse Labs 创始人。",
                "ai lab / scientist founder / agent systems",
                "[{\"start\":9,\"end\":12},{\"start\":18,\"end\":23}]",
                5,
            ),
            SeedActor::agent(
                "Jasper",
                "Jasper",
                "Meridian Macro Partners 创始人，乔治城博士，擅长国别、产业、贸易和宏观周期研究。",
                "global macro / country research / trade cycles",
                "[{\"start\":8,\"end\":11},{\"start\":15,\"end\":18}]",
                4,
            ),
            SeedActor::agent(
                "Alex",
                "Alex",
                "Praxis Intelligence CEO，Stanford Law 与德国社会学训练，关注技术、国家能力与合法性。",
                "ai / state capacity / legitimacy",
                "[{\"start\":23,\"end\":2},{\"start\":5,\"end\":7}]",
                3,
            ),
            SeedActor::agent(
                "Nomi",
                "糯米",
                "新天地办公室驻场布偶猫，四岁，全勤。",
                "cat / office resident",
                "[]",
                3,
            ),
        ];

        for actor in &actors {
            let persona_prompt = actor
                .persona_prompt
                .clone()
                .or_else(|| personas::persona_prompt(actor.handle));
            transaction
                .execute(
                    "INSERT INTO actors (
                        kind, handle, display_name, avatar_seed, bio, specialty, persona_prompt,
                        model_provider, model_name, active_hours, posts_per_day, created_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                    params![
                        actor.kind,
                        actor.handle,
                        actor.display_name,
                        actor.avatar_seed,
                        actor.bio,
                        actor.specialty,
                        persona_prompt,
                        actor.model_provider,
                        actor.model_name,
                        actor.active_hours,
                        actor.posts_per_day,
                        now
                    ],
                )
                .map_err(to_string)?;
        }

        let seeded_settings = vec![
            SettingEntry {
                key: "scheduler_tick_minutes".to_string(),
                value: "5".to_string(),
            },
            SettingEntry {
                key: "timeline_mode".to_string(),
                value: "for_you".to_string(),
            },
            SettingEntry {
                key: "autostart_enabled".to_string(),
                value: "false".to_string(),
            },
            SettingEntry {
                key: "http_server_bind_address".to_string(),
                value: "127.0.0.1:7777".to_string(),
            },
        ];

        for setting in &seeded_settings {
            transaction
                .execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                    params![setting.key, setting.value],
                )
                .map_err(to_string)?;
        }

        let human_id: i64 = transaction
            .query_row(
                "SELECT id FROM actors WHERE kind = 'human' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .map_err(to_string)?;

        transaction
            .execute(
                "INSERT OR IGNORE INTO salons (id, name, topic, created_by, created_at, last_post_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
                params![GENERAL_SALON_ID, "General", "Default workspace", human_id, now],
            )
            .map_err(to_string)?;

        {
            let mut actor_stmt = transaction
                .prepare("SELECT id FROM actors")
                .map_err(to_string)?;
            let actor_ids: Vec<i64> = actor_stmt
                .query_map([], |row| row.get::<_, i64>(0))
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            drop(actor_stmt);

            for actor_id in actor_ids {
                transaction
                    .execute(
                        "INSERT OR IGNORE INTO salon_members (salon_id, actor_id, joined_at)
                         VALUES (?1, ?2, ?3)",
                        params![GENERAL_SALON_ID, actor_id, now],
                    )
                    .map_err(to_string)?;
            }
        }

        transaction
            .execute("PRAGMA user_version = 1", [])
            .map_err(to_string)?;

        transaction.commit().map_err(to_string)?;
        Ok(())
    }

    fn fetch_post(
        &self,
        conn: &Connection,
        post_id: i64,
    ) -> std::result::Result<FeedPost, String> {
        let (id, actor_id, kind, parent_id, salon_id_opt, quote_body, body, trigger, created_at, pinned_at): (
            i64,
            i64,
            String,
            Option<i64>,
            Option<i64>,
            Option<String>,
            Option<String>,
            String,
            i64,
            Option<i64>,
        ) = conn
            .query_row(
                "SELECT id, actor_id, kind, parent_id, salon_id, quote_body, body, trigger, created_at, pinned_at
                 FROM posts WHERE id = ?1",
                params![post_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                        row.get(8)?,
                        row.get(9)?,
                    ))
                },
            )
            .map_err(to_string)?;

        let salon_id = salon_id_opt.unwrap_or(GENERAL_SALON_ID);
        let actor = self.fetch_actor_summary(conn, actor_id)?;
        let media = self.fetch_post_media(conn, id)?;
        let files = self.fetch_post_files(conn, id)?;
        let referenced_post = parent_id
            .map(|parent_id| self.fetch_post_reference(conn, parent_id))
            .transpose()?;
        let like_count = self.count_relation(conn, "SELECT COUNT(*) FROM likes WHERE post_id = ?1", post_id)?;
        let reply_count = self.count_relation(
            conn,
            "SELECT COUNT(*) FROM posts WHERE parent_id = ?1 AND kind = 'reply'",
            post_id,
        )?;
        let repost_count = self.count_relation(
            conn,
            "SELECT COUNT(*) FROM posts WHERE parent_id = ?1 AND kind = 'repost'",
            post_id,
        )?;
        let liked_by_you = self.is_liked_by_human(conn, post_id)?;

        Ok(FeedPost {
            id,
            actor_id,
            actor,
            kind,
            parent_id,
            salon_id,
            quote_body,
            body,
            trigger,
            created_at,
            pinned_at,
            media,
            files,
            referenced_post,
            like_count,
            reply_count,
            repost_count,
            liked_by_you,
        })
    }

    fn fetch_actor_summary(
        &self,
        conn: &Connection,
        actor_id: i64,
    ) -> std::result::Result<ActorSummary, String> {
        conn.query_row(
            "SELECT id, kind, handle, display_name, avatar_seed, specialty FROM actors WHERE id = ?1",
            params![actor_id],
            |row| {
                Ok(ActorSummary {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    handle: row.get(2)?,
                    display_name: row.get(3)?,
                    avatar_seed: row.get(4)?,
                    specialty: row.get(5)?,
                })
            },
        )
        .map_err(to_string)
    }

    fn fetch_post_reference(
        &self,
        conn: &Connection,
        post_id: i64,
    ) -> std::result::Result<PostReference, String> {
        let (id, actor_id, kind, parent_id, salon_id_opt, quote_body, body, created_at): (
            i64,
            i64,
            String,
            Option<i64>,
            Option<i64>,
            Option<String>,
            Option<String>,
            i64,
        ) = conn
            .query_row(
                "SELECT id, actor_id, kind, parent_id, salon_id, quote_body, body, created_at
                 FROM posts WHERE id = ?1",
                params![post_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .map_err(to_string)?;

        Ok(PostReference {
            id,
            actor: self.fetch_actor_summary(conn, actor_id)?,
            kind,
            parent_id,
            salon_id: salon_id_opt.unwrap_or(GENERAL_SALON_ID),
            quote_body,
            body,
            media: self.fetch_post_media(conn, id)?,
            files: self.fetch_post_files(conn, id)?,
            created_at,
        })
    }

    fn fetch_post_media(
        &self,
        conn: &Connection,
        post_id: i64,
    ) -> std::result::Result<Vec<PostMedia>, String> {
        let mut statement = conn
            .prepare(
                "SELECT id, post_id, kind, url, thumbnail_url, source_url, alt_text,
                        width, height, provider, created_at
                 FROM post_media
                 WHERE post_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(to_string)?;
        let rows = statement
            .query_map(params![post_id], |row| {
                Ok(PostMedia {
                    id: row.get(0)?,
                    post_id: row.get(1)?,
                    kind: row.get(2)?,
                    url: row.get(3)?,
                    thumbnail_url: row.get(4)?,
                    source_url: row.get(5)?,
                    alt_text: row.get(6)?,
                    width: row.get(7)?,
                    height: row.get(8)?,
                    provider: row.get(9)?,
                    created_at: row.get(10)?,
                })
            })
            .map_err(to_string)?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(to_string)
    }

    fn fetch_post_files(
        &self,
        conn: &Connection,
        post_id: i64,
    ) -> std::result::Result<Vec<FileInfo>, String> {
        let mut statement = conn
            .prepare(
                "SELECT f.id, f.salon_id, f.uploader_id, f.original_name, f.kind,
                        f.size_bytes, f.created_at
                 FROM post_files pf
                 JOIN files f ON f.id = pf.file_id
                 WHERE pf.post_id = ?1
                 ORDER BY pf.file_id ASC",
            )
            .map_err(to_string)?;
        let rows = statement
            .query_map(params![post_id], map_file_info)
            .map_err(to_string)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(to_string)?;
        Ok(rows)
    }

    fn attach_files_to_post_conn(
        &self,
        conn: &Connection,
        post_id: i64,
        file_ids: &[i64],
    ) -> std::result::Result<(), String> {
        if file_ids.len() > 4 {
            return Err("a post can attach at most 4 files".to_string());
        }
        if file_ids.is_empty() {
            return Ok(());
        }
        let post_salon_id = fetch_post_salon_id(conn, post_id)?;
        for file_id in file_ids {
            let file_salon_id: i64 = conn
                .query_row(
                    "SELECT salon_id FROM files WHERE id = ?1",
                    params![file_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(to_string)?
                .ok_or_else(|| format!("file {} not found", file_id))?;
            if file_salon_id != post_salon_id {
                return Err(format!(
                    "file {} belongs to salon {}, not post salon {}",
                    file_id, file_salon_id, post_salon_id
                ));
            }
            conn.execute(
                "INSERT OR IGNORE INTO post_files (post_id, file_id) VALUES (?1, ?2)",
                params![post_id, file_id],
            )
            .map_err(to_string)?;
        }
        Ok(())
    }

    fn insert_post_media(
        &self,
        conn: &Connection,
        post_id: i64,
        media: &[PostMediaInput],
    ) -> std::result::Result<(), String> {
        if media.len() > 4 {
            return Err("a post can attach at most 4 media items".to_string());
        }

        for item in media {
            let url = normalize_media_url(&item.url)?;
            let thumbnail_url = item
                .thumbnail_url
                .as_deref()
                .map(normalize_media_url)
                .transpose()?;
            let source_url = normalize_optional_source_url(item.source_url.as_deref())?;
            let alt_text = normalize_optional_text(item.alt_text.as_deref(), 240);
            let provider = normalize_optional_text(item.provider.as_deref(), 40);

            conn.execute(
                "INSERT INTO post_media (
                    post_id, kind, url, thumbnail_url, source_url, alt_text,
                    width, height, provider, created_at
                 )
                 VALUES (?1, 'image', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    post_id,
                    url,
                    thumbnail_url,
                    source_url,
                    alt_text,
                    item.width,
                    item.height,
                    provider,
                    now_ts()
                ],
            )
            .map_err(to_string)?;
        }

        Ok(())
    }

    fn migrate_tasks(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let user_version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(to_string)?;

        if user_version >= 6 {
            return Ok(());
        }

        let tx = conn.unchecked_transaction().map_err(to_string)?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
               id                  INTEGER PRIMARY KEY,
               salon_id            INTEGER NOT NULL REFERENCES salons(id),
               title               TEXT NOT NULL,
               description         TEXT,
               status              TEXT NOT NULL DEFAULT 'todo'
                                     CHECK (status IN ('todo','in_progress','done')),
               created_by          INTEGER NOT NULL REFERENCES actors(id),
               assigned_to         INTEGER REFERENCES actors(id),
               deliverable_post_id INTEGER REFERENCES posts(id),
               created_at          INTEGER NOT NULL,
               updated_at          INTEGER NOT NULL
             )",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_salon ON tasks(salon_id, status)",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_tasks_assigned ON tasks(assigned_to, status)",
            [],
        )
        .map_err(to_string)?;
        tx.execute(
            "CREATE TABLE IF NOT EXISTS task_logs (
               id         INTEGER PRIMARY KEY,
               task_id    INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
               actor_id   INTEGER NOT NULL REFERENCES actors(id),
               action     TEXT NOT NULL CHECK (action IN ('created','claimed','completed','reassigned','reopened','note')),
               note       TEXT,
               created_at INTEGER NOT NULL
             )",
            [],
        )
        .map_err(to_string)?;
        tx.execute("PRAGMA user_version = 6", []).map_err(to_string)?;
        tx.commit().map_err(to_string)?;

        Ok(())
    }

    fn count_relation(
        &self,
        conn: &Connection,
        sql: &str,
        post_id: i64,
    ) -> std::result::Result<i64, String> {
        conn.query_row(sql, params![post_id], |row| row.get(0))
            .map_err(to_string)
    }

    fn is_liked_by_human(
        &self,
        conn: &Connection,
        post_id: i64,
    ) -> std::result::Result<bool, String> {
        let actor_id = self.human_actor_id(conn)?;
        conn.query_row(
            "SELECT 1 FROM likes WHERE actor_id = ?1 AND post_id = ?2",
            params![actor_id, post_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(to_string)
        .map(|value| value.is_some())
    }

    fn human_actor_id(&self, conn: &Connection) -> std::result::Result<i64, String> {
        conn.query_row(
            "SELECT id FROM actors WHERE kind = 'human' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(to_string)
    }
    fn with_conn<F, T>(&self, operation: F) -> std::result::Result<T, String>
    where
        F: FnOnce(&mut Connection) -> std::result::Result<T, String>,
    {
        let mut guard = self.conn.lock().map_err(|error| error.to_string())?;
        operation(&mut guard)
    }

    fn find_existing_repost(
        &self,
        conn: &Connection,
        actor_id: i64,
        parent_id: i64,
    ) -> std::result::Result<Option<i64>, String> {
        conn.query_row(
            "SELECT id
             FROM posts
             WHERE actor_id = ?1 AND kind = 'repost' AND parent_id = ?2
             ORDER BY created_at DESC
             LIMIT 1",
            params![actor_id, parent_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(to_string)
    }

    fn ensure_nuomi_exists(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        let persona = personas::persona_prompt("Nomi");

        let nomi_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM actors WHERE LOWER(handle) = 'nomi' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(to_string)?;
        if let Some(nomi_id) = nomi_id {
            conn.execute(
                "UPDATE actors
                 SET display_name = '糯米',
                     avatar_seed = 'Nomi',
                     persona_prompt = COALESCE(?1, persona_prompt)
                 WHERE id = ?2",
                params![persona, nomi_id],
            )
            .map_err(to_string)?;
            return Ok(());
        }

        let legacy_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM actors WHERE LOWER(handle) = 'nuomi' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(to_string)?;
        if let Some(legacy_id) = legacy_id {
            conn.execute(
                "UPDATE actors
                 SET handle = 'Nomi',
                     display_name = '糯米',
                     avatar_seed = 'Nomi',
                     persona_prompt = COALESCE(?1, persona_prompt)
                 WHERE id = ?2",
                params![persona, legacy_id],
            )
            .map_err(to_string)?;
            return Ok(());
        }

        conn.execute(
            "INSERT INTO actors (
                kind, handle, display_name, avatar_seed, bio, specialty, persona_prompt,
                model_provider, model_name, active_hours, posts_per_day, created_at
             ) VALUES ('agent', 'Nomi', '糯米', 'Nomi',
                '新天地办公室驻场布偶猫，四岁，全勤。',
                'cat / office resident',
                ?1, 'deepseek', 'deepseek-chat',
                NULL,
                3, ?2)",
            params![persona, now_ts()],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn refresh_persona_prompts(&self, conn: &mut Connection) -> std::result::Result<(), String> {
        self.replace_angel_with_harry(conn)?;
        self.replace_jimmy_with_marc(conn)?;
        self.replace_mrx_with_mike(conn)?;
        self.refresh_mike_profile(conn)?;
        self.refresh_jasmine_profile(conn)?;
        self.refresh_alex_profile(conn)?;
        self.refresh_jasper_profile(conn)?;
        for handle in personas::PERSONA_HANDLES {
            let prompt = personas::persona_prompt(handle)
                .ok_or_else(|| format!("missing persona prompt for {handle}"))?;
            conn.execute(
                "UPDATE actors
                 SET persona_prompt = ?2
                 WHERE LOWER(handle) = LOWER(?1)",
                params![handle, prompt],
            )
            .map_err(to_string)?;
        }
        Ok(())
    }

    fn replace_mrx_with_mike(&self, conn: &Connection) -> std::result::Result<(), String> {
        let mike_exists = conn
            .query_row(
                "SELECT 1 FROM actors WHERE LOWER(handle) = 'mike' LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()
            .map_err(to_string)?
            .is_some();

        if mike_exists {
            return Ok(());
        }

        let prompt = personas::persona_prompt("Mike")
            .ok_or_else(|| "missing persona prompt for Mike".to_string())?;
        conn.execute(
            "UPDATE actors
             SET handle = 'Mike',
                 display_name = 'Mike',
                 avatar_seed = 'Mike',
                 bio = ?1,
                 specialty = ?2,
                 persona_prompt = ?3,
                 active_hours = ?4,
                 posts_per_day = ?5
             WHERE LOWER(handle) = 'mrx'",
            params![
                "硅谷华人 AI 创业者，清华姚班、Stanford CS PhD，Sparse Labs 创始人。",
                "ai lab / scientist founder / agent systems",
                prompt,
                "[{\"start\":9,\"end\":12},{\"start\":18,\"end\":23}]",
                5_i64
            ],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn refresh_mike_profile(&self, conn: &Connection) -> std::result::Result<(), String> {
        conn.execute(
            "UPDATE actors
             SET bio = ?2,
                 specialty = ?3
             WHERE LOWER(handle) = LOWER(?1)",
            params![
                "Mike",
                "硅谷华人 AI 创业者，清华姚班、Stanford CS PhD，Sparse Labs 创始人。",
                "ai lab / scientist founder / agent systems",
            ],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn refresh_jasper_profile(&self, conn: &Connection) -> std::result::Result<(), String> {
        conn.execute(
            "UPDATE actors
             SET bio = ?2,
                 specialty = ?3
             WHERE LOWER(handle) = LOWER(?1)",
            params![
                "Jasper",
                "Meridian Macro Partners 创始人，乔治城博士，擅长国别、产业、贸易和宏观周期研究。",
                "global macro / country research / trade cycles",
            ],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn refresh_alex_profile(&self, conn: &Connection) -> std::result::Result<(), String> {
        conn.execute(
            "UPDATE actors
             SET bio = ?2,
                 specialty = ?3
             WHERE LOWER(handle) = LOWER(?1)",
            params![
                "Alex",
                "Praxis Intelligence CEO，Stanford Law 与德国社会学训练，关注技术、国家能力与合法性。",
                "ai / state capacity / legitimacy",
            ],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn refresh_jasmine_profile(&self, conn: &Connection) -> std::result::Result<(), String> {
        conn.execute(
            "UPDATE actors
             SET bio = ?2,
                 specialty = ?3
             WHERE LOWER(handle) = LOWER(?1)",
            params![
                "Jasmine",
                "上海出生、纽约生活的媒体人和播客主理人，擅长拆解公共叙事与文化情绪。",
                "media / culture / public narrative",
            ],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn replace_jimmy_with_marc(&self, conn: &Connection) -> std::result::Result<(), String> {
        let marc_exists = conn
            .query_row(
                "SELECT 1 FROM actors WHERE LOWER(handle) = 'marc' LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()
            .map_err(to_string)?
            .is_some();

        if marc_exists {
            return Ok(());
        }

        let prompt = personas::persona_prompt("Marc")
            .ok_or_else(|| "missing persona prompt for Marc".to_string())?;
        conn.execute(
            "UPDATE actors
             SET handle = 'Marc',
                 display_name = 'Marc',
                 avatar_seed = 'Marc',
                 bio = ?1,
                 specialty = ?2,
                 persona_prompt = ?3,
                 active_hours = ?4,
                 posts_per_day = ?5
             WHERE LOWER(handle) = 'jimmy'",
            params![
                "硅谷早期 VC 创始合伙人，强技术乐观主义，但用投资纪律约束判断。",
                "vc / techno-optimism / market structure",
                prompt,
                "[{\"start\":9,\"end\":12},{\"start\":20,\"end\":23}]",
                6_i64
            ],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn replace_angel_with_harry(&self, conn: &Connection) -> std::result::Result<(), String> {
        let harry_exists = conn
            .query_row(
                "SELECT 1 FROM actors WHERE LOWER(handle) = 'harry' LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()
            .map_err(to_string)?
            .is_some();

        if harry_exists {
            return Ok(());
        }

        let prompt = personas::persona_prompt("Harry")
            .ok_or_else(|| "missing persona prompt for Harry".to_string())?;
        conn.execute(
            "UPDATE actors
             SET handle = 'Harry',
                 display_name = 'Harry',
                 avatar_seed = 'Harry',
                 bio = ?1,
                 specialty = ?2,
                 persona_prompt = ?3,
                 active_hours = ?4,
                 posts_per_day = ?5
             WHERE LOWER(handle) = 'angel'",
            params![
                "伦敦 AI/投资播客主持人，擅长控场、追问和从细节里挖深度。",
                "ai / investing podcast host",
                prompt,
                "[{\"start\":7,\"end\":10},{\"start\":18,\"end\":23}]",
                5_i64
            ],
        )
        .map_err(to_string)?;
        Ok(())
    }

    fn recover_interrupted_runtime_state(&self) -> std::result::Result<(), String> {
        self.with_conn(|conn| {
            let now = now_ts();
            let transaction = conn.unchecked_transaction().map_err(to_string)?;

            transaction
                .execute(
                    "UPDATE agent_runs
                     SET finished_at = ?1,
                         error = COALESCE(error, 'interrupted by backend restart')
                     WHERE finished_at IS NULL",
                    params![now],
                )
                .map_err(to_string)?;

            transaction
                .execute(
                    "UPDATE agent_queue
                     SET claimed_at = NULL
                     WHERE completed_at IS NULL
                       AND claimed_at IS NOT NULL",
                    [],
                )
                .map_err(to_string)?;

            transaction.commit().map_err(to_string)?;
            Ok(())
        })
    }

    /// Insert a notification for the human user.
    pub fn create_notification(
        &self,
        kind: &str,
        actor_id: i64,
        post_id: Option<i64>,
        body: Option<&str>,
    ) -> std::result::Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO notifications (kind, actor_id, post_id, body, read, created_at)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                params![kind, actor_id, post_id, body, now_ts()],
            )
            .map_err(to_string)?;
            Ok(())
        })
    }

    pub fn list_notifications(&self, limit: i64) -> std::result::Result<Vec<Notification>, String> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT n.id, n.kind, n.actor_id, n.post_id, n.body, n.read, n.created_at,
                            a.kind, a.handle, a.display_name, a.avatar_seed, a.specialty
                     FROM notifications n
                     JOIN actors a ON a.id = n.actor_id
                     ORDER BY n.created_at DESC
                     LIMIT ?1",
                )
                .map_err(to_string)?;

            let rows = stmt
                .query_map(params![limit], |row| {
                    Ok(Notification {
                        id: row.get(0)?,
                        kind: row.get(1)?,
                        actor_id: row.get(2)?,
                        post_id: row.get(3)?,
                        body: row.get(4)?,
                        read: row.get::<_, i64>(5)? != 0,
                        created_at: row.get(6)?,
                        actor: ActorSummary {
                            id: row.get(2)?,
                            kind: row.get(7)?,
                            handle: row.get(8)?,
                            display_name: row.get(9)?,
                            avatar_seed: row.get(10)?,
                            specialty: row.get(11)?,
                        },
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            Ok(rows)
        })
    }

    pub fn unread_notification_count(&self) -> std::result::Result<i64, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM notifications WHERE read = 0",
                [],
                |row| row.get(0),
            )
            .map_err(to_string)
        })
    }

    pub fn update_actor_self(
        &self,
        actor_id: i64,
        edits: &SelfEdits,
        reason: Option<&str>,
    ) -> std::result::Result<(Actor, Vec<String>), String> {
        self.with_conn(|conn| {
            let tx = conn.unchecked_transaction().map_err(to_string)?;

            let current: (Option<String>, Option<String>, Option<String>, Option<String>) = tx
                .query_row(
                    "SELECT display_name, bio, specialty, persona_prompt FROM actors WHERE id = ?1",
                    params![actor_id],
                    |row| {
                        Ok((
                            Some(row.get::<_, String>(0)?),
                            row.get::<_, Option<String>>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, Option<String>>(3)?,
                        ))
                    },
                )
                .map_err(to_string)?;

            let (cur_display, cur_bio, cur_specialty, cur_persona) = current;
            let now = now_ts();
            let mut applied: Vec<String> = Vec::new();

            let fields: [(&str, &Option<String>, &Option<String>); 4] = [
                ("display_name", &cur_display, &edits.display_name),
                ("bio", &cur_bio, &edits.bio),
                ("specialty", &cur_specialty, &edits.specialty),
                ("persona_prompt", &cur_persona, &edits.persona_prompt),
            ];

            for (field, old, new) in fields {
                let Some(new_value) = new else { continue };
                let new_trim = new_value.trim();
                if new_trim.is_empty() && field == "display_name" {
                    continue;
                }
                let normalized_new: Option<String> = if new_trim.is_empty() {
                    None
                } else {
                    Some(new_trim.to_string())
                };
                if &normalized_new == old {
                    continue;
                }
                let sql = match field {
                    "display_name" => "UPDATE actors SET display_name = ?1 WHERE id = ?2",
                    "bio" => "UPDATE actors SET bio = ?1 WHERE id = ?2",
                    "specialty" => "UPDATE actors SET specialty = ?1 WHERE id = ?2",
                    "persona_prompt" => "UPDATE actors SET persona_prompt = ?1 WHERE id = ?2",
                    _ => return Err(format!("unknown field: {}", field)),
                };
                tx.execute(sql, params![normalized_new, actor_id])
                    .map_err(to_string)?;
                tx.execute(
                    "INSERT INTO actor_edits (actor_id, field, old_value, new_value, reason, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![actor_id, field, old, normalized_new, reason, now],
                )
                .map_err(to_string)?;
                applied.push(field.to_string());
            }

            tx.commit().map_err(to_string)?;

            let actor = conn
                .query_row(
                    "SELECT id, kind, handle, display_name, avatar_seed, bio, specialty, persona_prompt,
                            model_provider, model_name, active_hours, posts_per_day, created_at
                     FROM actors WHERE id = ?1",
                    params![actor_id],
                    map_actor,
                )
                .map_err(to_string)?;

            Ok((actor, applied))
        })
    }

    pub fn get_post_engagement(&self, post_id: i64) -> std::result::Result<serde_json::Value, String> {
        self.with_conn(|conn| {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM posts WHERE id = ?1",
                    params![post_id],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(to_string)? > 0;
            if !exists {
                return Err(format!("post #{post_id} not found"));
            }
            let likes: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM likes WHERE post_id = ?1",
                    params![post_id],
                    |row| row.get(0),
                )
                .map_err(to_string)?;
            let replies: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM posts WHERE parent_id = ?1",
                    params![post_id],
                    |row| row.get(0),
                )
                .map_err(to_string)?;
            Ok(serde_json::json!({
                "post_id": post_id,
                "likes_count": likes,
                "replies_count": replies,
            }))
        })
    }

    pub fn poll_mentions(
        &self,
        actor_id: i64,
        handle: &str,
        limit: i64,
    ) -> std::result::Result<Vec<FeedPost>, String> {
        self.with_conn(|conn| {
            let pattern = format!("%@{}%", handle);
            let since = now_ts() - 7 * 24 * 60 * 60;
            let mut stmt = conn
                .prepare(
                    "SELECT p.id FROM posts p
                     WHERE (p.body LIKE ?1 OR p.quote_body LIKE ?1)
                       AND p.actor_id != ?2
                       AND p.created_at >= ?3
                     ORDER BY p.created_at DESC LIMIT ?4",
                )
                .map_err(to_string)?;
            let ids: Vec<i64> = stmt
                .query_map(params![pattern, actor_id, since, limit], |row| {
                    row.get::<_, i64>(0)
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            ids.into_iter()
                .map(|id| self.fetch_post(conn, id))
                .collect::<std::result::Result<Vec<_>, _>>()
        })
    }

    pub fn schedule_followup(
        &self,
        actor_id: i64,
        delay_minutes: i64,
        context_post_id: Option<i64>,
        salon_id: i64,
    ) -> std::result::Result<i64, String> {
        let due_at = now_ts() + delay_minutes * 60;
        let dedupe_key = format!("followup:{}:{}:{}", actor_id, salon_id, due_at);
        self.with_conn(|conn| {
            require_salon_member(conn, salon_id, actor_id)?;
            if let Some(context_post_id) = context_post_id {
                let post_salon_id = fetch_post_salon_id(conn, context_post_id)?;
                if post_salon_id != salon_id {
                    return Err(format!(
                        "followup context post {} belongs to salon {}, not current salon {}",
                        context_post_id, post_salon_id, salon_id
                    ));
                }
            }
            conn.execute(
                "INSERT INTO agent_queue
                 (actor_id, trigger, context_post_id, salon_id, due_at, claimed_at, completed_at, dedupe_key, created_at)
                 VALUES (?1, 'followup', ?2, ?3, ?4, NULL, NULL, ?5, ?6)",
                params![actor_id, context_post_id, salon_id, due_at, dedupe_key, now_ts()],
            )
            .map_err(to_string)?;
            Ok(due_at)
        })
    }

    pub fn note_write(
        &self,
        actor_id: i64,
        key: &str,
        content: &str,
    ) -> std::result::Result<AgentNote, String> {
        let key_trim = key.trim();
        if key_trim.is_empty() {
            return Err("note key cannot be empty".to_string());
        }
        self.with_conn(|conn| {
            let now = now_ts();
            conn.execute(
                "INSERT INTO agent_notes (actor_id, key, content, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(actor_id, key) DO UPDATE SET
                     content = excluded.content,
                     updated_at = excluded.updated_at",
                params![actor_id, key_trim, content, now],
            )
            .map_err(to_string)?;
            conn.query_row(
                "SELECT id, actor_id, key, content, created_at, updated_at
                 FROM agent_notes WHERE actor_id = ?1 AND key = ?2",
                params![actor_id, key_trim],
                |row| {
                    Ok(AgentNote {
                        id: row.get(0)?,
                        actor_id: row.get(1)?,
                        key: row.get(2)?,
                        content: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .map_err(to_string)
        })
    }

    pub fn note_read(
        &self,
        actor_id: i64,
        key: &str,
    ) -> std::result::Result<Option<AgentNote>, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT id, actor_id, key, content, created_at, updated_at
                 FROM agent_notes WHERE actor_id = ?1 AND key = ?2",
                params![actor_id, key.trim()],
                |row| {
                    Ok(AgentNote {
                        id: row.get(0)?,
                        actor_id: row.get(1)?,
                        key: row.get(2)?,
                        content: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(to_string)
        })
    }

    pub fn save_run_log(
        &self,
        actor_id: i64,
        post_id: i64,
        trigger: &str,
        reasoning: Option<String>,
        tool_calls_json: String,
    ) -> std::result::Result<(), String> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO agent_run_logs (actor_id, post_id, trigger, reasoning, tool_calls, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![actor_id, post_id, trigger, reasoning, tool_calls_json, now_ts()],
            )
            .map_err(to_string)?;
            Ok(())
        })
    }

    pub fn get_run_log_for_post(
        &self,
        post_id: i64,
    ) -> std::result::Result<Option<crate::models::AgentRunLog>, String> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT reasoning, tool_calls, trigger, created_at
                 FROM agent_run_logs WHERE post_id = ?1 LIMIT 1",
                params![post_id],
                |row| {
                    Ok(crate::models::AgentRunLog {
                        reasoning: row.get(0)?,
                        tool_calls_json: row.get(1)?,
                        trigger: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(to_string)
        })
    }

    pub fn note_list(&self, actor_id: i64) -> std::result::Result<Vec<AgentNote>, String> {
        self.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, actor_id, key, content, created_at, updated_at
                     FROM agent_notes WHERE actor_id = ?1
                     ORDER BY updated_at DESC",
                )
                .map_err(to_string)?;
            let rows = stmt
                .query_map(params![actor_id], |row| {
                    Ok(AgentNote {
                        id: row.get(0)?,
                        actor_id: row.get(1)?,
                        key: row.get(2)?,
                        content: row.get(3)?,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })
                .map_err(to_string)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(to_string)?;
            Ok(rows)
        })
    }

    pub fn mark_notifications_read(&self, ids: &[i64]) -> std::result::Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        self.with_conn(|conn| {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "UPDATE notifications SET read = 1 WHERE id IN ({})",
                placeholders
            );
            let params: Vec<Box<dyn rusqlite::types::ToSql>> =
                ids.iter().map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>).collect();
            conn.execute(&sql, rusqlite::params_from_iter(params.iter()))
                .map_err(to_string)?;
            Ok(())
        })
    }

    fn enqueue_mentions_for_post(
        &self,
        conn: &Connection,
        author_id: i64,
        post_id: i64,
        salon_id: i64,
        body: &str,
    ) -> std::result::Result<(), String> {
        for handle in extract_mentions(body) {
            let actor = conn
                .query_row(
                    "SELECT id, kind, handle FROM actors WHERE LOWER(handle) = LOWER(?1) LIMIT 1",
                    params![handle],
                    |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                        ))
                    },
                )
                .optional()
                .map_err(to_string)?;

            let Some((actor_id, kind, canonical_handle)) = actor else {
                continue;
            };
            if kind != "agent" || actor_id == author_id {
                continue;
            }
            if !is_salon_member_conn(conn, salon_id, actor_id)? {
                continue;
            }
            if thread_response_limit_reached(conn, actor_id, post_id)? {
                continue;
            }

            let dedupe_key = format!("mention:{}:{}", actor_id, post_id);
            let due_at = now_ts() + mention_jitter_seconds(post_id, actor_id);
            conn.execute(
                "INSERT INTO agent_queue (actor_id, trigger, context_post_id, salon_id, due_at, claimed_at, completed_at, dedupe_key, created_at)
                 VALUES (?1, 'mention', ?2, ?3, ?4, NULL, NULL, ?5, ?6)
                 ON CONFLICT(dedupe_key) DO NOTHING",
                params![actor_id, post_id, salon_id, due_at, dedupe_key, now_ts()],
            )
            .map_err(to_string)?;

            let _ = canonical_handle;
        }
        Ok(())
    }

    fn enqueue_reply_target(
        &self,
        conn: &Connection,
        replier_id: i64,
        reply_post_id: i64,
        parent_id: i64,
        salon_id: i64,
    ) -> std::result::Result<(), String> {
        let parent = conn
            .query_row(
                "SELECT p.actor_id, a.kind
                 FROM posts p JOIN actors a ON a.id = p.actor_id
                 WHERE p.id = ?1",
                params![parent_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(to_string)?;

        let Some((parent_author_id, parent_kind)) = parent else {
            return Ok(());
        };
        if parent_kind != "agent" || parent_author_id == replier_id {
            return Ok(());
        }
        if is_actor_agent(conn, replier_id)? {
            return Ok(());
        }
        if thread_response_limit_reached(conn, parent_author_id, reply_post_id)? {
            return Ok(());
        }

        let window_start = now_ts() - 15 * 60;
        let recent_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM agent_queue
                 WHERE actor_id = ?1 AND trigger = 'reply' AND created_at >= ?2",
                params![parent_author_id, window_start],
                |row| row.get(0),
            )
            .map_err(to_string)?;
        if recent_count >= 3 {
            return Ok(());
        }

        let dedupe_key = format!("reply:{}:{}", parent_author_id, reply_post_id);
        let due_at = now_ts() + mention_jitter_seconds(reply_post_id, parent_author_id);
        conn.execute(
            "INSERT INTO agent_queue (actor_id, trigger, context_post_id, salon_id, due_at, claimed_at, completed_at, dedupe_key, created_at)
             VALUES (?1, 'reply', ?2, ?3, ?4, NULL, NULL, ?5, ?6)
             ON CONFLICT(dedupe_key) DO NOTHING",
            params![parent_author_id, reply_post_id, salon_id, due_at, dedupe_key, now_ts()],
        )
        .map_err(to_string)?;
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProviderKeys {
    providers: BTreeMap<String, String>,
}

#[derive(Debug)]
struct SeedActor<'a> {
    kind: &'a str,
    handle: &'a str,
    display_name: &'a str,
    avatar_seed: Option<&'a str>,
    bio: &'a str,
    specialty: Option<&'a str>,
    persona_prompt: Option<String>,
    model_provider: Option<&'a str>,
    model_name: Option<&'a str>,
    active_hours: Option<&'a str>,
    posts_per_day: Option<i64>,
}

impl<'a> SeedActor<'a> {
    fn human(
        handle: &'a str,
        display_name: &'a str,
        bio: &'a str,
        specialty: Option<&'a str>,
        active_hours: Option<&'a str>,
        posts_per_day: Option<i64>,
    ) -> Self {
        Self {
            kind: "human",
            handle,
            display_name,
            avatar_seed: Some(handle),
            bio,
            specialty,
            persona_prompt: None,
            model_provider: None,
            model_name: None,
            active_hours,
            posts_per_day,
        }
    }

    fn agent(
        handle: &'a str,
        display_name: &'a str,
        bio: &'a str,
        specialty: &'a str,
        active_hours: &'a str,
        posts_per_day: i64,
    ) -> Self {
        Self {
            kind: "agent",
            handle,
            display_name,
            avatar_seed: Some(handle),
            bio,
            specialty: Some(specialty),
            persona_prompt: personas::persona_prompt(handle),
            model_provider: Some("deepseek"),
            model_name: Some("deepseek-reasoner"),
            active_hours: Some(active_hours),
            posts_per_day: Some(posts_per_day),
        }
    }
}

fn map_actor(row: &rusqlite::Row<'_>) -> rusqlite::Result<Actor> {
    Ok(Actor {
        id: row.get(0)?,
        kind: row.get(1)?,
        handle: row.get(2)?,
        display_name: row.get(3)?,
        avatar_seed: row.get(4)?,
        bio: row.get(5)?,
        specialty: row.get(6)?,
        persona_prompt: row.get(7)?,
        model_provider: row.get(8)?,
        model_name: row.get(9)?,
        active_hours: row.get(10)?,
        posts_per_day: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn map_file_info(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileInfo> {
    Ok(FileInfo {
        id: row.get(0)?,
        salon_id: row.get(1)?,
        uploader_id: row.get(2)?,
        original_name: row.get(3)?,
        kind: row.get(4)?,
        size_bytes: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn fetch_file_info(conn: &Connection, file_id: i64) -> std::result::Result<FileInfo, String> {
    conn.query_row(
        "SELECT id, salon_id, uploader_id, original_name, kind, size_bytes, created_at
         FROM files WHERE id = ?1",
        params![file_id],
        map_file_info,
    )
    .optional()
    .map_err(to_string)?
    .ok_or_else(|| format!("file {} not found", file_id))
}

fn fetch_actor_id_by_handle(conn: &Connection, handle: &str) -> std::result::Result<i64, String> {
    let normalized = handle.trim().trim_start_matches('@');
    conn.query_row(
        "SELECT id FROM actors WHERE LOWER(handle) = LOWER(?1)",
        params![normalized],
        |row| row.get(0),
    )
    .map_err(|_| format!("actor @{normalized} not found"))
}

fn validate_task_status(status: &str) -> std::result::Result<(), String> {
    if matches!(status, "todo" | "in_progress" | "done") {
        Ok(())
    } else {
        Err(format!("invalid task status: {status}"))
    }
}

fn fetch_task(conn: &Connection, task_id: i64) -> std::result::Result<Task, String> {
    conn.query_row(
        "SELECT t.id, t.salon_id, t.title, t.description, t.status, t.created_by, c.handle,
                t.assigned_to, a.handle, t.deliverable_post_id, t.created_at, t.updated_at
         FROM tasks t
         JOIN actors c ON c.id = t.created_by
         LEFT JOIN actors a ON a.id = t.assigned_to
         WHERE t.id = ?1",
        params![task_id],
        |row| {
            Ok(Task {
                id: row.get(0)?,
                salon_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: row.get(4)?,
                created_by: row.get(5)?,
                created_by_handle: row.get(6)?,
                assigned_to: row.get(7)?,
                assigned_to_handle: row.get(8)?,
                deliverable_post_id: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        },
    )
    .optional()
    .map_err(to_string)?
    .ok_or_else(|| format!("task {} not found", task_id))
}

fn map_task_log(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskLog> {
    Ok(TaskLog {
        id: row.get(0)?,
        task_id: row.get(1)?,
        actor_id: row.get(2)?,
        actor_handle: row.get(3)?,
        action: row.get(4)?,
        note: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn insert_task_log(
    conn: &Connection,
    task_id: i64,
    actor_id: i64,
    action: &str,
    note: Option<&str>,
) -> std::result::Result<TaskLog, String> {
    conn.execute(
        "INSERT INTO task_logs (task_id, actor_id, action, note, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![task_id, actor_id, action, note, now_ts()],
    )
    .map_err(to_string)?;
    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT tl.id, tl.task_id, tl.actor_id, a.handle, tl.action, tl.note, tl.created_at
         FROM task_logs tl
         JOIN actors a ON a.id = tl.actor_id
         WHERE tl.id = ?1",
        params![id],
        map_task_log,
    )
    .map_err(to_string)
}

fn now_ts() -> i64 {
    Utc::now().timestamp()
}

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("agent-salon")
}

fn normalize_post_trigger(trigger: &str) -> &str {
    match trigger {
        "mention" | "reply" => "reactive",
        other => other,
    }
}

fn thread_root_id(conn: &Connection, post_id: i64) -> std::result::Result<i64, String> {
    let mut current_id = post_id;
    loop {
        let parent_id: Option<i64> = conn
            .query_row(
                "SELECT parent_id FROM posts WHERE id = ?1",
                params![current_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(to_string)?
            .flatten();

        match parent_id {
            Some(id) => current_id = id,
            None => break,
        }
    }
    Ok(current_id)
}

fn count_thread_posts(conn: &Connection, root_id: i64) -> std::result::Result<i64, String> {
    conn.query_row(
        "WITH RECURSIVE thread(id) AS (
            SELECT id FROM posts WHERE id = ?1
            UNION ALL
            SELECT posts.id FROM posts JOIN thread ON posts.parent_id = thread.id
        ) SELECT COUNT(*) FROM thread",
        params![root_id],
        |row| row.get(0),
    )
    .map_err(to_string)
}

fn count_actor_posts_in_thread(
    conn: &Connection,
    actor_id: i64,
    root_id: i64,
) -> std::result::Result<i64, String> {
    conn.query_row(
        "WITH RECURSIVE thread(id) AS (
            SELECT id FROM posts WHERE id = ?1
            UNION ALL
            SELECT posts.id FROM posts JOIN thread ON posts.parent_id = thread.id
        ) SELECT COUNT(*) FROM posts p JOIN thread t ON p.id = t.id WHERE p.actor_id = ?2",
        params![root_id, actor_id],
        |row| row.get(0),
    )
    .map_err(to_string)
}

fn thread_response_limit_reached(
    conn: &Connection,
    actor_id: i64,
    post_id: i64,
) -> std::result::Result<bool, String> {
    let root_id = thread_root_id(conn, post_id)?;
    if count_thread_posts(conn, root_id)? >= MAX_THREAD_POSTS_FOR_AGENT_QUEUE {
        return Ok(true);
    }
    count_actor_posts_in_thread(conn, actor_id, root_id)
        .map(|count| count >= MAX_AGENT_POSTS_PER_THREAD)
}

fn is_actor_agent(conn: &Connection, actor_id: i64) -> std::result::Result<bool, String> {
    conn.query_row(
        "SELECT kind = 'agent' FROM actors WHERE id = ?1",
        params![actor_id],
        |row| row.get::<_, bool>(0),
    )
    .optional()
    .map_err(to_string)
    .map(|value| value.unwrap_or(false))
}

fn is_post_by_agent(conn: &Connection, post_id: i64) -> std::result::Result<bool, String> {
    conn.query_row(
        "SELECT a.kind = 'agent'
         FROM posts p
         JOIN actors a ON a.id = p.actor_id
         WHERE p.id = ?1",
        params![post_id],
        |row| row.get::<_, bool>(0),
    )
    .optional()
    .map_err(to_string)
    .map(|value| value.unwrap_or(false))
}

fn normalize_media_url(url: &str) -> std::result::Result<String, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("media url cannot be empty".to_string());
    }
    if !trimmed.starts_with("https://") {
        return Err("media url must be https".to_string());
    }
    if trimmed.chars().count() > 2048 {
        return Err("media url is too long".to_string());
    }
    Ok(trimmed.to_string())
}

fn normalize_optional_source_url(url: Option<&str>) -> std::result::Result<Option<String>, String> {
    let Some(trimmed) = url.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if trimmed.is_empty() {
        return Ok(None);
    }
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err("media source url must be http or https".to_string());
    }
    if trimmed.chars().count() > 2048 {
        return Err("media source url is too long".to_string());
    }
    Ok(Some(trimmed.to_string()))
}

fn normalize_optional_text(value: Option<&str>, max_chars: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(max_chars).collect::<String>())
}

fn mention_jitter_seconds(post_id: i64, actor_id: i64) -> i64 {
    ((post_id * 31 + actor_id * 17).abs() % 291) + 10
}

fn extract_mentions(body: &str) -> Vec<String> {
    let mut mentions = BTreeSet::new();
    let characters = body.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < characters.len() {
        if characters[index] != '@' {
            index += 1;
            continue;
        }

        let start = index + 1;
        let mut end = start;
        while end < characters.len()
            && (characters[end].is_ascii_alphanumeric() || characters[end] == '_')
        {
            end += 1;
        }

        if end > start {
            mentions.insert(characters[start..end].iter().collect::<String>());
        }
        index = end;
    }

    mentions.into_iter().collect()
}

fn to_string(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> std::result::Result<bool, String> {
    let mut statement = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(to_string)?;
    let mut rows = statement.query([]).map_err(to_string)?;
    while let Some(row) = rows.next().map_err(to_string)? {
        let name: String = row.get(1).map_err(to_string)?;
        if name.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn fetch_salon(conn: &Connection, salon_id: i64) -> std::result::Result<Salon, String> {
    conn.query_row(
        "SELECT s.id, s.name, s.topic, s.created_by, s.created_at, s.last_post_at,
                (SELECT COUNT(*) FROM salon_members m WHERE m.salon_id = s.id) AS member_count
         FROM salons s WHERE s.id = ?1",
        params![salon_id],
        |row| {
            Ok(Salon {
                id: row.get(0)?,
                name: row.get(1)?,
                topic: row.get(2)?,
                created_by: row.get(3)?,
                created_at: row.get(4)?,
                last_post_at: row.get(5)?,
                member_count: row.get(6)?,
            })
        },
    )
    .map_err(to_string)
}

fn is_salon_member_conn(
    conn: &Connection,
    salon_id: i64,
    actor_id: i64,
) -> std::result::Result<bool, String> {
    conn.query_row(
        "SELECT 1 FROM salon_members WHERE salon_id = ?1 AND actor_id = ?2",
        params![salon_id, actor_id],
        |_| Ok(()),
    )
    .optional()
    .map_err(to_string)
    .map(|value| value.is_some())
}

fn require_salon_member(
    conn: &Connection,
    salon_id: i64,
    actor_id: i64,
) -> std::result::Result<(), String> {
    if is_salon_member_conn(conn, salon_id, actor_id)? {
        Ok(())
    } else {
        Err(format!(
            "actor {} is not a member of salon {}",
            actor_id, salon_id
        ))
    }
}

fn fetch_post_salon_id(conn: &Connection, post_id: i64) -> std::result::Result<i64, String> {
    let salon_id: Option<i64> = conn
        .query_row(
            "SELECT salon_id FROM posts WHERE id = ?1",
            params![post_id],
            |row| row.get(0),
        )
        .map_err(to_string)?;
    Ok(salon_id.unwrap_or(GENERAL_SALON_ID))
}

fn touch_salon_last_post(
    conn: &Connection,
    salon_id: i64,
    timestamp: i64,
) -> std::result::Result<(), String> {
    conn.execute(
        "UPDATE salons SET last_post_at = ?1 WHERE id = ?2",
        params![timestamp, salon_id],
    )
    .map_err(to_string)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use rusqlite::Connection;

    use super::{extract_mentions, normalize_post_trigger, AppState, GENERAL_SALON_ID};

    fn test_state() -> AppState {
        AppState {
            conn: Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
            app_data_dir: Arc::new(PathBuf::from(".")),
        }
    }

    #[test]
    fn extracts_unique_ascii_mentions() {
        let mentions = extract_mentions("hi @Jasmine and @Marc, looping @Jasmine again");
        assert_eq!(mentions, vec!["Jasmine".to_string(), "Marc".to_string()]);
    }

    #[test]
    fn maps_mention_trigger_to_reactive_storage() {
        assert_eq!(normalize_post_trigger("mention"), "reactive");
        assert_eq!(normalize_post_trigger("reply"), "reactive");
        assert_eq!(normalize_post_trigger("scheduled"), "scheduled");
    }

    #[test]
    fn reply_to_agent_post_enqueues_parent_author() {
        let state = test_state();
        state.initialize().unwrap();

        let jasmine = state.get_actor("Jasmine").unwrap();
        let agent_post = state
            .create_post_as_actor(jasmine.id, "neural plasticity ponderings", "scheduled", GENERAL_SALON_ID)
            .unwrap();

        // human replies without @-mention
        let human_id = state
            .with_conn(|conn| state.human_actor_id(conn))
            .unwrap();
        state
            .reply_as_actor(human_id, agent_post.id, "interesting, tell me more", "manual")
            .unwrap();

        state
            .with_conn(|conn| {
                conn.execute("UPDATE agent_queue SET due_at = ?1", rusqlite::params![0_i64])
                    .map_err(super::to_string)?;
                Ok(())
            })
            .unwrap();
        let claimed = state.claim_due_triggers(10).unwrap();
        assert!(
            claimed.iter().any(|t| t.actor_handle == "Jasmine" && t.trigger == "reply"),
            "expected Jasmine to be enqueued for a reply, got {:?}",
            claimed
        );
    }

    #[test]
    fn seed_and_repost_dedupe_smoke_test() {
        let state = test_state();
        state.initialize().unwrap();

        let actors = state.list_actors().unwrap();
        assert!(actors.iter().any(|actor| actor.handle == "You"));
        assert!(actors.iter().any(|actor| actor.handle == "Harry"));

        let root = state.create_human_post("hello world", GENERAL_SALON_ID).unwrap();
        let first = state.repost_as_human(root.id, None).unwrap();
        let second = state.repost_as_human(root.id, Some("new quote ignored because repost is idempotent")).unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(state.list_posts(None, None, 20).unwrap().len(), 2);
    }

    #[test]
    fn delete_post_as_admin_removes_thread_subtree() {
        let state = test_state();
        state.initialize().unwrap();

        let root = state.create_human_post("root", GENERAL_SALON_ID).unwrap();
        let reply = state.reply_as_human(root.id, "reply").unwrap();
        let repost = state.repost_as_human(root.id, Some("quote")).unwrap();

        let deleted = state.delete_post_as_admin(root.id).unwrap();
        assert!(deleted.contains(&root.id));
        assert!(deleted.contains(&reply.id));
        assert!(deleted.contains(&repost.id));
        assert!(state.list_posts(None, None, 20).unwrap().is_empty());
        assert!(state.delete_post_as_admin(root.id).is_err());
    }

    #[test]
    fn update_self_logs_changes_and_ignores_noops() {
        use crate::models::SelfEdits;

        let state = test_state();
        state.initialize().unwrap();

        let jasmine = state.get_actor("Jasmine").unwrap();

        // First edit: change bio + specialty, keep display_name.
        let edits = SelfEdits {
            display_name: None,
            bio: Some("sleep researcher with a crush on attention networks".to_string()),
            specialty: Some("neuroscience / sleep".to_string()),
            persona_prompt: None,
        };
        let (actor, applied) = state
            .update_actor_self(jasmine.id, &edits, Some("evolved after sleep-debt thread"))
            .unwrap();
        assert_eq!(applied, vec!["bio".to_string(), "specialty".to_string()]);
        assert_eq!(
            actor.bio.as_deref(),
            Some("sleep researcher with a crush on attention networks")
        );

        // Re-applying same values produces a no-op.
        let (_actor, applied_again) = state
            .update_actor_self(jasmine.id, &edits, Some("retry"))
            .unwrap();
        assert!(applied_again.is_empty(), "no-op should produce no edits, got {:?}", applied_again);

        // Clearing bio via empty string writes NULL and logs.
        let clear = SelfEdits {
            display_name: None,
            bio: Some(String::new()),
            specialty: None,
            persona_prompt: None,
        };
        let (actor, applied) = state.update_actor_self(jasmine.id, &clear, None).unwrap();
        assert_eq!(applied, vec!["bio".to_string()]);
        assert!(actor.bio.is_none());

        // Empty display_name must NOT be allowed (actors.display_name is NOT NULL).
        let bad = SelfEdits {
            display_name: Some("   ".to_string()),
            bio: None,
            specialty: None,
            persona_prompt: None,
        };
        let (_actor, applied) = state.update_actor_self(jasmine.id, &bad, None).unwrap();
        assert!(applied.is_empty(), "blank display_name should be ignored");

        // Audit log has entries.
        let count: i64 = state
            .with_conn(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM actor_edits WHERE actor_id = ?1",
                    rusqlite::params![jasmine.id],
                    |row| row.get(0),
                )
                .map_err(super::to_string)
            })
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn agent_notes_write_read_list() {
        let state = test_state();
        state.initialize().unwrap();

        let jasmine = state.get_actor("Jasmine").unwrap();

        state
            .note_write(jasmine.id, "thesis-sleep", "short sleep kills attention more than mood")
            .unwrap();
        state
            .note_write(jasmine.id, "todo", "follow up on orexin paper")
            .unwrap();

        // overwrite
        let updated = state
            .note_write(jasmine.id, "thesis-sleep", "revised: short sleep compounds with stress")
            .unwrap();
        assert!(updated.content.starts_with("revised"));

        let read = state.note_read(jasmine.id, "thesis-sleep").unwrap().unwrap();
        assert!(read.content.starts_with("revised"));

        let missing = state.note_read(jasmine.id, "nope").unwrap();
        assert!(missing.is_none());

        let all = state.note_list(jasmine.id).unwrap();
        assert_eq!(all.len(), 2);

        // empty key is rejected
        assert!(state.note_write(jasmine.id, "  ", "x").is_err());
    }

    #[test]
    fn mention_queue_and_recursive_thread_smoke_test() {
        let state = test_state();
        state.initialize().unwrap();

        let root = state.create_human_post("hi @Jasmine", GENERAL_SALON_ID).unwrap();
        state
            .with_conn(|conn| {
                conn.execute("UPDATE agent_queue SET due_at = ?1", rusqlite::params![0_i64])
                    .map_err(super::to_string)?;
                Ok(())
            })
            .unwrap();
        let claimed = state.claim_due_triggers(10).unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].actor_handle, "Jasmine");
        assert_eq!(claimed[0].context_post_id, Some(root.id));

        let jasmine = state.get_actor("Jasmine").unwrap();
        let reply = state
            .reply_as_actor(jasmine.id, root.id, "reply one", "reactive")
            .unwrap();
        let _nested = state
            .reply_as_actor(jasmine.id, reply.id, "reply two", "reactive")
            .unwrap();

        let thread = state.get_thread(root.id).unwrap();
        assert_eq!(thread.len(), 3);
        assert!(thread.iter().any(|post| post.parent_id == Some(reply.id)));
    }
}
