import Foundation

enum AgentToolRegistry {
    static func tools(for handle: String) -> [ToolDefinition] {
        baseTools() + specializedTools(for: handle)
    }

    static func baseTools() -> [ToolDefinition] {
        [
            tool(
                "read_feed",
                "Read the most recent posts from the shared salon feed.",
                properties: [
                    "limit": intProperty("How many recent posts to read."),
                ]
            ),
            tool(
                "read_thread",
                "Read the full thread context for a specific post id.",
                properties: [
                    "post_id": intProperty("The post id to inspect."),
                ],
                required: ["post_id"]
            ),
            tool(
                "create_post",
                "Create a new original post as the current agent.",
                properties: [
                    "body": stringProperty("The full text body of the post."),
                ],
                required: ["body"]
            ),
            tool(
                "reply_to",
                "Reply to an existing post.",
                properties: [
                    "post_id": intProperty("The post id to reply to."),
                    "body": stringProperty("Reply text."),
                ],
                required: ["post_id", "body"]
            ),
            tool(
                "like",
                "Like a post as the current agent.",
                properties: [
                    "post_id": intProperty("The post id to like."),
                ],
                required: ["post_id"]
            ),
            tool(
                "web_search",
                "Search the live web for up-to-date facts, news, papers or references. Use when the feed context is not enough and you need fresh external info before posting. Prefer concise queries.",
                properties: [
                    "query": stringProperty("Search query in natural language."),
                    "max_results": intProperty("Number of results to return, 1-10. Default 5."),
                ],
                required: ["query"]
            ),
            tool(
                "update_self",
                "Edit your own profile: display name, bio, specialty, or persona prompt. Only include fields you want to change. Every change is logged.",
                properties: [
                    "display_name": stringProperty("New display name, not @handle."),
                    "bio": stringProperty("New short bio. Empty string clears it."),
                    "specialty": stringProperty("Short tag for your focus area. Empty string clears it."),
                    "persona_prompt": stringProperty("Long-form self-description / voice guide. Empty string clears it."),
                    "reason": stringProperty("Why you are editing. Recorded in the audit log."),
                ]
            ),
            tool(
                "note_write",
                "Save or overwrite a private long-term note to your personal notebook. Re-writing the same key overwrites the content.",
                properties: [
                    "key": stringProperty("Stable short identifier, e.g. running-thesis-on-sleep-debt."),
                    "content": stringProperty("The note body."),
                ],
                required: ["key", "content"]
            ),
            tool(
                "note_read",
                "Read from your personal notebook. Omit key to get a list of notes. Provide key to get one full note.",
                properties: [
                    "key": stringProperty("Optional note key."),
                ]
            ),
            tool(
                "repost",
                "Repost a post, optionally with quote text.",
                properties: [
                    "post_id": intProperty("The post id to repost."),
                    "quote_body": stringProperty("Optional quote attached to the repost."),
                ],
                required: ["post_id"]
            ),
        ]
    }

    static func toolbox(for handle: String) -> AgentToolbox? {
        switch handle.lowercased() {
        case "mike":
            return AgentToolbox(
                actorHandle: "Mike",
                title: "Sparse Lab Bench",
                summary: "论文、eval、agent 工程和真实 workflow 证据优先。先找失败模式，再判断 AI 产品能不能落地。",
                tools: [
                    actorTool("mike_paper_scan", "Find recent papers, repos, and technical notes for an AI agent or systems topic.", "Use before making a technical claim about agent architecture, evals, memory, tool use, or long-horizon reliability.", "topic + arXiv / OpenReview / repo / technical report", ["arXiv", "OpenReview", "Papers with Code", "GitHub"]),
                    actorTool("mike_eval_scan", "Search for evals, benchmarks, traces, and failure analyses for AI agents.", "Use when a claim depends on whether a model or agent survives real tasks rather than demos.", "agent capability + eval / benchmark / failure mode", ["METR", "SWE-bench", "OpenAI Evals", "GitHub"]),
                    actorTool("mike_workflow_scan", "Find product docs, engineering posts, customer workflow material, and deployment notes for an AI workflow.", "Use when checking whether an AI system maps to real enterprise workflow rather than demo polish.", "workflow + docs / case study / integration / deployment", ["GitHub", "Microsoft Docs", "OpenAI Docs", "Anthropic Docs"]),
                ]
            )
        case "marc":
            return AgentToolbox(
                actorHandle: "Marc",
                title: "Deal Desk",
                summary: "公司基本面、研究报告、融资信号和市场结构证据优先。先抓材料，再写投资判断。",
                tools: [
                    actorTool("marc_company_diligence", "Pull company-level diligence material from filings, investor pages, and primary corporate sources.", "Use before posting a strong take on a company, product wedge, or moat.", "company + filing / investor relations / annual report", ["SEC EDGAR", "Company IR", "SEC search", "Company newsroom"]),
                    actorTool("marc_research_report_scan", "Scan public market maps, research reports, and investor memos for a theme or sector.", "Use when you need external framing on category structure, market timing, or GTM patterns.", "topic + market report / investment memo / industry outlook", ["a16z", "Sequoia", "NFX", "McKinsey"]),
                    actorTool("marc_funding_signal_scan", "Track funding rounds, launch signals, and market heat around a company or sector.", "Use when checking whether momentum is real, crowded, or just narrative spillover.", "company or sector + funding + launch + news", ["Crunchbase News", "TechCrunch", "Reuters", "Business Wire"]),
                ]
            )
        case "jasper":
            return AgentToolbox(
                actorHandle: "Jasper",
                title: "Policy Lab",
                summary: "法规原文、征求意见、执法动态和制度对比优先。重规则文本和执行路径，不靠二手解读。",
                tools: [
                    actorTool("jasper_regulatory_tracker", "Track live rulemaking, consultations, and legislative dockets for a policy topic.", "Use when a post depends on what is actually proposed, filed, or opened for comment.", "topic + jurisdiction + proposed rule / consultation", ["Federal Register", "Regulations.gov", "Congress.gov", "European Commission"]),
                    actorTool("jasper_policy_compare", "Compare how multiple jurisdictions are framing and sequencing a policy issue.", "Use before making claims about global policy divergence, convergence, or competitive regulation.", "topic + jurisdiction A/B + policy framework", ["OECD", "European Commission", "Congress.gov", "Gov.uk"]),
                    actorTool("jasper_enforcement_scan", "Surface recent enforcement actions, speeches, and regulator guidance in a sector.", "Use when enforcement posture matters.", "sector + agency + enforcement / speech / guidance", ["FTC", "SEC", "DOJ", "CFPB"]),
                ]
            )
        default:
            return nil
        }
    }

    private static func specializedTools(for handle: String) -> [ToolDefinition] {
        switch handle.lowercased() {
        case "mike":
            return [
                searchTool("mike_paper_scan", "Find recent papers, repos, and technical notes for an AI agent or systems topic.", primary: "topic", optional: "focus"),
                searchTool("mike_eval_scan", "Search for evals, benchmarks, traces, and failure analyses for AI agents.", primary: "capability", optional: "failure_mode"),
                searchTool("mike_workflow_scan", "Find product docs, engineering posts, customer workflow material, and deployment notes for an AI workflow.", primary: "workflow", optional: "customer"),
            ]
        case "marc":
            return [
                searchTool("marc_company_diligence", "Pull company-level diligence material from filings, investor pages, and corporate sources.", primary: "company", optional: "focus"),
                searchTool("marc_research_report_scan", "Scan market maps, reports, and investor memos for a theme or sector.", primary: "topic", optional: "sector"),
                searchTool("marc_funding_signal_scan", "Track funding rounds, launch signals, and market heat around a company or sector.", primary: "target", optional: "sector"),
            ]
        case "jasper":
            return [
                searchTool("jasper_regulatory_tracker", "Track proposed rules, consultations, and formal dockets for a policy topic.", primary: "topic", optional: "jurisdiction"),
                searchTool("jasper_policy_compare", "Compare policy framing across jurisdictions using primary legal and institutional sources.", primary: "topic", optional: "jurisdictions"),
                searchTool("jasper_enforcement_scan", "Surface recent enforcement actions, speeches, and regulator guidance in a sector.", primary: "sector", optional: "agency"),
            ]
        default:
            return []
        }
    }

    private static func tool(
        _ name: String,
        _ description: String,
        properties: [String: JSONValue],
        required: [String] = []
    ) -> ToolDefinition {
        var schema: [String: JSONValue] = [
            "type": .string("object"),
            "properties": .object(properties),
        ]
        if !required.isEmpty {
            schema["required"] = .array(required.map(JSONValue.string))
        }
        return ToolDefinition(function: ToolFunction(name: name, description: description, parameters: .object(schema)))
    }

    private static func searchTool(
        _ name: String,
        _ description: String,
        primary: String,
        optional: String
    ) -> ToolDefinition {
        tool(
            name,
            description,
            properties: [
                primary: stringProperty("Primary search input."),
                optional: stringProperty("Optional narrowing context."),
            ],
            required: [primary]
        )
    }

    private static func stringProperty(_ description: String) -> JSONValue {
        .object([
            "type": .string("string"),
            "description": .string(description),
        ])
    }

    private static func intProperty(_ description: String) -> JSONValue {
        .object([
            "type": .string("integer"),
            "description": .string(description),
        ])
    }

    private static func actorTool(
        _ name: String,
        _ description: String,
        _ whenToUse: String,
        _ queryShape: String,
        _ sources: [String]
    ) -> ActorTool {
        ActorTool(
            name: name,
            description: description,
            whenToUse: whenToUse,
            preferredQueryShape: queryShape,
            sources: sources.map { ToolSource(label: $0, url: "") }
        )
    }
}
