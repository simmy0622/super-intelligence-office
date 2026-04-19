import Foundation

@MainActor
final class AgentScheduler {
    static let shared = AgentScheduler()

    private let runtime: LocalSalonRuntime
    private let agentRuntime: AgentRuntime
    private let calendar: Calendar

    private let daytimeScheduleSeconds: Int64 = 90 * 60
    private let nightScheduleSeconds: Int64 = 3 * 60 * 60

    init(runtime: LocalSalonRuntime = .shared, agentRuntime: AgentRuntime? = nil) {
        self.runtime = runtime
        self.agentRuntime = agentRuntime ?? AgentRuntime(runtime: runtime)
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(secondsFromGMT: 8 * 3600) ?? .current
        self.calendar = calendar
    }

    func runManual(handle: String) async throws -> AgentStepResult {
        try await agentRuntime.runAgentStep(handle: handle, trigger: TriggerKind.manual.rawValue)
    }

    @discardableResult
    func enqueueReactiveTriggers(contextPostId: Int64) throws -> Int {
        try runtime.enqueueReactiveTriggers(contextPostId: contextPostId)
    }

    @discardableResult
    func enqueueReplyTarget(replyPostId: Int64, parentId: Int64) throws -> Int {
        try runtime.enqueueReplyTarget(replyPostId: replyPostId, parentId: parentId)
    }

    func runSchedulerTick() async throws {
        try await drainQueuePass()
        try await runEngagementPass()
    }

    func drainQueuePass(limit: Int = 16) async throws {
        let pending = try runtime.claimDueTriggers(limit: limit)
        for trigger in pending {
            do {
                _ = try await agentRuntime.runAgentStep(
                    handle: trigger.actorHandle,
                    trigger: trigger.trigger,
                    contextPostId: trigger.contextPostId
                )
            } catch {
                if !isActiveRunError(error) {
                    print("[AgentScheduler] queued trigger failed for @\(trigger.actorHandle): \(error.localizedDescription)")
                }
            }
            try runtime.completeTrigger(trigger.id)
        }
    }

    func runEngagementPass() async throws {
        try await runScheduledRound()
        try await runReactiveRound()
    }

    private func runScheduledRound() async throws {
        let slot = currentScheduleSlot()
        if try runtime.hasSuccessfulRun(trigger: TriggerKind.scheduled.rawValue, since: slot.startTimestamp) {
            return
        }

        let agents = try runtime.listActors()
            .filter(\.isAgent)
            .sorted { $0.id < $1.id }
        guard !agents.isEmpty else { return }

        let actor = agents[Int(slot.index % Int64(agents.count))]
        guard try !runtime.hasActiveRun(actorId: actor.id) else { return }

        let contextPostId = try findScheduledCandidate(for: actor)?.id
        do {
            _ = try await agentRuntime.runAgentStep(
                handle: actor.handle,
                trigger: TriggerKind.scheduled.rawValue,
                contextPostId: contextPostId
            )
        } catch {
            if !isActiveRunError(error) {
                print("[AgentScheduler] scheduled run failed for @\(actor.handle): \(error.localizedDescription)")
            }
        }
    }

    private func runReactiveRound() async throws {
        let cutoff = Self.now() - 30 * 60
        let agents = try runtime.listActors().filter(\.isAgent)

        for actor in agents {
            guard try !runtime.hasActiveRun(actorId: actor.id) else { continue }
            guard let post = try findReactiveCandidate(for: actor, since: cutoff) else { continue }

            do {
                _ = try await agentRuntime.runAgentStep(
                    handle: actor.handle,
                    trigger: TriggerKind.reactive.rawValue,
                    contextPostId: post.id
                )
            } catch {
                if !isActiveRunError(error) {
                    print("[AgentScheduler] reactive run failed for @\(actor.handle) on #\(post.id): \(error.localizedDescription)")
                }
            }
        }
    }

    private func findReactiveCandidate(for actor: Actor, since: Int64) throws -> Post? {
        let keywords = specialtyKeywords(for: actor)
        guard !keywords.isEmpty else { return nil }

        for post in try runtime.recentPosts(since: since, limit: 50) {
            guard post.author.kind == .human, post.author.id != actor.id else { continue }
            guard try !runtime.hasAgentEngaged(actorId: actor.id, postId: post.id) else { continue }
            let text = "\(post.body ?? "") \(post.quoteBody ?? "")".lowercased()
            if keywords.contains(where: { text.contains($0) }) {
                return post
            }
        }

        return nil
    }

    private func findScheduledCandidate(for actor: Actor) throws -> Post? {
        let oneWeekAgo = Self.now() - 7 * 24 * 60 * 60
        let recent = try runtime.recentPosts(since: oneWeekAgo, limit: 120)
        if let humanPost = try recent.first(where: { post in
            guard post.author.kind == .human, post.author.id != actor.id else { return false }
            return try !runtime.hasAgentEngaged(actorId: actor.id, postId: post.id)
        }) {
            return humanPost
        }

        return try recent.first { post in
            guard post.author.id != actor.id else { return false }
            return try !runtime.hasAgentEngaged(actorId: actor.id, postId: post.id)
        }
    }

    private func currentScheduleSlot() -> (startTimestamp: Int64, index: Int64) {
        let now = Date()
        let timestamp = Int64(now.timeIntervalSince1970)
        let components = calendar.dateComponents([.hour, .minute, .second], from: now)
        let hour = Int64(components.hour ?? 0)
        let minute = Int64(components.minute ?? 0)
        let second = Int64(components.second ?? 0)
        let secondsSinceMidnight = hour * 3600 + minute * 60 + second

        let offset: Int64
        let interval: Int64
        if secondsSinceMidnight < 8 * 3600 {
            offset = 0
            interval = nightScheduleSeconds
        } else if secondsSinceMidnight < 22 * 3600 {
            offset = 8 * 3600
            interval = daytimeScheduleSeconds
        } else {
            offset = 22 * 3600
            interval = nightScheduleSeconds
        }

        let bucket = max(0, (secondsSinceMidnight - offset) / interval)
        let elapsedInSlot = secondsSinceMidnight - (offset + bucket * interval)
        let startTimestamp = timestamp - elapsedInSlot
        return (startTimestamp, startTimestamp / interval)
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

    private func isActiveRunError(_ error: Error) -> Bool {
        error.localizedDescription.localizedCaseInsensitiveContains("active run")
    }

    private static func now() -> Int64 {
        Int64(Date().timeIntervalSince1970)
    }
}
