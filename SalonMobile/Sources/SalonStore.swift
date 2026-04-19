import Foundation
import SwiftUI

@MainActor
final class SalonStore: ObservableObject {
    @Published private(set) var actors: [Actor] = []
    @Published private(set) var feed: [Post] = []
    @Published private(set) var notifications: [SalonNotification] = []
    @Published private(set) var agentRuns: [AgentRun] = []
    @Published private(set) var lastError: String?
    @Published private(set) var lastAgentResult: AgentStepResult?
    @Published private(set) var runningAgentHandles: Set<String> = []
    @Published private(set) var isSchedulerRunning = false
    @Published var humanHandle: String = "you"

    private let services: LocalSalonServiceContainer
    private let scheduler: AgentScheduler

    var currentUser: Actor {
        actor(byHandle: humanHandle) ?? Self.fallbackHuman
    }

    var unreadNotificationCount: Int {
        notifications.filter { !$0.read }.count
    }

    init(
        services: LocalSalonServiceContainer = LocalSalonServiceContainer(),
        scheduler: AgentScheduler = .shared
    ) {
        self.services = services
        self.scheduler = scheduler
        refresh()
    }

    // MARK: Lookups

    func actor(byHandle handle: String) -> Actor? {
        actors.first { $0.handle.lowercased() == handle.lowercased() }
    }

    func actor(byId id: Int64) -> Actor? {
        actors.first { $0.id == id }
    }

    func post(byId id: Int64) -> Post? {
        feed.first { $0.id == id } ?? (try? services.feed.post(byId: id))
    }

    func replies(to postId: Int64) -> [Post] {
        do {
            return try services.feed.replies(to: postId)
        } catch {
            publish(error)
            return feed.filter { $0.parentId == postId }
        }
    }

    func posts(by actorId: Int64) -> [Post] {
        do {
            return try services.feed.posts(by: actorId)
        } catch {
            publish(error)
            return feed.filter { $0.author.id == actorId }
        }
    }

    // MARK: Mutations

    func toggleLike(postId: Int64) {
        do {
            let liked = try services.interaction.toggleLike(postId: postId)
            updateCachedPost(postId) { post in
                post.likedByYou = liked
                post.likeCount = max(0, post.likeCount + (liked ? 1 : -1))
            }
            refreshFeed()
        } catch {
            publish(error)
        }
    }

    func createPost(body: String) async throws {
        let post = try services.interaction.createPost(body: body.trimmingCharacters(in: .whitespacesAndNewlines))
        refreshFeed()
        if let post {
            enqueueReactiveWork(contextPostId: post.id)
        }
    }

    func reply(to parentId: Int64, body: String) async throws {
        let post = try services.interaction.reply(to: parentId, body: body.trimmingCharacters(in: .whitespacesAndNewlines))
        refreshFeed()
        refreshNotifications()
        if let post {
            enqueueReplyWork(replyPostId: post.id, parentId: parentId)
        }
    }

    func repost(_ originalId: Int64, quoteBody: String? = nil) {
        do {
            _ = try services.interaction.repost(originalId, quoteBody: quoteBody?.trimmedOrNil())
            refreshFeed()
        } catch {
            publish(error)
        }
    }

    func markAllNotificationsRead() {
        do {
            try services.notifications.markAllRead()
            refreshNotifications()
        } catch {
            publish(error)
        }
    }

    func refresh() {
        refreshActors()
        refreshFeed()
        refreshNotifications()
        refreshAgentRuns()
    }

    func isAgentRunning(_ handle: String) -> Bool {
        runningAgentHandles.contains(handle.lowercased())
    }

    func runAgent(handle: String) {
        let normalized = handle.lowercased()
        guard !runningAgentHandles.contains(normalized) else { return }

        Task { @MainActor in
            runningAgentHandles.insert(normalized)
            defer { runningAgentHandles.remove(normalized) }

            do {
                lastAgentResult = try await scheduler.runManual(handle: handle)
                refresh()
            } catch {
                publish(error)
            }
        }
    }

    func runSchedulerTick() {
        guard !isSchedulerRunning else { return }

        Task { @MainActor in
            isSchedulerRunning = true
            defer { isSchedulerRunning = false }

            do {
                try await scheduler.runSchedulerTick()
                refresh()
            } catch {
                publish(error)
            }
        }
    }

    private func refreshActors() {
        do {
            actors = try services.profile.listActors()
            lastError = nil
        } catch {
            publish(error)
        }
    }

    private func refreshFeed() {
        do {
            feed = try services.feed.listPosts()
            lastError = nil
        } catch {
            publish(error)
        }
    }

    private func refreshNotifications() {
        do {
            notifications = try services.notifications.listNotifications()
            lastError = nil
        } catch {
            publish(error)
        }
    }

    func refreshAgentRuns() {
        do {
            agentRuns = try services.diagnostics.listAgentRuns()
            lastError = nil
        } catch {
            publish(error)
        }
    }

    func updateCurrentUserAvatar(imageData: Data) {
        do {
            let avatarSeed = try AvatarStorage.saveUserAvatar(imageData)
            try services.profile.updateAvatar(actorId: currentUser.id, avatarSeed: avatarSeed)
            refreshActors()
        } catch {
            publish(error)
        }
    }

    func removeCurrentUserAvatar() {
        do {
            try AvatarStorage.deleteUserAvatar()
            try services.profile.updateAvatar(actorId: currentUser.id, avatarSeed: nil)
            refreshActors()
        } catch {
            publish(error)
        }
    }

    private func updateCachedPost(_ postId: Int64, transform: (inout Post) -> Void) {
        guard let index = feed.firstIndex(where: { $0.id == postId }) else { return }
        transform(&feed[index])
    }

    private func enqueueReactiveWork(contextPostId: Int64) {
        Task { @MainActor in
            do {
                _ = try scheduler.enqueueReactiveTriggers(contextPostId: contextPostId)
                try await scheduler.drainQueuePass()
                refresh()
            } catch {
                publish(error)
            }
        }
    }

    private func enqueueReplyWork(replyPostId: Int64, parentId: Int64) {
        Task { @MainActor in
            do {
                _ = try scheduler.enqueueReplyTarget(replyPostId: replyPostId, parentId: parentId)
                _ = try scheduler.enqueueReactiveTriggers(contextPostId: replyPostId)
                try await scheduler.drainQueuePass()
                refresh()
            } catch {
                publish(error)
            }
        }
    }

    private func publish(_ error: Error) {
        lastError = error.localizedDescription
    }

    private static let fallbackHuman = Actor(
        id: 1,
        kind: .human,
        handle: "you",
        displayName: "You",
        avatarSeed: nil,
        bio: "刚加入 salon。",
        specialty: nil,
        personaSummary: nil
    )
}
