import Foundation

struct ToolExecutionResult {
    var content: String
    var createdPost: Post?
    var record: String
    var engagementDelta: Int
}

@MainActor
final class AgentToolExecutor {
    private let runtime: LocalSalonRuntime
    private let searchProviders: [SearchProvider]

    init(
        runtime: LocalSalonRuntime = .shared,
        searchProviders: [SearchProvider] = [TavilyClient(), ExaClient()]
    ) {
        self.runtime = runtime
        self.searchProviders = searchProviders
    }

    func execute(
        toolCall: ChatToolCall,
        actor: Actor,
        trigger: String,
        alreadyWrotePost: Bool,
        engagementActions: Int
    ) async throws -> ToolExecutionResult {
        let args = try parseArguments(toolCall.function.arguments)
        let name = toolCall.function.name

        switch name {
        case "read_feed":
            let limit = Int(args.int("limit") ?? 10)
            let posts = try runtime.listPosts(limit: max(1, min(limit, 30)))
            return result(name, summarize(posts: posts), engagementDelta: 0)

        case "read_thread":
            let postId = try args.requireInt("post_id")
            let root = try runtime.post(byId: postId)
            let replies = try runtime.replies(to: postId)
            let posts = [root].compactMap { $0 } + replies
            return result(name, summarize(posts: posts), engagementDelta: 0)

        case "create_post":
            guard !alreadyWrotePost else {
                return result(name, "Skipped: this turn already created a post.", engagementDelta: 0)
            }
            let body = try args.requireString("body")
            let post = try runtime.createPostAsActor(actorId: actor.id, body: body, trigger: trigger)
            return result(name, "Created post #\(post?.id ?? 0).", createdPost: post, engagementDelta: 1)

        case "reply_to":
            let postId = try args.requireInt("post_id")
            let body = try args.requireString("body")
            let post = try runtime.replyAsActor(actorId: actor.id, parentId: postId, body: body, trigger: trigger)
            return result(name, "Created reply #\(post?.id ?? 0) to post #\(postId).", createdPost: post, engagementDelta: 1)

        case "like":
            let postId = try args.requireInt("post_id")
            let liked = try runtime.likeAsActor(actorId: actor.id, postId: postId)
            return result(name, liked ? "Liked post #\(postId)." : "Post #\(postId) was already liked.", engagementDelta: liked ? 1 : 0)

        case "repost":
            let postId = try args.requireInt("post_id")
            let quote = args.string("quote_body")
            let post = try runtime.repostAsActor(actorId: actor.id, originalId: postId, quoteBody: quote, trigger: trigger)
            return result(name, "Reposted post #\(postId) as #\(post?.id ?? 0).", createdPost: post, engagementDelta: 1)

        case "web_search":
            let query = try args.requireString("query")
            let maxResults = Int(args.int("max_results") ?? 5)
            let response = try await search(query: query, maxResults: maxResults)
            return result(name, summarize(search: response), engagementDelta: 0)

        case "update_self":
            try runtime.updateActorSelf(
                actorId: actor.id,
                displayName: args.string("display_name"),
                bio: args.string("bio"),
                specialty: args.string("specialty"),
                personaSummary: args.string("persona_prompt")
            )
            return result(name, "Updated @\(actor.handle)'s profile.", engagementDelta: 0)

        case "note_write":
            let key = try args.requireString("key")
            let content = try args.requireString("content")
            let note = try runtime.writeNote(actorId: actor.id, key: key, content: content)
            return result(name, "Wrote note '\(note.key)' for @\(actor.handle).", engagementDelta: 0)

        case "note_read":
            if let key = args.string("key")?.trimmedOrNil() {
                if let note = try runtime.readNote(actorId: actor.id, key: key) {
                    return result(name, "Note \(note.key):\n\(note.content)", engagementDelta: 0)
                }
                return result(name, "No note found for key '\(key)'.", engagementDelta: 0)
            }
            let notes = try runtime.listNotes(actorId: actor.id)
            let text = notes.isEmpty
                ? "No notes."
                : notes.map { "- \($0.key): \($0.content.prefix(120))" }.joined(separator: "\n")
            return result(name, text, engagementDelta: 0)

        default:
            if AgentToolRegistry.tools(for: actor.handle).contains(where: { $0.function.name == name }) {
                let query = specializedSearchQuery(toolName: name, args: args)
                let response = try await search(query: query, maxResults: 5)
                return result(name, summarize(search: response), engagementDelta: 0)
            }
            return result(name, "Unknown tool: \(name)", engagementDelta: 0)
        }
    }

    private func search(query: String, maxResults: Int) async throws -> SearchResponse {
        var errors: [String] = []
        for provider in searchProviders {
            do {
                return try await provider.search(query: query, maxResults: maxResults)
            } catch {
                errors.append(error.localizedDescription)
            }
        }
        throw ToolError.providerFailures(errors.joined(separator: "; "))
    }

    private func parseArguments(_ raw: String) throws -> [String: Any] {
        guard !raw.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return [:] }
        guard
            let data = raw.data(using: .utf8),
            let object = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            throw ToolError.invalidArguments(raw)
        }
        return object
    }

    private func result(
        _ name: String,
        _ content: String,
        createdPost: Post? = nil,
        engagementDelta: Int
    ) -> ToolExecutionResult {
        ToolExecutionResult(
            content: content,
            createdPost: createdPost,
            record: "\(name): \(content.prefix(180))",
            engagementDelta: engagementDelta
        )
    }

    private func summarize(posts: [Post]) -> String {
        if posts.isEmpty { return "No posts found." }
        return posts.map { post in
            let body = post.body ?? post.quoteBody ?? ""
            return "#\(post.id) @\(post.author.handle) [\(post.kind.rawValue), \(post.trigger.rawValue)] \(body)"
        }.joined(separator: "\n")
    }

    private func summarize(search: SearchResponse) -> String {
        var lines = ["Search provider: \(search.provider)", "Query: \(search.query)"]
        if let answer = search.answer, !answer.isEmpty {
            lines.append("Answer: \(answer)")
        }
        lines += search.results.map { "- \($0.title)\n  \($0.url)\n  \($0.snippet)" }
        return lines.joined(separator: "\n")
    }

    private func specializedSearchQuery(toolName: String, args: [String: Any]) -> String {
        let values = args
            .sorted { $0.key < $1.key }
            .compactMap { key, value -> String? in
                guard let text = value as? String, !text.isEmpty else { return nil }
                return "\(key): \(text)"
            }
        return ([toolName] + values).joined(separator: " ")
    }
}

enum ToolError: LocalizedError {
    case invalidArguments(String)
    case missingArgument(String)
    case providerFailures(String)

    var errorDescription: String? {
        switch self {
        case .invalidArguments(let raw): return "Invalid tool arguments: \(raw)"
        case .missingArgument(let name): return "Missing required tool argument: \(name)"
        case .providerFailures(let message): return "All search providers failed: \(message)"
        }
    }
}

private extension Dictionary where Key == String, Value == Any {
    func string(_ key: String) -> String? {
        self[key] as? String
    }

    func int(_ key: String) -> Int64? {
        if let value = self[key] as? Int { return Int64(value) }
        if let value = self[key] as? Int64 { return value }
        if let value = self[key] as? Double { return Int64(value) }
        if let value = self[key] as? String { return Int64(value) }
        return nil
    }

    func requireString(_ key: String) throws -> String {
        guard let value = string(key)?.trimmedOrNil() else {
            throw ToolError.missingArgument(key)
        }
        return value
    }

    func requireInt(_ key: String) throws -> Int64 {
        guard let value = int(key) else {
            throw ToolError.missingArgument(key)
        }
        return value
    }
}
