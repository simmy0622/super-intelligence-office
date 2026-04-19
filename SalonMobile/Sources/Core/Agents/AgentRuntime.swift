import Foundation

@MainActor
final class AgentRuntime {
    private let runtime: LocalSalonRuntime
    private let llm: LLMProvider
    private let toolExecutor: AgentToolExecutor

    private let maxToolRounds = 4

    init(
        runtime: LocalSalonRuntime = .shared,
        llm: LLMProvider = DeepSeekClient(),
        toolExecutor: AgentToolExecutor? = nil
    ) {
        self.runtime = runtime
        self.llm = llm
        self.toolExecutor = toolExecutor ?? AgentToolExecutor(runtime: runtime)
    }

    func runAgentStep(
        handle: String,
        trigger: String = TriggerKind.manual.rawValue,
        contextPostId: Int64? = nil
    ) async throws -> AgentStepResult {
        guard let actor = try runtime.actor(byHandle: handle), actor.isAgent else {
            throw AgentRuntimeError.notAgent(handle)
        }

        let runId = try runtime.createAgentRun(actorId: actor.id, trigger: trigger)

        do {
            let result = try await runAgentStepInner(actor: actor, trigger: trigger, contextPostId: contextPostId)
            let toolCallsJSON = String(data: try JSONEncoder().encode(result.toolCalls), encoding: .utf8)
            try runtime.finishAgentRun(
                runId: runId,
                promptTokens: result.promptTokens,
                completionTokens: result.completionTokens,
                toolCalls: toolCallsJSON,
                error: nil
            )

            if let post = result.createdPost, let kind = try notificationKindIfUserTargeted(post) {
                try? runtime.createNotification(
                    kind: kind,
                    actorId: actor.id,
                    postId: post.id,
                    body: post.body ?? post.quoteBody
                )
            }

            return result
        } catch {
            try? runtime.finishAgentRun(
                runId: runId,
                promptTokens: nil,
                completionTokens: nil,
                toolCalls: nil,
                error: error.localizedDescription
            )
            throw error
        }
    }

    private func runAgentStepInner(
        actor: Actor,
        trigger: String,
        contextPostId: Int64?
    ) async throws -> AgentStepResult {
        var messages: [ChatMessage] = [
            .system(try systemPrompt(for: actor)),
            .user(userPrompt(trigger: trigger, contextPostId: contextPostId)),
        ]

        let tools = AgentToolRegistry.tools(for: actor.handle)
        var createdPost: Post?
        var toolRecords: [String] = []
        var engagementActions = 0
        var promptTokens: Int64 = 0
        var completionTokens: Int64 = 0
        var finalContent: String?
        var finalReasoning: String?

        for _ in 0..<maxToolRounds {
            let response = try await llm.chatCompletion(ChatRequest(
                model: DeepSeekClient.defaultModel,
                messages: messages,
                tools: tools,
                maxTokens: 4096
            ))

            if let usage = response.usage {
                promptTokens += usage.promptTokens
                completionTokens += usage.completionTokens
            }

            guard let assistant = response.choices.first?.message else {
                throw AgentRuntimeError.emptyLLMResponse
            }

            finalContent = assistant.content
            finalReasoning = assistant.reasoningContent
            messages.append(assistant)

            guard let toolCalls = assistant.toolCalls, !toolCalls.isEmpty else {
                break
            }

            for toolCall in toolCalls {
                let execution = try await toolExecutor.execute(
                    toolCall: toolCall,
                    actor: actor,
                    trigger: trigger,
                    alreadyWrotePost: createdPost != nil,
                    engagementActions: engagementActions
                )

                if createdPost == nil {
                    createdPost = execution.createdPost
                }
                engagementActions += execution.engagementDelta
                toolRecords.append(execution.record)
                messages.append(.tool(id: toolCall.id, content: execution.content))
            }
        }

        if isWakeTrigger(trigger), engagementActions == 0 {
            throw AgentRuntimeError.noEngagement(actor.handle, trigger)
        }

        return AgentStepResult(
            actorHandle: actor.handle,
            trigger: trigger,
            createdPost: createdPost,
            assistantContent: finalContent,
            reasoningContent: finalReasoning,
            toolCalls: toolRecords,
            promptTokens: promptTokens,
            completionTokens: completionTokens
        )
    }

    private func systemPrompt(for actor: Actor) throws -> String {
        let recentFeed = try runtime.listPosts(limit: 12)
            .map { "#\($0.id) @\($0.author.handle): \($0.body ?? $0.quoteBody ?? "")" }
            .joined(separator: "\n")
        let notes = try runtime.listNotes(actorId: actor.id)
            .prefix(8)
            .map { "- \($0.key): \($0.content)" }
            .joined(separator: "\n")

        return """
        You are @\(actor.handle) (\(actor.displayName)) in Agent Salon.

        Bio: \(actor.bio)
        Specialty: \(actor.specialty ?? "none")
        Persona: \(actor.personaSummary ?? "Stay concise, useful, and distinct.")

        You participate in a shared X-style salon. Use tools to read context and take actions.
        Prefer one high-quality engagement action per turn: create_post, reply_to, repost, or like.
        Do not claim facts requiring current information unless you use web_search or a specialized search tool.

        Recent feed:
        \(recentFeed.isEmpty ? "No recent posts." : recentFeed)

        Private notes:
        \(notes.isEmpty ? "No notes." : notes)
        """
    }

    private func userPrompt(trigger: String, contextPostId: Int64?) -> String {
        if let contextPostId {
            return "Trigger: \(trigger). Consider post #\(contextPostId). Read context first, then decide whether to reply, like, repost, or write a new post."
        }
        return "Trigger: \(trigger). Read the salon context and take one useful action if appropriate."
    }

    private func isWakeTrigger(_ trigger: String) -> Bool {
        trigger == TriggerKind.scheduled.rawValue || trigger == TriggerKind.reactive.rawValue
    }

    private func notificationKindIfUserTargeted(_ post: Post) throws -> String? {
        switch post.kind {
        case .reply:
            guard let parentId = post.parentId,
                  let parent = try runtime.post(byId: parentId),
                  parent.author.kind == .human
            else {
                return nil
            }
            return "reply"
        case .repost:
            guard let referencedPostId = post.referencedPostId,
                  let referenced = try runtime.post(byId: referencedPostId),
                  referenced.author.kind == .human
            else {
                return nil
            }
            return "repost"
        case .original:
            return nil
        }
    }
}

enum AgentRuntimeError: LocalizedError {
    case notAgent(String)
    case emptyLLMResponse
    case noEngagement(String, String)

    var errorDescription: String? {
        switch self {
        case .notAgent(let handle): return "@\(handle) is not an agent."
        case .emptyLLMResponse: return "The LLM returned no choices."
        case .noEngagement(let handle, let trigger): return "\(trigger) wake for @\(handle) finished without engagement."
        }
    }
}
