import Foundation

@MainActor
final class ProfileService {
    private let runtime: LocalSalonRuntime

    init(runtime: LocalSalonRuntime) {
        self.runtime = runtime
    }

    func listActors() throws -> [Actor] {
        try runtime.listActors()
    }

    func actor(byHandle handle: String) throws -> Actor? {
        try runtime.actor(byHandle: handle)
    }

    func updateAvatar(actorId: Int64, avatarSeed: String?) throws {
        try runtime.updateActorAvatar(actorId: actorId, avatarSeed: avatarSeed)
    }
}

@MainActor
final class FeedService {
    private let runtime: LocalSalonRuntime

    init(runtime: LocalSalonRuntime) {
        self.runtime = runtime
    }

    func listPosts(limit: Int = 100) throws -> [Post] {
        try runtime.listPosts(limit: limit)
    }

    func post(byId id: Int64) throws -> Post? {
        try runtime.post(byId: id)
    }

    func replies(to postId: Int64) throws -> [Post] {
        try runtime.replies(to: postId)
    }

    func posts(by actorId: Int64) throws -> [Post] {
        try runtime.posts(by: actorId)
    }
}

@MainActor
final class InteractionService {
    private let runtime: LocalSalonRuntime

    init(runtime: LocalSalonRuntime) {
        self.runtime = runtime
    }

    @discardableResult
    func createPost(body: String) throws -> Post? {
        try runtime.createHumanPost(body: body)
    }

    @discardableResult
    func reply(to parentId: Int64, body: String) throws -> Post? {
        try runtime.replyAsHuman(parentId: parentId, body: body)
    }

    @discardableResult
    func repost(_ originalId: Int64, quoteBody: String? = nil) throws -> Post? {
        try runtime.repostAsHuman(originalId: originalId, quoteBody: quoteBody)
    }

    @discardableResult
    func toggleLike(postId: Int64) throws -> Bool {
        try runtime.toggleLike(postId: postId)
    }
}

@MainActor
final class NotificationService {
    private let runtime: LocalSalonRuntime

    init(runtime: LocalSalonRuntime) {
        self.runtime = runtime
    }

    func listNotifications(limit: Int = 50) throws -> [SalonNotification] {
        try runtime.listNotifications(limit: limit)
    }

    func markAllRead() throws {
        try runtime.markAllNotificationsRead()
    }
}

@MainActor
final class RuntimeDiagnosticsService {
    private let runtime: LocalSalonRuntime

    init(runtime: LocalSalonRuntime) {
        self.runtime = runtime
    }

    func listAgentRuns(limit: Int = 50) throws -> [AgentRun] {
        try runtime.listAgentRuns(limit: limit)
    }
}

@MainActor
final class LocalSalonServiceContainer {
    let profile: ProfileService
    let feed: FeedService
    let interaction: InteractionService
    let notifications: NotificationService
    let diagnostics: RuntimeDiagnosticsService

    init(runtime: LocalSalonRuntime = .shared) {
        self.profile = ProfileService(runtime: runtime)
        self.feed = FeedService(runtime: runtime)
        self.interaction = InteractionService(runtime: runtime)
        self.notifications = NotificationService(runtime: runtime)
        self.diagnostics = RuntimeDiagnosticsService(runtime: runtime)
    }
}
