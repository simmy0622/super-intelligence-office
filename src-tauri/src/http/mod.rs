use axum::{
    body::Body,
    extract::{Path, Query, State},
    extract::DefaultBodyLimit,
    extract::Multipart,
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde::de::Deserializer;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    agents::tools,
    db::{AppState, GENERAL_SALON_ID, MAX_SALONS_V1},
    models::{
        Actor, AgentRun, AgentStepResult, AgentToolbox, FeedPost, FileInfo, FileSearchResult,
        Notification, Salon, SalonMember, SelfEdits, SettingEntry, Task,
    },
    scheduler,
    services::files,
};

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:7777";

pub async fn run_http_server(state: AppState) -> Result<(), String> {
    let bind_address = state
        .get_setting_value("http_server_bind_address")?
        .unwrap_or_else(|| DEFAULT_BIND_ADDRESS.to_string());

    let listener = TcpListener::bind(&bind_address)
        .await
        .map_err(|error| format!("failed to bind HTTP server on {}: {}", bind_address, error))?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);
    let _ = HeaderValue::from_static; // silence if unused

    let router = Router::new()
        .route("/health", get(health))
        .route("/api/actors", get(list_actors))
        .route("/api/actors/{handle}", get(get_actor))
        .route("/api/actors/{handle}/toolbox", get(get_actor_toolbox))
        .route("/api/actors/{handle}/run", post(run_actor))
        .route("/api/actors/{handle}/reset-runs", post(reset_agent_runs))
        .route("/api/actors/{handle}", patch(update_actor))
        .route("/api/agent-runs", get(list_agent_runs))
        .route("/api/salons", get(list_salons).post(create_salon))
        .route("/api/salons/{id}", get(get_salon).delete(delete_salon))
        .route("/api/salons/{id}/standup", get(get_salon_standup))
        .route("/api/salons/{id}/tasks", get(list_tasks).post(create_task))
        .route("/api/standup/run", post(force_standup))
        .route(
            "/api/salons/{id}/members",
            get(list_salon_members).post(add_salon_member),
        )
        .route("/api/salons/{id}/files", get(list_salon_files))
        .route("/api/files/upload", post(upload_file))
        .route("/api/files/{id}/download", get(download_file))
        .route("/api/files/{id}/text", get(get_file_text))
        .route("/api/files/generate", post(generate_file))
        .route("/api/posts", get(list_posts).post(create_post))
        .route("/api/posts/search", get(search_posts))
        .route("/api/posts/{post_id}", delete(delete_post))
        .route("/api/posts/{post_id}/thread", get(get_thread))
        .route("/api/posts/{post_id}/replies", post(reply_to_post))
        .route("/api/posts/{post_id}/repost", post(repost_post))
        .route("/api/posts/{post_id}/like-toggle", post(toggle_like))
        .route("/api/posts/{post_id}/run-log", get(get_post_run_log))
        .route("/api/posts/{post_id}/pin-toggle", post(pin_toggle_post))
        .route("/api/tasks/{id}", patch(update_task).delete(delete_task))
        .route("/api/tasks/{id}/claim", post(claim_task))
        .route("/api/tasks/{id}/complete", post(complete_task))
        .route("/api/tasks/{id}/reopen", post(reopen_task))
        .route("/api/settings", get(get_settings).put(put_settings))
        .route("/api/api-keys/{provider}", get(get_api_key_status).put(put_api_key))
        .route("/api/notifications", get(list_notifications))
        .route("/api/notifications/read", post(mark_notifications_read))
        .route("/api/notifications/unread-count", get(unread_notification_count))
        .layer(DefaultBodyLimit::max(files::MAX_UPLOAD_BYTES))
        .layer(cors)
        .with_state(state);

    eprintln!("[http] Agent Salon API listening on http://{bind_address}");
    axum::serve(listener, router)
        .await
        .map_err(|error| error.to_string())
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    let bind_address = state
        .get_setting_value("http_server_bind_address")?
        .unwrap_or_else(|| DEFAULT_BIND_ADDRESS.to_string());

    Ok(Json(HealthResponse {
        ok: true,
        bind_address,
    }))
}

async fn list_actors(State(state): State<AppState>) -> Result<Json<Vec<Actor>>, ApiError> {
    Ok(Json(state.list_actors()?))
}

async fn get_actor(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Json<Actor>, ApiError> {
    Ok(Json(state.get_actor(&handle)?))
}

async fn update_actor(
    State(state): State<AppState>,
    Path(handle): Path<String>,
    Json(edits): Json<SelfEdits>,
) -> Result<Json<Actor>, ApiError> {
    let actor = state.get_actor(&handle)?;
    let (updated, _) = state.update_actor_self(actor.id, &edits, None)?;
    Ok(Json(updated))
}

async fn get_actor_toolbox(
    Path(handle): Path<String>,
) -> Result<Json<AgentToolbox>, ApiError> {
    Ok(Json(
        tools::toolbox_for_actor(&handle)
            .ok_or_else(|| ApiError::from(format!("No specialized toolbox for @{}", handle)))?,
    ))
}

async fn run_actor(
    State(state): State<AppState>,
    Path(handle): Path<String>,
    Json(body): Json<RunActorRequest>,
) -> Result<Json<AgentStepResult>, ApiError> {
    Ok(Json(
        scheduler::run_agent_step(
            &state,
            &handle,
            body.trigger.as_deref().unwrap_or("manual"),
            body.context_post_id,
            body.salon_id,
        )
        .await?,
    ))
}

async fn list_agent_runs(
    State(state): State<AppState>,
    Query(query): Query<ListAgentRunsQuery>,
) -> Result<Json<Vec<AgentRun>>, ApiError> {
    Ok(Json(state.list_agent_runs(query.limit.unwrap_or(5))?))
}

async fn reset_agent_runs(
    State(state): State<AppState>,
    Path(handle): Path<String>,
) -> Result<Json<ResetRunsResponse>, ApiError> {
    let actor = state.get_actor(&handle)?;
    let reset_count = state.reset_agent_runs(actor.id)?;
    Ok(Json(ResetRunsResponse {
        handle,
        reset_count,
        ok: true,
    }))
}

async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<Vec<FeedPost>>, ApiError> {
    Ok(Json(
        state.list_posts(query.salon_id, query.before_timestamp, query.limit.unwrap_or(20))?,
    ))
}

async fn search_posts(
    State(state): State<AppState>,
    Query(query): Query<SearchPostsQuery>,
) -> Result<Json<Vec<FeedPost>>, ApiError> {
    let limit = query.limit.unwrap_or(30).clamp(1, 100);
    let keyword = query.q.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let actor_handle = query
        .actor_handle
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    Ok(Json(state.search_posts(keyword, actor_handle, limit)?))
}

async fn list_salons(State(state): State<AppState>) -> Result<Json<Vec<Salon>>, ApiError> {
    Ok(Json(state.list_salons()?))
}

async fn get_salon(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Salon>, ApiError> {
    Ok(Json(state.get_salon(id)?))
}

async fn delete_salon(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<OkResponse>, ApiError> {
    state.delete_salon(id)?;
    Ok(Json(OkResponse { ok: true }))
}

async fn list_tasks(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<Vec<Task>>, ApiError> {
    Ok(Json(state.list_tasks(
        id,
        query.status.as_deref(),
        query.assigned_to.as_deref(),
        query.limit.unwrap_or(20),
    )?))
}

async fn get_salon_standup(
    State(state): State<AppState>,
    Path(salon_id): Path<i64>,
) -> Result<Json<Option<FeedPost>>, ApiError> {
    let since = Utc::now().timestamp() - 48 * 3600;
    let post = state.get_latest_standup(salon_id, since)?;
    Ok(Json(post))
}

async fn pin_toggle_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let pinned = state.toggle_pin_post(post_id)?;
    Ok(Json(serde_json::json!({ "pinned": pinned })))
}

async fn force_standup(State(state): State<AppState>) -> Result<StatusCode, ApiError> {
    scheduler::force_standup_pass(&state).await?;
    Ok(StatusCode::OK)
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    let assigned_to = body
        .assigned_to
        .as_ref()
        .map(|value| match value {
            None => Ok(None),
            Some(handle) => state.get_actor_id_by_handle(handle).map(Some),
        })
        .transpose()?;
    let description = body.description.as_ref().map(|value| value.as_deref());

    Ok(Json(state.update_task(
        id,
        body.title.as_deref(),
        description,
        assigned_to,
        body.status.as_deref(),
    )?))
}

async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    state.delete_task(id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn claim_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<TaskActorRequest>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(state.claim_task(id, body.actor_id)?))
}

async fn complete_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<CompleteTaskRequest>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(state.complete_task(id, body.actor_id, body.deliverable_post_id)?))
}

async fn reopen_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<TaskActorRequest>,
) -> Result<Json<Task>, ApiError> {
    Ok(Json(state.reopen_task(id, body.actor_id)?))
}

async fn create_salon(
    State(state): State<AppState>,
    Json(body): Json<CreateSalonRequest>,
) -> Result<(StatusCode, Json<Salon>), ApiError> {
    if state.list_salons()?.len() >= MAX_SALONS_V1 {
        return Err(ApiError::with_status(
            StatusCode::BAD_REQUEST,
            format!("V1 supports at most {} salons", MAX_SALONS_V1),
        ));
    }

    let creator = state
        .list_actors()?
        .into_iter()
        .find(|actor| actor.id == body.created_by)
        .ok_or_else(|| ApiError::from("creator actor not found".to_string()))?;
    if creator.kind != "human" {
        return Err(ApiError::with_status(
            StatusCode::FORBIDDEN,
            "only human actors can create salons".to_string(),
        ));
    }

    Ok((
        StatusCode::CREATED,
        Json(state.create_salon(
            &body.name,
            body.topic.as_deref(),
            body.created_by,
            &body.member_actor_ids,
        )?),
    ))
}

async fn create_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    let creator = state
        .list_actors()?
        .into_iter()
        .find(|actor| actor.id == body.created_by)
        .ok_or_else(|| ApiError::from("creator actor not found".to_string()))?;
    if creator.kind != "human" && !creator.handle.eq_ignore_ascii_case("Nomi") {
        return Err(ApiError::with_status(
            StatusCode::FORBIDDEN,
            "only human or Nomi can create tasks".to_string(),
        ));
    }

    let assigned_to = body
        .assigned_to
        .as_deref()
        .map(|handle| state.get_actor_id_by_handle(handle))
        .transpose()?;

    Ok((
        StatusCode::CREATED,
        Json(state.create_task(
            id,
            &body.title,
            body.description.as_deref(),
            body.created_by,
            assigned_to,
        )?),
    ))
}

async fn list_salon_members(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<SalonMember>>, ApiError> {
    Ok(Json(state.list_salon_members(id)?))
}

async fn add_salon_member(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<AddSalonMemberRequest>,
) -> Result<(StatusCode, Json<SalonMember>), ApiError> {
    let actor = state
        .list_actors()?
        .into_iter()
        .find(|actor| actor.id == body.actor_id)
        .ok_or_else(|| ApiError::from("actor not found".to_string()))?;
    if actor.kind != "agent" {
        return Err(ApiError::with_status(
            StatusCode::BAD_REQUEST,
            "V1 only allows adding agent members from the frontend".to_string(),
        ));
    }
    Ok((StatusCode::CREATED, Json(state.add_salon_member(id, body.actor_id)?)))
}

async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<FileInfo>), ApiError> {
    let mut salon_id: Option<i64> = None;
    let mut actor_id: Option<i64> = None;
    let mut original_name: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|error| ApiError::from(error.to_string()))? {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "salon_id" | "salonId" => {
                let value = field.text().await.map_err(|error| ApiError::from(error.to_string()))?;
                salon_id = value.trim().parse::<i64>().ok();
            }
            "actor_id" | "actorId" => {
                let value = field.text().await.map_err(|error| ApiError::from(error.to_string()))?;
                actor_id = value.trim().parse::<i64>().ok();
            }
            "file" => {
                original_name = field.file_name().map(ToString::to_string);
                let bytes = field.bytes().await.map_err(|error| ApiError::from(error.to_string()))?;
                if bytes.len() > files::MAX_UPLOAD_BYTES {
                    return Err(ApiError::with_status(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "file exceeds 32 MB upload limit".to_string(),
                    ));
                }
                file_bytes = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    let salon_id = salon_id.ok_or_else(|| ApiError::from("salon_id is required".to_string()))?;
    let actor_id = actor_id.ok_or_else(|| ApiError::from("actor_id is required".to_string()))?;
    let original_name = original_name.ok_or_else(|| ApiError::from("file name is required".to_string()))?;
    let file_bytes = file_bytes.ok_or_else(|| ApiError::from("file field is required".to_string()))?;
    let kind = files::kind_from_filename(&original_name)
        .ok_or_else(|| ApiError::from(format!("unsupported file type: {}", original_name)))?;
    let storage_name = files::new_storage_name(&original_name, &kind);
    let uploads_dir = files::uploads_dir(&state.app_data_dir()).map_err(ApiError::from)?;
    let dest = uploads_dir.join(&storage_name);
    std::fs::write(&dest, &file_bytes).map_err(|error| ApiError::from(error.to_string()))?;
    let extracted_text = files::extract_text(&dest, &kind);
    let file = state.upload_file(
        salon_id,
        actor_id,
        &original_name,
        &kind,
        &storage_name,
        file_bytes.len() as i64,
        extracted_text.as_deref(),
    )?;

    Ok((StatusCode::CREATED, Json(file)))
}

async fn download_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, ApiError> {
    let file = state.get_file(id)?;
    let storage_path = state.get_file_storage_path(id)?;
    let path = files::uploads_dir(&state.app_data_dir())
        .map_err(ApiError::from)?
        .join(storage_path);
    let bytes = std::fs::read(path).map_err(|error| ApiError::from(error.to_string()))?;
    let content_type = content_type_for_file(&file);
    let disposition_name = file.original_name.replace('"', "");
    let mut response = Body::from(bytes).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(content_type),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", disposition_name))
            .map_err(|error| ApiError::from(error.to_string()))?,
    );
    Ok(response)
}

async fn get_file_text(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<FileTextResponse>, ApiError> {
    Ok(Json(FileTextResponse {
        text: state.get_file_text(id)?.unwrap_or_default(),
    }))
}

async fn list_salon_files(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<FileSearchResult>>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    if let Some(q) = query.q.as_deref().filter(|value| !value.trim().is_empty()) {
        Ok(Json(state.search_files(id, q, limit)?))
    } else {
        Ok(Json(
            state
                .list_salon_files(id, limit)?
                .into_iter()
                .map(|file| FileSearchResult {
                    file,
                    snippet: String::new(),
                })
                .collect(),
        ))
    }
}

async fn generate_file(
    State(state): State<AppState>,
    Json(body): Json<GenerateFileRequest>,
) -> Result<(StatusCode, Json<GenerateFileResponse>), ApiError> {
    let format = normalize_generated_format(&body.format)?;
    let filename = ensure_filename_extension(&body.filename, &format);
    let storage_name = files::new_storage_name(&filename, &format);
    let uploads_dir = files::uploads_dir(&state.app_data_dir()).map_err(ApiError::from)?;
    let dest = uploads_dir.join(&storage_name);
    files::generate_file(&format, &body.content, &dest).map_err(ApiError::from)?;
    let bytes_len = std::fs::metadata(&dest).map_err(|error| ApiError::from(error.to_string()))?.len() as i64;
    let extracted_text = files::extract_text(&dest, &format).or_else(|| Some(body.content.clone()));
    let file = state.upload_file(
        body.salon_id,
        body.actor_id,
        &filename,
        &format,
        &storage_name,
        bytes_len,
        extracted_text.as_deref(),
    )?;
    let post = state.create_post_as_actor_with_files(
        body.actor_id,
        &body.post_body,
        "manual",
        body.salon_id,
        &[file.id],
    )?;

    Ok((
        StatusCode::CREATED,
        Json(GenerateFileResponse {
            file_id: file.id,
            post_id: post.id,
            file,
            post,
        }),
    ))
}

async fn get_thread(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<Vec<FeedPost>>, ApiError> {
    Ok(Json(state.get_thread(post_id)?))
}

async fn create_post(
    State(state): State<AppState>,
    Json(body): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<FeedPost>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(state.create_human_post_with_files(
            &body.body,
            body.salon_id.unwrap_or(GENERAL_SALON_ID),
            body.file_ids.as_deref().unwrap_or(&[]),
        )?),
    ))
}

async fn delete_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<DeletePostResponse>, ApiError> {
    let deleted_post_ids = state.delete_post_as_admin(post_id)?;
    Ok(Json(DeletePostResponse {
        ok: true,
        deleted_post_ids,
    }))
}

async fn reply_to_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Json(body): Json<ReplyRequest>,
) -> Result<(StatusCode, Json<FeedPost>), ApiError> {
    if let Some(salon_id) = body.salon_id {
        let parent_salon_id = state.post_salon_id(post_id)?;
        if parent_salon_id != salon_id {
            return Err(ApiError::with_status(
                StatusCode::BAD_REQUEST,
                format!(
                    "post {} belongs to salon {}, not requested salon {}",
                    post_id, parent_salon_id, salon_id
                ),
            ));
        }
    }
    Ok((
        StatusCode::CREATED,
        Json(state.reply_as_human_with_files(
            post_id,
            &body.body,
            body.file_ids.as_deref().unwrap_or(&[]),
        )?),
    ))
}

async fn repost_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Json(body): Json<RepostRequest>,
) -> Result<(StatusCode, Json<FeedPost>), ApiError> {
    if let Some(salon_id) = body.salon_id {
        let parent_salon_id = state.post_salon_id(post_id)?;
        if parent_salon_id != salon_id {
            return Err(ApiError::with_status(
                StatusCode::BAD_REQUEST,
                format!(
                    "post {} belongs to salon {}, not requested salon {}",
                    post_id, parent_salon_id, salon_id
                ),
            ));
        }
    }
    Ok((
        StatusCode::CREATED,
        Json(state.repost_as_human(post_id, body.quote_body.as_deref())?),
    ))
}

async fn toggle_like(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<LikeToggleResponse>, ApiError> {
    Ok(Json(LikeToggleResponse {
        liked: state.toggle_like(post_id)?,
    }))
}

async fn get_post_run_log(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<crate::models::AgentRunLog>, ApiError> {
    state
        .get_run_log_for_post(post_id)?
        .ok_or_else(|| ApiError::with_status(StatusCode::NOT_FOUND, "no run log for post".into()))
        .map(Json)
}

async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<Vec<SettingEntry>>, ApiError> {
    Ok(Json(state.get_settings()?))
}

async fn put_settings(
    State(state): State<AppState>,
    Json(body): Json<PutSettingsRequest>,
) -> Result<Json<Vec<SettingEntry>>, ApiError> {
    state.set_settings(&body.settings)?;
    Ok(Json(state.get_settings()?))
}

async fn put_api_key(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Json(body): Json<PutApiKeyRequest>,
) -> Result<Json<PutApiKeyResponse>, ApiError> {
    state.write_api_key(&provider, &body.key)?;
    Ok(Json(PutApiKeyResponse { ok: true, provider }))
}

async fn get_api_key_status(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<ApiKeyStatusResponse>, ApiError> {
    Ok(Json(ApiKeyStatusResponse { configured: state.is_api_key_configured(&provider), provider }))
}

async fn list_notifications(
    State(state): State<AppState>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<Vec<Notification>>, ApiError> {
    Ok(Json(state.list_notifications(query.limit.unwrap_or(30))?))
}

async fn unread_notification_count(
    State(state): State<AppState>,
) -> Result<Json<UnreadCountResponse>, ApiError> {
    Ok(Json(UnreadCountResponse {
        count: state.unread_notification_count()?,
    }))
}

async fn mark_notifications_read(
    State(state): State<AppState>,
    Json(body): Json<MarkReadRequest>,
) -> Result<Json<OkResponse>, ApiError> {
    state.mark_notifications_read(&body.ids)?;
    Ok(Json(OkResponse { ok: true }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    ok: bool,
    bind_address: String,
}

#[derive(Debug, Deserialize)]
struct ListPostsQuery {
    #[serde(alias = "salonId")]
    salon_id: Option<i64>,
    #[serde(alias = "beforeTimestamp")]
    before_timestamp: Option<i64>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SearchPostsQuery {
    q: Option<String>,
    #[serde(alias = "actorHandle")]
    actor_handle: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ListAgentRunsQuery {
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunActorRequest {
    trigger: Option<String>,
    context_post_id: Option<i64>,
    salon_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePostRequest {
    body: String,
    salon_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplyRequest {
    body: String,
    salon_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepostRequest {
    quote_body: Option<String>,
    salon_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSalonRequest {
    name: String,
    topic: Option<String>,
    created_by: i64,
    member_actor_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
struct ListTasksQuery {
    status: Option<String>,
    #[serde(alias = "assignedTo")]
    assigned_to: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTaskRequest {
    title: String,
    description: Option<String>,
    assigned_to: Option<String>,
    created_by: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTaskRequest {
    title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_double_option_string")]
    description: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_double_option_string")]
    assigned_to: Option<Option<String>>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskActorRequest {
    actor_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompleteTaskRequest {
    actor_id: i64,
    deliverable_post_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddSalonMemberRequest {
    actor_id: i64,
}

#[derive(Debug, Deserialize)]
struct ListFilesQuery {
    q: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
struct FileTextResponse {
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateFileRequest {
    format: String,
    filename: String,
    content: String,
    salon_id: i64,
    actor_id: i64,
    post_body: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateFileResponse {
    file_id: i64,
    post_id: i64,
    file: FileInfo,
    post: FeedPost,
}

#[derive(Debug, Serialize)]
struct LikeToggleResponse {
    liked: bool,
}

#[derive(Debug, Deserialize)]
struct PutSettingsRequest {
    settings: Vec<SettingEntry>,
}

#[derive(Debug, Deserialize)]
struct PutApiKeyRequest {
    key: String,
}

#[derive(Debug, Serialize)]
struct PutApiKeyResponse {
    ok: bool,
    provider: String,
}

#[derive(Debug, Serialize)]
struct ApiKeyStatusResponse {
    provider: String,
    configured: bool,
}

#[derive(Debug, Deserialize)]
struct ListNotificationsQuery {
    limit: Option<i64>,
}

#[derive(Debug, Serialize)]
struct UnreadCountResponse {
    count: i64,
}

#[derive(Debug, Deserialize)]
struct MarkReadRequest {
    ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct OkResponse {
    ok: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeletePostResponse {
    ok: bool,
    deleted_post_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResetRunsResponse {
    handle: String,
    reset_count: usize,
    ok: bool,
}

fn content_type_for_file(file: &FileInfo) -> &'static str {
    if file.kind == "image" {
        let lower = file.original_name.to_ascii_lowercase();
        if lower.ends_with(".png") {
            return "image/png";
        }
        if lower.ends_with(".webp") {
            return "image/webp";
        }
        return "image/jpeg";
    }

    match file.kind.as_str() {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "csv" => "text/csv; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn normalize_generated_format(format: &str) -> Result<String, ApiError> {
    let normalized = format.trim().trim_start_matches('.').to_ascii_lowercase();
    match normalized.as_str() {
        "docx" | "xlsx" | "csv" | "md" | "pdf" | "pptx" => Ok(normalized),
        _ => Err(ApiError::with_status(
            StatusCode::BAD_REQUEST,
            format!("unsupported generated file format: {format}"),
        )),
    }
}

fn ensure_filename_extension(filename: &str, format: &str) -> String {
    let trimmed = filename.trim();
    let base = if trimmed.is_empty() { "agent-salon-export" } else { trimmed };
    if base.to_ascii_lowercase().ends_with(&format!(".{format}")) {
        base.to_string()
    } else {
        format!("{base}.{format}")
    }
}

fn deserialize_double_option_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(Option::<String>::deserialize(deserializer)?))
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        let status = if message.contains("Query returned no rows") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };

        Self { status, message }
    }
}

impl ApiError {
    fn with_status(status: StatusCode, message: String) -> Self {
        Self { status, message }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorResponse { error: self.message })).into_response()
    }
}
