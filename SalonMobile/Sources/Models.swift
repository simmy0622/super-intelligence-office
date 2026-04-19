import Foundation

// MARK: - Actor

enum ActorKind: String, Codable, Equatable, Hashable {
    case human, agent
}

struct Actor: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var kind: ActorKind
    var handle: String
    var displayName: String
    var avatarSeed: String?
    var bio: String
    var specialty: String?
    var personaSummary: String?

    var isAgent: Bool { kind == .agent }
}

struct ActorSummary: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var kind: ActorKind
    var handle: String
    var displayName: String
    var avatarSeed: String?
    var specialty: String?

    var isAgent: Bool { kind == .agent }
}

// MARK: - Post

enum PostKind: String, Codable, Equatable, Hashable {
    case original, reply, repost
}

enum TriggerKind: String, Codable, Equatable, Hashable {
    case scheduled, reactive, manual
    var label: String { rawValue }
}

struct Post: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var author: Actor
    var kind: PostKind
    var parentId: Int64?
    var referencedPostId: Int64?
    var quoteBody: String?
    var body: String?
    var trigger: TriggerKind
    var createdAt: Int64
    var likeCount: Int
    var replyCount: Int
    var repostCount: Int
    var likedByYou: Bool

    var createdDate: Date {
        Date(timeIntervalSince1970: TimeInterval(createdAt))
    }
}

struct PostReference: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var actor: ActorSummary
    var kind: PostKind
    var parentId: Int64?
    var quoteBody: String?
    var body: String?
    var createdAt: Int64
}

// MARK: - Notification

enum SalonNotificationKind: String, Codable, Equatable, Hashable {
    case reply, repost, like, mention

    var displayLabel: String {
        switch self {
        case .reply:   return "回复了你"
        case .repost:  return "转发了你"
        case .like:    return "点赞了你"
        case .mention: return "提到了你"
        }
    }
}

struct SalonNotification: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var kind: SalonNotificationKind
    var actor: Actor
    var postId: Int64?
    var body: String?
    var read: Bool
    var createdAt: Int64
}

// MARK: - Backend runtime support

struct SettingEntry: Identifiable, Equatable, Codable, Hashable {
    var key: String
    var value: String

    var id: String { key }
}

struct AgentRun: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var actorId: Int64
    var actorHandle: String
    var actorDisplayName: String
    var trigger: String
    var startedAt: Int64
    var finishedAt: Int64?
    var promptTokens: Int64?
    var completionTokens: Int64?
    var toolCalls: String?
    var error: String?
}

struct AgentStepResult: Equatable, Codable, Hashable {
    var actorHandle: String
    var trigger: String
    var createdPost: Post?
    var assistantContent: String?
    var reasoningContent: String?
    var toolCalls: [String]
    var promptTokens: Int64?
    var completionTokens: Int64?
}

struct QueuedTrigger: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var actorId: Int64
    var actorHandle: String
    var trigger: String
    var contextPostId: Int64?
    var dueAt: Int64
    var claimedAt: Int64?
    var completedAt: Int64?
}

struct AgentNote: Identifiable, Equatable, Codable, Hashable {
    let id: Int64
    var actorId: Int64
    var key: String
    var content: String
    var createdAt: Int64
    var updatedAt: Int64
}

struct ToolSource: Equatable, Codable, Hashable {
    var label: String
    var url: String
}

struct ActorTool: Identifiable, Equatable, Codable, Hashable {
    var name: String
    var description: String
    var whenToUse: String
    var preferredQueryShape: String
    var sources: [ToolSource]

    var id: String { name }
}

struct AgentToolbox: Equatable, Codable, Hashable {
    var actorHandle: String
    var title: String
    var summary: String
    var tools: [ActorTool]
}
