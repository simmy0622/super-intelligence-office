import Foundation

@MainActor
final class LocalSalonRuntime {
    static let shared = LocalSalonRuntime()

    private let db: SQLiteDatabase
    private let humanHandle = "you"

    init(database: SQLiteDatabase? = nil) {
        do {
            self.db = try database ?? SQLiteDatabase()
            try migrate()
            try cleanupLegacyMockDataIfNeeded()
            try seedIfNeeded()
            try migrateLegacyInvestorToMarc()
            try migrateLegacyMrxToMike()
            try syncDefaultAvatarSeeds()
        } catch {
            fatalError("Failed to initialize local Salon runtime: \(error)")
        }
    }

    // MARK: Queries

    func listActors() throws -> [Actor] {
        try db.query(
            """
            SELECT id, kind, handle, display_name, avatar_seed, bio, specialty, persona_summary
            FROM actors
            ORDER BY CASE kind WHEN 'human' THEN 0 ELSE 1 END, display_name ASC
            """
        ).map(mapActor)
    }

    func actor(byHandle handle: String) throws -> Actor? {
        try db.query(
            """
            SELECT id, kind, handle, display_name, avatar_seed, bio, specialty, persona_summary
            FROM actors
            WHERE LOWER(handle) = LOWER(?)
            LIMIT 1
            """,
            values: [.text(handle)]
        ).first.map(mapActor)
    }

    func listPosts(limit: Int = 100) throws -> [Post] {
        let ids = try db.query(
            "SELECT id FROM posts ORDER BY created_at DESC LIMIT ?",
            values: [.int(Int64(limit))]
        ).compactMap { $0.int("id") }
        return try ids.map(fetchPost)
    }

    func post(byId id: Int64) throws -> Post? {
        try fetchPost(id)
    }

    func replies(to postId: Int64) throws -> [Post] {
        let ids = try db.query(
            "SELECT id FROM posts WHERE parent_id = ? ORDER BY created_at ASC",
            values: [.int(postId)]
        ).compactMap { $0.int("id") }
        return try ids.map(fetchPost)
    }

    func posts(by actorId: Int64) throws -> [Post] {
        let ids = try db.query(
            "SELECT id FROM posts WHERE actor_id = ? ORDER BY created_at DESC",
            values: [.int(actorId)]
        ).compactMap { $0.int("id") }
        return try ids.map(fetchPost)
    }

    func listNotifications(limit: Int = 50) throws -> [SalonNotification] {
        try db.query(
            """
            SELECT n.id, n.kind, n.actor_id, n.post_id, n.body, n.read, n.created_at,
                   a.kind AS actor_kind, a.handle, a.display_name, a.avatar_seed, a.bio, a.specialty, a.persona_summary
            FROM notifications n
            JOIN actors a ON a.id = n.actor_id
            ORDER BY n.created_at DESC
            LIMIT ?
            """,
            values: [.int(Int64(limit))]
        ).map(mapNotification)
    }

    func listAgentRuns(limit: Int = 20) throws -> [AgentRun] {
        try db.query(
            """
            SELECT ar.id, ar.actor_id, a.handle, a.display_name, ar.trigger, ar.started_at,
                   ar.finished_at, ar.prompt_tokens, ar.completion_tokens, ar.tool_calls, ar.error
            FROM agent_runs ar
            JOIN actors a ON a.id = ar.actor_id
            ORDER BY ar.started_at DESC
            LIMIT ?
            """,
            values: [.int(Int64(limit))]
        ).map(mapAgentRun)
    }

    func recentPosts(since sinceTimestamp: Int64, limit: Int = 50) throws -> [Post] {
        let ids = try db.query(
            "SELECT id FROM posts WHERE created_at >= ? ORDER BY created_at DESC LIMIT ?",
            values: [.int(sinceTimestamp), .int(Int64(limit))]
        ).compactMap { $0.int("id") }
        return try ids.map(fetchPost)
    }

    func hasSuccessfulRun(trigger: String, since sinceTimestamp: Int64) throws -> Bool {
        try db.query(
            """
            SELECT 1 AS found
            FROM agent_runs
            WHERE trigger = ? AND started_at >= ? AND finished_at IS NOT NULL AND error IS NULL
            LIMIT 1
            """,
            values: [.text(trigger), .int(sinceTimestamp)]
        ).isEmpty == false
    }

    func hasActiveRun(actorId: Int64) throws -> Bool {
        try db.query(
            "SELECT 1 AS found FROM agent_runs WHERE actor_id = ? AND finished_at IS NULL LIMIT 1",
            values: [.int(actorId)]
        ).isEmpty == false
    }

    func hasAgentEngaged(actorId: Int64, postId: Int64) throws -> Bool {
        let liked = try db.query(
            "SELECT 1 AS found FROM likes WHERE actor_id = ? AND post_id = ? LIMIT 1",
            values: [.int(actorId), .int(postId)]
        ).isEmpty == false
        if liked { return true }

        return try db.query(
            """
            SELECT 1 AS found
            FROM posts
            WHERE actor_id = ?
              AND (parent_id = ? OR referenced_post_id = ?)
            LIMIT 1
            """,
            values: [.int(actorId), .int(postId), .int(postId)]
        ).isEmpty == false
    }

    // MARK: Mutations

    @discardableResult
    func createHumanPost(body: String) throws -> Post? {
        guard let trimmed = body.trimmedOrNil() else { return nil }
        let actor = try humanActor()
        try db.execute(
            """
            INSERT INTO posts (actor_id, kind, parent_id, referenced_post_id, quote_body, body, trigger, created_at)
            VALUES (?, 'original', NULL, NULL, NULL, ?, 'manual', ?)
            """,
            values: [.int(actor.id), .text(trimmed), .int(Self.now())]
        )
        return try fetchPost(db.lastInsertRowID)
    }

    @discardableResult
    func replyAsHuman(parentId: Int64, body: String) throws -> Post? {
        guard let trimmed = body.trimmedOrNil() else { return nil }
        let actor = try humanActor()
        try db.execute(
            """
            INSERT INTO posts (actor_id, kind, parent_id, referenced_post_id, quote_body, body, trigger, created_at)
            VALUES (?, 'reply', ?, ?, NULL, ?, 'manual', ?)
            """,
            values: [.int(actor.id), .int(parentId), .int(parentId), .text(trimmed), .int(Self.now())]
        )
        return try fetchPost(db.lastInsertRowID)
    }

    @discardableResult
    func repostAsHuman(originalId: Int64, quoteBody: String? = nil) throws -> Post? {
        let actor = try humanActor()
        if let existing = try db.query(
            """
            SELECT id FROM posts
            WHERE actor_id = ? AND kind = 'repost' AND referenced_post_id = ?
            LIMIT 1
            """,
            values: [.int(actor.id), .int(originalId)]
        ).first?.int("id") {
            return try fetchPost(existing)
        }

        try db.execute(
            """
            INSERT INTO posts (actor_id, kind, parent_id, referenced_post_id, quote_body, body, trigger, created_at)
            VALUES (?, 'repost', NULL, ?, ?, NULL, 'manual', ?)
            """,
            values: [.int(actor.id), .int(originalId), quoteBody.sqlValue, .int(Self.now())]
        )
        return try fetchPost(db.lastInsertRowID)
    }

    @discardableResult
    func toggleLike(postId: Int64) throws -> Bool {
        let actor = try humanActor()
        let existing = try db.query(
            "SELECT 1 AS found FROM likes WHERE actor_id = ? AND post_id = ? LIMIT 1",
            values: [.int(actor.id), .int(postId)]
        ).isEmpty == false

        if existing {
            try db.execute(
                "DELETE FROM likes WHERE actor_id = ? AND post_id = ?",
                values: [.int(actor.id), .int(postId)]
            )
            return false
        } else {
            try db.execute(
                "INSERT OR IGNORE INTO likes (actor_id, post_id, created_at) VALUES (?, ?, ?)",
                values: [.int(actor.id), .int(postId), .int(Self.now())]
            )
            return true
        }
    }

    func markAllNotificationsRead() throws {
        try db.execute("UPDATE notifications SET read = 1 WHERE read = 0")
    }

    @discardableResult
    func enqueueReactiveTriggers(contextPostId: Int64) throws -> Int {
        guard let contextPost = try post(byId: contextPostId), contextPost.author.kind == .human else {
            return 0
        }

        let agents = try listActors().filter { $0.isAgent && $0.id != contextPost.author.id }
        let text = "\(contextPost.body ?? "") \(contextPost.quoteBody ?? "")".lowercased()
        var count = 0

        for agent in agents {
            guard try !hasAgentEngaged(actorId: agent.id, postId: contextPostId) else { continue }
            let keywords = specialtyKeywords(for: agent)
            guard keywords.isEmpty || keywords.contains(where: { text.contains($0) }) else { continue }

            let dedupeKey = "reactive:\(agent.id):\(contextPostId)"
            try db.execute(
                """
                INSERT OR IGNORE INTO agent_queue
                    (actor_id, trigger, context_post_id, due_at, claimed_at, completed_at, dedupe_key, created_at)
                VALUES (?, 'reactive', ?, ?, NULL, NULL, ?, ?)
                """,
                values: [.int(agent.id), .int(contextPostId), .int(Self.now()), .text(dedupeKey), .int(Self.now())]
            )
            count += Int(try db.query("SELECT changes() AS count").first?.int("count") ?? 0)
        }

        return count
    }

    @discardableResult
    func enqueueReplyTarget(replyPostId: Int64, parentId: Int64) throws -> Int {
        guard let reply = try post(byId: replyPostId),
              let parent = try post(byId: parentId),
              reply.author.kind == .human,
              parent.author.kind == .agent,
              parent.author.id != reply.author.id
        else {
            return 0
        }

        let dedupeKey = "reply:\(parent.author.id):\(replyPostId)"
        try db.execute(
            """
            INSERT OR IGNORE INTO agent_queue
                (actor_id, trigger, context_post_id, due_at, claimed_at, completed_at, dedupe_key, created_at)
            VALUES (?, 'reactive', ?, ?, NULL, NULL, ?, ?)
            """,
            values: [.int(parent.author.id), .int(replyPostId), .int(Self.now()), .text(dedupeKey), .int(Self.now())]
        )
        return Int(try db.query("SELECT changes() AS count").first?.int("count") ?? 0)
    }

    func claimDueTriggers(limit: Int = 16) throws -> [QueuedTrigger] {
        let now = Self.now()
        let rows = try db.query(
            """
            SELECT q.id, q.actor_id, a.handle, q.trigger, q.context_post_id, q.due_at, q.claimed_at, q.completed_at
            FROM agent_queue q
            JOIN actors a ON a.id = q.actor_id
            WHERE q.completed_at IS NULL
              AND q.claimed_at IS NULL
              AND q.due_at <= ?
            ORDER BY q.due_at ASC, q.id ASC
            LIMIT ?
            """,
            values: [.int(now), .int(Int64(limit))]
        )
        let triggers = rows.map(mapQueuedTrigger)
        for trigger in triggers {
            try db.execute(
                "UPDATE agent_queue SET claimed_at = ? WHERE id = ?",
                values: [.int(now), .int(trigger.id)]
            )
        }
        return triggers
    }

    func completeTrigger(_ id: Int64) throws {
        try db.execute(
            "UPDATE agent_queue SET completed_at = ? WHERE id = ?",
            values: [.int(Self.now()), .int(id)]
        )
    }

    @discardableResult
    func createAgentRun(actorId: Int64, trigger: String) throws -> Int64 {
        let active = try db.query(
            "SELECT id FROM agent_runs WHERE actor_id = ? AND finished_at IS NULL LIMIT 1",
            values: [.int(actorId)]
        ).first?.int("id")
        if active != nil {
            throw RuntimeError.activeRun(actorId)
        }

        try db.execute(
            "INSERT INTO agent_runs (actor_id, trigger, started_at) VALUES (?, ?, ?)",
            values: [.int(actorId), .text(trigger), .int(Self.now())]
        )
        return db.lastInsertRowID
    }

    func finishAgentRun(
        runId: Int64,
        promptTokens: Int64?,
        completionTokens: Int64?,
        toolCalls: String?,
        error: String?
    ) throws {
        try db.execute(
            """
            UPDATE agent_runs
            SET finished_at = ?, prompt_tokens = ?, completion_tokens = ?, tool_calls = ?, error = ?
            WHERE id = ?
            """,
            values: [
                .int(Self.now()),
                promptTokens.sqlValue,
                completionTokens.sqlValue,
                toolCalls.sqlValue,
                error.sqlValue,
                .int(runId),
            ]
        )
    }

    @discardableResult
    func createPostAsActor(actorId: Int64, body: String, trigger: String) throws -> Post? {
        guard let trimmed = body.trimmedOrNil() else { return nil }
        try db.execute(
            """
            INSERT INTO posts (actor_id, kind, parent_id, referenced_post_id, quote_body, body, trigger, created_at)
            VALUES (?, 'original', NULL, NULL, NULL, ?, ?, ?)
            """,
            values: [.int(actorId), .text(trimmed), .text(normalizedTrigger(trigger)), .int(Self.now())]
        )
        return try fetchPost(db.lastInsertRowID)
    }

    @discardableResult
    func replyAsActor(actorId: Int64, parentId: Int64, body: String, trigger: String) throws -> Post? {
        guard let trimmed = body.trimmedOrNil() else { return nil }
        try db.execute(
            """
            INSERT INTO posts (actor_id, kind, parent_id, referenced_post_id, quote_body, body, trigger, created_at)
            VALUES (?, 'reply', ?, ?, NULL, ?, ?, ?)
            """,
            values: [.int(actorId), .int(parentId), .int(parentId), .text(trimmed), .text(normalizedTrigger(trigger)), .int(Self.now())]
        )
        return try fetchPost(db.lastInsertRowID)
    }

    @discardableResult
    func repostAsActor(actorId: Int64, originalId: Int64, quoteBody: String?, trigger: String) throws -> Post? {
        if let existing = try db.query(
            """
            SELECT id FROM posts
            WHERE actor_id = ? AND kind = 'repost' AND referenced_post_id = ?
            LIMIT 1
            """,
            values: [.int(actorId), .int(originalId)]
        ).first?.int("id") {
            return try fetchPost(existing)
        }

        try db.execute(
            """
            INSERT INTO posts (actor_id, kind, parent_id, referenced_post_id, quote_body, body, trigger, created_at)
            VALUES (?, 'repost', NULL, ?, ?, NULL, ?, ?)
            """,
            values: [.int(actorId), .int(originalId), quoteBody.sqlValue, .text(normalizedTrigger(trigger)), .int(Self.now())]
        )
        return try fetchPost(db.lastInsertRowID)
    }

    @discardableResult
    func likeAsActor(actorId: Int64, postId: Int64) throws -> Bool {
        let existing = try db.query(
            "SELECT 1 AS found FROM likes WHERE actor_id = ? AND post_id = ? LIMIT 1",
            values: [.int(actorId), .int(postId)]
        ).isEmpty == false
        if existing { return false }

        try db.execute(
            "INSERT INTO likes (actor_id, post_id, created_at) VALUES (?, ?, ?)",
            values: [.int(actorId), .int(postId), .int(Self.now())]
        )
        return true
    }

    func createNotification(kind: String, actorId: Int64, postId: Int64?, body: String?) throws {
        try db.execute(
            "INSERT INTO notifications (kind, actor_id, post_id, body, read, created_at) VALUES (?, ?, ?, ?, 0, ?)",
            values: [.text(kind), .int(actorId), postId.sqlValue, body?.prefixString(120).sqlValue ?? .null, .int(Self.now())]
        )
    }

    func updateActorSelf(
        actorId: Int64,
        displayName: String?,
        bio: String?,
        specialty: String?,
        personaSummary: String?
    ) throws {
        let actor = try actorById(actorId)
        try db.execute(
            """
            UPDATE actors
            SET display_name = ?, bio = ?, specialty = ?, persona_summary = ?
            WHERE id = ?
            """,
            values: [
                (displayName?.trimmedOrNil() ?? actor.displayName).sqlValue,
                (bio ?? actor.bio).sqlValue,
                (specialty?.trimmedOrNil() ?? actor.specialty).sqlValue,
                (personaSummary?.trimmedOrNil() ?? actor.personaSummary).sqlValue,
                .int(actorId),
            ]
        )
    }

    func updateActorAvatar(actorId: Int64, avatarSeed: String?) throws {
        try db.execute(
            "UPDATE actors SET avatar_seed = ? WHERE id = ?",
            values: [avatarSeed.sqlValue, .int(actorId)]
        )
    }

    @discardableResult
    func writeNote(actorId: Int64, key: String, content: String) throws -> AgentNote {
        guard let trimmedKey = key.trimmedOrNil(), let trimmedContent = content.trimmedOrNil() else {
            throw RuntimeError.invalidInput("note key and content are required")
        }
        let now = Self.now()
        try db.execute(
            """
            INSERT INTO agent_notes (actor_id, key, content, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(actor_id, key) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at
            """,
            values: [.int(actorId), .text(trimmedKey), .text(trimmedContent), .int(now), .int(now)]
        )
        return try readNote(actorId: actorId, key: trimmedKey) ?? AgentNote(id: 0, actorId: actorId, key: trimmedKey, content: trimmedContent, createdAt: now, updatedAt: now)
    }

    func readNote(actorId: Int64, key: String) throws -> AgentNote? {
        try db.query(
            "SELECT id, actor_id, key, content, created_at, updated_at FROM agent_notes WHERE actor_id = ? AND key = ? LIMIT 1",
            values: [.int(actorId), .text(key)]
        ).first.map(mapNote)
    }

    func listNotes(actorId: Int64) throws -> [AgentNote] {
        try db.query(
            "SELECT id, actor_id, key, content, created_at, updated_at FROM agent_notes WHERE actor_id = ? ORDER BY updated_at DESC",
            values: [.int(actorId)]
        ).map(mapNote)
    }

    // MARK: Schema

    private func migrate() throws {
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS actors (
                id INTEGER PRIMARY KEY,
                kind TEXT NOT NULL,
                handle TEXT NOT NULL UNIQUE COLLATE NOCASE,
                display_name TEXT NOT NULL,
                avatar_seed TEXT,
                bio TEXT,
                specialty TEXT,
                persona_summary TEXT,
                persona_prompt TEXT,
                model_provider TEXT,
                model_name TEXT,
                active_hours TEXT,
                posts_per_day INTEGER,
                created_at INTEGER NOT NULL
            )
            """
        )
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                actor_id INTEGER NOT NULL REFERENCES actors(id),
                kind TEXT NOT NULL,
                parent_id INTEGER REFERENCES posts(id),
                referenced_post_id INTEGER REFERENCES posts(id),
                quote_body TEXT,
                body TEXT,
                trigger TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )
            """
        )
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS likes (
                actor_id INTEGER NOT NULL REFERENCES actors(id),
                post_id INTEGER NOT NULL REFERENCES posts(id),
                created_at INTEGER NOT NULL,
                PRIMARY KEY (actor_id, post_id)
            )
            """
        )
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS notifications (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                actor_id INTEGER NOT NULL REFERENCES actors(id),
                post_id INTEGER REFERENCES posts(id),
                body TEXT,
                read INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )
            """
        )
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS agent_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                actor_id INTEGER NOT NULL REFERENCES actors(id),
                trigger TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                finished_at INTEGER,
                prompt_tokens INTEGER,
                completion_tokens INTEGER,
                tool_calls TEXT,
                error TEXT
            )
            """
        )
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            """
        )
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS agent_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                actor_id INTEGER NOT NULL REFERENCES actors(id),
                trigger TEXT NOT NULL,
                context_post_id INTEGER REFERENCES posts(id),
                due_at INTEGER NOT NULL,
                claimed_at INTEGER,
                completed_at INTEGER,
                dedupe_key TEXT,
                created_at INTEGER
            )
            """
        )
        try db.execute(
            """
            CREATE TABLE IF NOT EXISTS agent_notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                actor_id INTEGER NOT NULL REFERENCES actors(id),
                key TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                UNIQUE(actor_id, key)
            )
            """
        )
        try db.execute("CREATE INDEX IF NOT EXISTS idx_posts_created_at ON posts(created_at DESC)")
        try db.execute("CREATE INDEX IF NOT EXISTS idx_posts_parent ON posts(parent_id)")
        try db.execute("CREATE INDEX IF NOT EXISTS idx_notifications_created_at ON notifications(created_at DESC)")
        try? db.execute("ALTER TABLE agent_queue ADD COLUMN dedupe_key TEXT")
        try? db.execute("ALTER TABLE agent_queue ADD COLUMN created_at INTEGER")
        try db.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_queue_dedupe ON agent_queue(dedupe_key)")
    }

    private func cleanupLegacyMockDataIfNeeded() throws {
        let markerKey = "legacy_mock_cleanup_v1"
        let alreadyCleaned = try db.query(
            "SELECT 1 AS found FROM settings WHERE key = ? LIMIT 1",
            values: [.text(markerKey)]
        ).isEmpty == false
        guard !alreadyCleaned else { return }

        let hasLegacySeed = try db.query(
            """
            SELECT 1 AS found
            FROM posts
            WHERE id = 7
              AND actor_id = 1
              AND body = '刚加入这个 salon。还在看这帮 agent 怎么聊。'
            LIMIT 1
            """
        ).isEmpty == false

        if hasLegacySeed {
            try db.execute("CREATE TEMP TABLE IF NOT EXISTS legacy_mock_posts (id INTEGER PRIMARY KEY)")
            try db.execute("DELETE FROM legacy_mock_posts")
            try db.execute(
                """
                WITH RECURSIVE affected(id) AS (
                    SELECT id FROM posts WHERE id BETWEEN 1 AND 10
                    UNION
                    SELECT p.id
                    FROM posts p
                    JOIN affected a
                      ON p.parent_id = a.id OR p.referenced_post_id = a.id
                )
                INSERT OR IGNORE INTO legacy_mock_posts(id)
                SELECT id FROM affected
                """
            )
            try db.execute("DELETE FROM notifications WHERE post_id IN (SELECT id FROM legacy_mock_posts)")
            try db.execute("DELETE FROM likes WHERE post_id IN (SELECT id FROM legacy_mock_posts)")
            try db.execute("DELETE FROM agent_queue WHERE context_post_id IN (SELECT id FROM legacy_mock_posts)")
            try db.execute("DELETE FROM posts WHERE id IN (SELECT id FROM legacy_mock_posts)")
            try db.execute("DELETE FROM legacy_mock_posts")
        }

        try db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, 'done')",
            values: [.text(markerKey)]
        )
    }

    private func seedIfNeeded() throws {
        let count = try db.query("SELECT COUNT(*) AS count FROM actors").first?.int("count") ?? 0
        guard count == 0 else { return }

        try db.execute("BEGIN IMMEDIATE TRANSACTION")
        do {
            try seedActors()
            try db.execute("COMMIT")
        } catch {
            try? db.execute("ROLLBACK")
            throw error
        }
    }

    private func seedActors() throws {
        let now = Self.now()
        let actors: [(Int64, ActorKind, String, String, String?, String, String?, String?)] = [
            (1, .human, "you", "You", nil, "刚加入 salon，先看看这帮人怎么聊。", nil, nil),
            (2, .agent, "jasmine", "Jasmine", "agent-1", "上海出生、纽约生活的媒体人和播客主理人。", "媒体 · 文化 · 公共叙事", "不羁、锐利、热爱生活，拆穿太顺的故事。"),
            (3, .agent, "marc", "Marc", "agent-2", "硅谷早期 VC 创始合伙人，强技术乐观主义。", "VC · 技术乐观 · 市场结构", "结论先行，先看 usage、distribution 和 workflow wedge。"),
            (4, .agent, "angel", "Angel", "agent-3", "Berkeley 心理学博士。", "心理 · 健康行为", "温柔，但有分量。让人「被理解」而不是「被教育」。"),
            (5, .agent, "mike", "Mike", "agent-4", "硅谷华人 AI 创业者，清华姚班、Stanford CS PhD。", "AI Lab · 科学家型创业者", "Sparse Labs 创始人，lean team，高密度实验和真实 workflow。"),
            (6, .agent, "jasper", "Jasper", "agent-5", "Meridian Macro Partners 创始人，乔治城博士。", "全球宏观 · 国别研究 · 贸易周期", "冷静精密，田野笔记式地画出宏观约束。"),
            (7, .agent, "alex", "Alex", "agent-6", "Praxis Intelligence CEO，Stanford Law 与德国社会学训练。", "AI · 国家能力 · 合法性", "冷峻强硬，喜欢引经据典，但最后落到责任链。"),
        ]

        for actor in actors {
            try db.execute(
                """
                INSERT INTO actors (id, kind, handle, display_name, avatar_seed, bio, specialty, persona_summary, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                values: [
                    .int(actor.0), .text(actor.1.rawValue), .text(actor.2), .text(actor.3),
                    actor.4.sqlValue, .text(actor.5), actor.6.sqlValue, actor.7.sqlValue, .int(now),
                ]
            )
        }
    }

    private func migrateLegacyInvestorToMarc() throws {
        let markerKey = "legacy_investor_to_marc_v1"
        let alreadyMigrated = try db.query(
            "SELECT 1 AS found FROM settings WHERE key = ? LIMIT 1",
            values: [.text(markerKey)]
        ).isEmpty == false
        guard !alreadyMigrated else { return }

        try db.execute(
            """
            UPDATE actors
            SET handle = 'marc',
                display_name = 'Marc',
                avatar_seed = 'agent-2',
                bio = '硅谷早期 VC 创始合伙人，强技术乐观主义。',
                specialty = 'VC · 技术乐观 · 市场结构',
                persona_summary = '结论先行，先看 usage、distribution 和 workflow wedge。'
            WHERE LOWER(handle) = ?
            """,
            values: [.text("jimmy")]
        )

        try db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, 'done')",
            values: [.text(markerKey)]
        )
    }

    private func migrateLegacyMrxToMike() throws {
        let markerKey = "legacy_mrx_to_mike_v1"
        let alreadyMigrated = try db.query(
            "SELECT 1 AS found FROM settings WHERE key = ? LIMIT 1",
            values: [.text(markerKey)]
        ).isEmpty == false
        guard !alreadyMigrated else { return }

        try db.execute(
            """
            UPDATE actors
            SET handle = 'mike',
                display_name = 'Mike',
                avatar_seed = 'agent-4',
                bio = '硅谷华人 AI 创业者，清华姚班、Stanford CS PhD。',
                specialty = 'AI Lab · 科学家型创业者',
                persona_summary = 'Sparse Labs 创始人，lean team，高密度实验和真实 workflow。'
            WHERE LOWER(handle) = ?
            """,
            values: [.text("mrx")]
        )

        try db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, 'done')",
            values: [.text(markerKey)]
        )
    }

    private func syncDefaultAvatarSeeds() throws {
        for (handle, seed) in Self.defaultAgentAvatarSeeds {
            try db.execute(
                """
                UPDATE actors
                SET avatar_seed = ?
                WHERE LOWER(handle) = LOWER(?)
                  AND (avatar_seed IS NULL OR avatar_seed = '' OR LOWER(avatar_seed) = LOWER(handle))
                """,
                values: [.text(seed), .text(handle)]
            )
        }
    }

    // MARK: Mapping

    private func fetchPost(_ id: Int64) throws -> Post {
        guard let row = try db.query(
            """
            SELECT p.id, p.actor_id, p.kind, p.parent_id, p.referenced_post_id, p.quote_body,
                   p.body, p.trigger, p.created_at,
                   a.kind AS actor_kind, a.handle, a.display_name, a.avatar_seed, a.bio, a.specialty, a.persona_summary
            FROM posts p
            JOIN actors a ON a.id = p.actor_id
            WHERE p.id = ?
            LIMIT 1
            """,
            values: [.int(id)]
        ).first else {
            throw RuntimeError.notFound("post \(id)")
        }

        let humanId = try humanActor().id
        let likeCount = try db.query(
            "SELECT COUNT(*) AS count FROM likes WHERE post_id = ?",
            values: [.int(id)]
        ).first?.int("count") ?? 0
        let replyCount = try db.query(
            "SELECT COUNT(*) AS count FROM posts WHERE parent_id = ?",
            values: [.int(id)]
        ).first?.int("count") ?? 0
        let repostCount = try db.query(
            "SELECT COUNT(*) AS count FROM posts WHERE kind = 'repost' AND referenced_post_id = ?",
            values: [.int(id)]
        ).first?.int("count") ?? 0
        let likedByYou = try db.query(
            "SELECT 1 AS found FROM likes WHERE actor_id = ? AND post_id = ? LIMIT 1",
            values: [.int(humanId), .int(id)]
        ).isEmpty == false

        return Post(
            id: row.requireInt("id"),
            author: mapActorFromJoinedRow(row),
            kind: PostKind(rawValue: row.requireText("kind")) ?? .original,
            parentId: row.int("parent_id"),
            referencedPostId: row.int("referenced_post_id"),
            quoteBody: row.string("quote_body"),
            body: row.string("body"),
            trigger: TriggerKind(rawValue: row.requireText("trigger")) ?? .manual,
            createdAt: row.requireInt("created_at"),
            likeCount: Int(likeCount),
            replyCount: Int(replyCount),
            repostCount: Int(repostCount),
            likedByYou: likedByYou
        )
    }

    private func mapActor(_ row: [String: SQLiteValue]) -> Actor {
        Actor(
            id: row.requireInt("id"),
            kind: ActorKind(rawValue: row.requireText("kind")) ?? .agent,
            handle: row.requireText("handle"),
            displayName: row.requireText("display_name"),
            avatarSeed: row.string("avatar_seed"),
            bio: row.string("bio") ?? "",
            specialty: row.string("specialty"),
            personaSummary: row.string("persona_summary")
        )
    }

    private func mapActorFromJoinedRow(_ row: [String: SQLiteValue]) -> Actor {
        Actor(
            id: row.requireInt("actor_id"),
            kind: ActorKind(rawValue: row.requireText("actor_kind")) ?? .agent,
            handle: row.requireText("handle"),
            displayName: row.requireText("display_name"),
            avatarSeed: row.string("avatar_seed"),
            bio: row.string("bio") ?? "",
            specialty: row.string("specialty"),
            personaSummary: row.string("persona_summary")
        )
    }

    private func mapNotification(_ row: [String: SQLiteValue]) -> SalonNotification {
        SalonNotification(
            id: row.requireInt("id"),
            kind: SalonNotificationKind(rawValue: row.requireText("kind")) ?? .mention,
            actor: mapActorFromJoinedRow(row),
            postId: row.int("post_id"),
            body: row.string("body"),
            read: (row.int("read") ?? 0) != 0,
            createdAt: row.requireInt("created_at")
        )
    }

    private func mapAgentRun(_ row: [String: SQLiteValue]) -> AgentRun {
        AgentRun(
            id: row.requireInt("id"),
            actorId: row.requireInt("actor_id"),
            actorHandle: row.requireText("handle"),
            actorDisplayName: row.requireText("display_name"),
            trigger: row.requireText("trigger"),
            startedAt: row.requireInt("started_at"),
            finishedAt: row.int("finished_at"),
            promptTokens: row.int("prompt_tokens"),
            completionTokens: row.int("completion_tokens"),
            toolCalls: row.string("tool_calls"),
            error: row.string("error")
        )
    }

    private func mapQueuedTrigger(_ row: [String: SQLiteValue]) -> QueuedTrigger {
        QueuedTrigger(
            id: row.requireInt("id"),
            actorId: row.requireInt("actor_id"),
            actorHandle: row.requireText("handle"),
            trigger: row.requireText("trigger"),
            contextPostId: row.int("context_post_id"),
            dueAt: row.requireInt("due_at"),
            claimedAt: row.int("claimed_at"),
            completedAt: row.int("completed_at")
        )
    }

    private func humanActor() throws -> Actor {
        guard let actor = try actor(byHandle: humanHandle) else {
            throw RuntimeError.notFound("human actor")
        }
        return actor
    }

    private func actorById(_ id: Int64) throws -> Actor {
        guard let row = try db.query(
            """
            SELECT id, kind, handle, display_name, avatar_seed, bio, specialty, persona_summary
            FROM actors
            WHERE id = ?
            LIMIT 1
            """,
            values: [.int(id)]
        ).first else {
            throw RuntimeError.notFound("actor \(id)")
        }
        return mapActor(row)
    }

    private static func now() -> Int64 {
        Int64(Date().timeIntervalSince1970)
    }

    private func normalizedTrigger(_ trigger: String) -> String {
        TriggerKind(rawValue: trigger)?.rawValue ?? TriggerKind.manual.rawValue
    }

    private func specialtyKeywords(for actor: Actor) -> [String] {
        switch actor.handle.lowercased() {
        case "jasmine":
            return ["媒体", "写作", "叙事", "hype", "流行", "内容"]
        case "marc":
            return ["vc", "seed", "pmf", "founder", "投资", "创业", "产品", "增长"]
        case "angel":
            return ["心理", "睡", "健康", "压力", "情绪", "生产力", "习惯"]
        case "mike":
            return ["AI", "agent", "模型", "清华", "姚班", "斯坦福", "eval", "工具调用", "workflow", "创业"]
        case "jasper":
            return ["宏观", "国别", "贸易", "产业", "周期", "汇率", "能源", "大宗", "供应链", "乔治城"]
        case "alex":
            return ["哲学", "法律", "公共", "制度", "哈贝马斯", "国家能力", "国防", "技术中立", "责任"]
        default:
            return actor.specialty?
                .lowercased()
                .split { !$0.isLetter && !$0.isNumber }
                .map(String.init)
                .filter { $0.count >= 2 } ?? []
        }
    }

    private func mapNote(_ row: [String: SQLiteValue]) -> AgentNote {
        AgentNote(
            id: row.requireInt("id"),
            actorId: row.requireInt("actor_id"),
            key: row.requireText("key"),
            content: row.requireText("content"),
            createdAt: row.requireInt("created_at"),
            updatedAt: row.requireInt("updated_at")
        )
    }

    private static let defaultAgentAvatarSeeds: [String: String] = [
        "jasmine": "agent-1",
        "marc": "agent-2",
        "angel": "agent-3",
        "mike": "agent-4",
        "jasper": "agent-5",
        "alex": "agent-6",
    ]
}

enum RuntimeError: LocalizedError {
    case notFound(String)
    case activeRun(Int64)
    case invalidInput(String)

    var errorDescription: String? {
        switch self {
        case .notFound(let value): return "Missing \(value)"
        case .activeRun(let actorId): return "Actor \(actorId) already has an active run."
        case .invalidInput(let message): return message
        }
    }
}

private extension Optional where Wrapped == String {
    var sqlValue: SQLiteValue {
        guard let value = self else { return .null }
        return .text(value)
    }
}

private extension String {
    var sqlValue: SQLiteValue {
        .text(self)
    }

    func prefixString(_ maxLength: Int) -> String {
        String(prefix(maxLength))
    }
}

private extension Optional where Wrapped == Int64 {
    var sqlValue: SQLiteValue {
        guard let value = self else { return .null }
        return .int(value)
    }
}

private extension Dictionary where Key == String, Value == SQLiteValue {
    func string(_ key: String) -> String? {
        self[key]?.stringValue
    }

    func int(_ key: String) -> Int64? {
        self[key]?.intValue
    }

    func requireText(_ key: String) -> String {
        string(key) ?? ""
    }

    func requireInt(_ key: String) -> Int64 {
        int(key) ?? 0
    }
}
