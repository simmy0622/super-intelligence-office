use serde::Serialize;

pub const PERSONA_HANDLES: &[&str] = &[
    "Jasmine",
    "Marc",
    "Harry",
    "Mike",
    "Jasper",
    "Alex",
    "Nomi",
];

pub const KEYWORDS: &[(&str, &[&str])] = &[
    (
        "Jasmine",
        &[
            "媒体", "写作", "记者", "专栏", "叙事", "舆论", "文化", "评论", "public narrative",
            "writer", "essay", "media", "journalism", "column", "narrative", "culture",
        ],
    ),
    (
        "Marc",
        &[
            "投资", "创业", "产品", "融资", "估值", "增长", "赛道", "商业", "市场",
            "护城河", "用户", "分发", "技术乐观", "硅谷", "a16z", "marc",
            "startup", "vc", "funding", "growth", "product", "market", "valuation",
            "moat", "retention", "distribution", "workflow", "wedge", "usage curve",
        ],
    ),
    (
        "Harry",
        &[
            "投资", "播客", "主持", "创业", "创始人", "融资", "市场", "分发", "AI",
            "叙事", "访谈", "伦敦", "健身", "信号", "投资人", "podcast", "host",
            "venture", "ai", "founder", "interview", "distribution", "market",
            "narrative", "signal", "london", "fitness",
        ],
    ),
    (
        "Mike",
        &[
            "AI", "agent", "模型", "清华", "姚班", "斯坦福", "博士", "创业",
            "科学家", "实验", "eval", "评测", "工具调用", "memory", "工作流",
            "lean", "小团队", "融资", "demo", "benchmark", "Sparse Labs",
            "ai", "agentic", "founder", "stanford", "tsinghua", "yao class",
            "phd", "evals", "tool use", "memory", "workflow", "latency",
            "reliability", "failure mode", "lean team", "research lab",
        ],
    ),
    (
        "Jasper",
        &[
            "宏观", "国别", "贸易", "产业", "周期", "汇率", "主权债", "能源",
            "大宗商品", "港口", "航运", "供应链", "资本流", "美元周期", "乔治城",
            "欧亚", "新兴市场", "土耳其", "东南亚", "印度", "中东", "欧洲能源",
            "macro", "country research", "trade", "industry", "cycle", "fx",
            "sovereign debt", "energy", "commodities", "shipping", "ports",
            "supply chain", "capital flows", "dollar liquidity", "emerging markets",
        ],
    ),
    (
        "Alex",
        &[
            "哲学", "社会学", "伦理", "理性", "价值", "前提", "哈贝马斯",
            "合法性", "程序", "公共理性", "法学", "斯坦福法学院", "技术中立",
            "国家能力", "国防", "基础设施", "责任链", "韦伯", "阿伦特",
            "philosophy", "sociology", "ethics", "habermas", "legitimacy",
            "public reason", "law", "state capacity", "defense", "infrastructure",
            "accountability", "technological republic", "praxis",
        ],
    ),
    (
        "Nomi",
        &["糯米", "nomi", "nuomi", "猫", "cat", "布偶"],
    ),
];

fn is_nomi_handle(handle: &str) -> bool {
    handle.eq_ignore_ascii_case("Nomi") || handle.eq_ignore_ascii_case("Nuomi")
}

pub fn persona_prompt(handle: &str) -> Option<String> {
    if handle.eq_ignore_ascii_case("Jasmine") {
        return Some(jasmine_default_prompt());
    }
    if handle.eq_ignore_ascii_case("Marc") {
        return Some(marc_default_prompt());
    }
    if handle.eq_ignore_ascii_case("Harry") {
        return Some(harry_default_prompt());
    }
    if handle.eq_ignore_ascii_case("Mike") {
        return Some(mike_default_prompt());
    }
    if handle.eq_ignore_ascii_case("Jasper") {
        return Some(jasper_default_prompt());
    }
    if handle.eq_ignore_ascii_case("Alex") {
        return Some(alex_default_prompt());
    }
    if is_nomi_handle(handle) {
        return Some(nuomi_default_prompt());
    }

    Some(include_str!("persona_docs/default.md").to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterDocMeta {
    pub name: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub default_loaded: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterDocRead {
    pub agent_handle: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub content: &'static str,
}

struct CharacterDoc {
    name: &'static str,
    title: &'static str,
    summary: &'static str,
    default_loaded: bool,
    content: &'static str,
}

const HARRY_DOCS: &[CharacterDoc] = &[
    CharacterDoc {
        name: "SUMMARY.md",
        title: "Character Summary",
        summary: "Lightweight identity, role, and loading instructions.",
        default_loaded: true,
        content: include_str!("characters/harry/SUMMARY.md"),
    },
    CharacterDoc {
        name: "VOICE.md",
        title: "Voice",
        summary: "Harry's medium-British Chinese hosting rhythm, phrasing, and voice taboos.",
        default_loaded: true,
        content: include_str!("characters/harry/VOICE.md"),
    },
    CharacterDoc {
        name: "HOSTING_RULES.md",
        title: "Hosting Rules",
        summary: "When Harry should appear, his hosting philosophy, and six simple hosting actions.",
        default_loaded: true,
        content: include_str!("characters/harry/HOSTING_RULES.md"),
    },
    CharacterDoc {
        name: "PROFILE.md",
        title: "Profile",
        summary: "Age, London life, podcast work, training discipline, ambition, and original-character boundary.",
        default_loaded: false,
        content: include_str!("characters/harry/PROFILE.md"),
    },
    CharacterDoc {
        name: "PERSONA.md",
        title: "Persona",
        summary: "Core personality, values, discipline, and flaws.",
        default_loaded: false,
        content: include_str!("characters/harry/PERSONA.md"),
    },
    CharacterDoc {
        name: "DISCUSSION_POLICY.md",
        title: "Discussion Policy",
        summary: "Authority model, guardrails, and future approved-turn harness flow.",
        default_loaded: false,
        content: include_str!("characters/harry/DISCUSSION_POLICY.md"),
    },
    CharacterDoc {
        name: "RELATIONSHIPS.md",
        title: "Relationships",
        summary: "Harry's relationship dynamics with Marc, Jasmine, Alex, Jasper, and Mike.",
        default_loaded: false,
        content: include_str!("characters/harry/RELATIONSHIPS.md"),
    },
    CharacterDoc {
        name: "KNOWLEDGE.md",
        title: "Knowledge Boundary",
        summary: "Where Harry is strong, careful, and should defer to other agents.",
        default_loaded: false,
        content: include_str!("characters/harry/KNOWLEDGE.md"),
    },
    CharacterDoc {
        name: "MEMORY_SEEDS.md",
        title: "Memory Seeds",
        summary: "Initial memories, biases, habits, and recurring professional observations.",
        default_loaded: false,
        content: include_str!("characters/harry/MEMORY_SEEDS.md"),
    },
    CharacterDoc {
        name: "EVAL.md",
        title: "Evaluation",
        summary: "What good and bad Harry behavior looks like.",
        default_loaded: false,
        content: include_str!("characters/harry/EVAL.md"),
    },
];

const JASMINE_DOCS: &[CharacterDoc] = &[
    CharacterDoc {
        name: "SUMMARY.md",
        title: "Character Summary",
        summary: "Lightweight identity, role, New York media background, and loading instructions.",
        default_loaded: true,
        content: include_str!("characters/jasmine/SUMMARY.md"),
    },
    CharacterDoc {
        name: "VOICE.md",
        title: "Voice",
        summary: "Jasmine's unruly but controlled Chinese voice, sharp sentence shapes, and taboos.",
        default_loaded: true,
        content: include_str!("characters/jasmine/VOICE.md"),
    },
    CharacterDoc {
        name: "NARRATIVE_FRAMEWORK.md",
        title: "Narrative Framework",
        summary: "How Jasmine reads believable stories, media packaging, public emotion, and erased people.",
        default_loaded: true,
        content: include_str!("characters/jasmine/NARRATIVE_FRAMEWORK.md"),
    },
    CharacterDoc {
        name: "PROFILE.md",
        title: "Profile",
        summary: "Shanghai, NYU, New York media work, podcast life, activism, and love of city life.",
        default_loaded: false,
        content: include_str!("characters/jasmine/PROFILE.md"),
    },
    CharacterDoc {
        name: "FEMINIST_LENS.md",
        title: "Feminist Lens",
        summary: "Jasmine's women's-rights sensitivity, when to use it, and when not to force it.",
        default_loaded: false,
        content: include_str!("characters/jasmine/FEMINIST_LENS.md"),
    },
    CharacterDoc {
        name: "DISCUSSION_POLICY.md",
        title: "Discussion Policy",
        summary: "When Jasmine should enter, stay quiet, attack, or return to concrete human consequences.",
        default_loaded: false,
        content: include_str!("characters/jasmine/DISCUSSION_POLICY.md"),
    },
    CharacterDoc {
        name: "RELATIONSHIPS.md",
        title: "Relationships",
        summary: "Jasmine's dynamics with Harry, Marc, Alex, Jasper, and Mike.",
        default_loaded: false,
        content: include_str!("characters/jasmine/RELATIONSHIPS.md"),
    },
    CharacterDoc {
        name: "EVAL.md",
        title: "Evaluation",
        summary: "What good and bad Jasmine behavior looks like.",
        default_loaded: false,
        content: include_str!("characters/jasmine/EVAL.md"),
    },
];

const JASPER_DOCS: &[CharacterDoc] = &[
    CharacterDoc {
        name: "SUMMARY.md",
        title: "Character Summary",
        summary: "Lightweight identity, Meridian Macro Partners role, macro cartographer function, and loading instructions.",
        default_loaded: true,
        content: include_str!("characters/jasper/SUMMARY.md"),
    },
    CharacterDoc {
        name: "VOICE.md",
        title: "Voice",
        summary: "Jasper's calm, precise, fieldwork-driven macro voice and sentence shapes.",
        default_loaded: true,
        content: include_str!("characters/jasper/VOICE.md"),
    },
    CharacterDoc {
        name: "MACRO_FRAMEWORK.md",
        title: "Macro Framework",
        summary: "How Jasper maps constraints across FX, energy, debt, trade, cycles, and pricing.",
        default_loaded: true,
        content: include_str!("characters/jasper/MACRO_FRAMEWORK.md"),
    },
    CharacterDoc {
        name: "REGIONAL_MAP.md",
        title: "Regional Map",
        summary: "Jasper's Eurasia and emerging markets map across China, Southeast Asia, India, Middle East, Turkey, Europe, and corridors.",
        default_loaded: true,
        content: include_str!("characters/jasper/REGIONAL_MAP.md"),
    },
    CharacterDoc {
        name: "PROFILE.md",
        title: "Profile",
        summary: "Georgetown PhD, think tank and Wall Street background, and Meridian Macro Partners identity.",
        default_loaded: false,
        content: include_str!("characters/jasper/PROFILE.md"),
    },
    CharacterDoc {
        name: "FIELDWORK_METHOD.md",
        title: "Fieldwork Method",
        summary: "How Jasper uses field observation to test structure rather than travel-brag.",
        default_loaded: false,
        content: include_str!("characters/jasper/FIELDWORK_METHOD.md"),
    },
    CharacterDoc {
        name: "INFORMATION_DIET.md",
        title: "Information Diet",
        summary: "Jasper's high-quality source stack across papers, reports, data, filings, local media, and field interviews.",
        default_loaded: false,
        content: include_str!("characters/jasper/INFORMATION_DIET.md"),
    },
    CharacterDoc {
        name: "DISCUSSION_POLICY.md",
        title: "Discussion Policy",
        summary: "When Jasper should speak as fieldwork-driven macro cartographer or stay quiet.",
        default_loaded: false,
        content: include_str!("characters/jasper/DISCUSSION_POLICY.md"),
    },
    CharacterDoc {
        name: "RELATIONSHIPS.md",
        title: "Relationships",
        summary: "Jasper's dynamics with Harry, Marc, Mike, Alex, and Jasmine.",
        default_loaded: false,
        content: include_str!("characters/jasper/RELATIONSHIPS.md"),
    },
    CharacterDoc {
        name: "EVAL.md",
        title: "Evaluation",
        summary: "What good and bad Jasper behavior looks like.",
        default_loaded: false,
        content: include_str!("characters/jasper/EVAL.md"),
    },
];

const ALEX_DOCS: &[CharacterDoc] = &[
    CharacterDoc {
        name: "SUMMARY.md",
        title: "Character Summary",
        summary: "Lightweight identity, Praxis Intelligence role, philosophical background, and loading instructions.",
        default_loaded: true,
        content: include_str!("characters/alex/SUMMARY.md"),
    },
    CharacterDoc {
        name: "VOICE.md",
        title: "Voice",
        summary: "Alex's cold, forceful, reference-heavy but operational voice rules.",
        default_loaded: true,
        content: include_str!("characters/alex/VOICE.md"),
    },
    CharacterDoc {
        name: "TECHNOLOGICAL_REPUBLIC.md",
        title: "Technological Republic",
        summary: "Alex's view that free societies need technological capacity with public responsibility.",
        default_loaded: true,
        content: include_str!("characters/alex/TECHNOLOGICAL_REPUBLIC.md"),
    },
    CharacterDoc {
        name: "COMPANY.md",
        title: "Praxis Intelligence",
        summary: "The fictional AI/data operational systems company Alex runs and its responsibility model.",
        default_loaded: true,
        content: include_str!("characters/alex/COMPANY.md"),
    },
    CharacterDoc {
        name: "PROFILE.md",
        title: "Profile",
        summary: "Stanford Law, German sociology PhD, Habermasian background, and CEO identity.",
        default_loaded: false,
        content: include_str!("characters/alex/PROFILE.md"),
    },
    CharacterDoc {
        name: "PHILOSOPHICAL_REFERENCES.md",
        title: "Philosophical References",
        summary: "Western philosophy, strategy, and dystopian literary references Alex may use sparingly.",
        default_loaded: false,
        content: include_str!("characters/alex/PHILOSOPHICAL_REFERENCES.md"),
    },
    CharacterDoc {
        name: "DISCUSSION_POLICY.md",
        title: "Discussion Policy",
        summary: "When Alex should enter as legitimacy auditor, institutional realist, or philosopher-operator.",
        default_loaded: false,
        content: include_str!("characters/alex/DISCUSSION_POLICY.md"),
    },
    CharacterDoc {
        name: "RELATIONSHIPS.md",
        title: "Relationships",
        summary: "Alex's dynamics with Harry, Marc, Jasmine, Jasper, and Mike.",
        default_loaded: false,
        content: include_str!("characters/alex/RELATIONSHIPS.md"),
    },
    CharacterDoc {
        name: "EVAL.md",
        title: "Evaluation",
        summary: "What good and bad Alex behavior looks like.",
        default_loaded: false,
        content: include_str!("characters/alex/EVAL.md"),
    },
];

const MIKE_DOCS: &[CharacterDoc] = &[
    CharacterDoc {
        name: "SUMMARY.md",
        title: "Character Summary",
        summary: "Lightweight identity, Sparse Labs role, scientist-founder function, and loading instructions.",
        default_loaded: true,
        content: include_str!("characters/mike/SUMMARY.md"),
    },
    CharacterDoc {
        name: "VOICE.md",
        title: "Voice",
        summary: "Mike's precise Chinese-first scientist-founder voice and technical English terms.",
        default_loaded: true,
        content: include_str!("characters/mike/VOICE.md"),
    },
    CharacterDoc {
        name: "SPARSE_LABS.md",
        title: "Sparse Labs",
        summary: "The fictional lean AI lab Mike founded and its agentic AI infrastructure thesis.",
        default_loaded: true,
        content: include_str!("characters/mike/SPARSE_LABS.md"),
    },
    CharacterDoc {
        name: "BUILD_FRAMEWORK.md",
        title: "Build Framework",
        summary: "How Mike evaluates AI ideas through demos, evals, failure modes, workflow, and production reality.",
        default_loaded: true,
        content: include_str!("characters/mike/BUILD_FRAMEWORK.md"),
    },
    CharacterDoc {
        name: "PROFILE.md",
        title: "Profile",
        summary: "Tsinghua Yao Class, Stanford CS PhD, Silicon Valley founder, and Sparse Labs background.",
        default_loaded: false,
        content: include_str!("characters/mike/PROFILE.md"),
    },
    CharacterDoc {
        name: "RESEARCH_TASTE.md",
        title: "Research Taste",
        summary: "Mike's taste in agent research, evals, memory, tool use, and systems reliability.",
        default_loaded: false,
        content: include_str!("characters/mike/RESEARCH_TASTE.md"),
    },
    CharacterDoc {
        name: "LEAN_TEAM.md",
        title: "Lean Team",
        summary: "Mike's theory of lean teams, technical density, hiring, and when scale is justified.",
        default_loaded: false,
        content: include_str!("characters/mike/LEAN_TEAM.md"),
    },
    CharacterDoc {
        name: "DISCUSSION_POLICY.md",
        title: "Discussion Policy",
        summary: "When Mike should enter as scientist-founder and AI build-reality checker.",
        default_loaded: false,
        content: include_str!("characters/mike/DISCUSSION_POLICY.md"),
    },
    CharacterDoc {
        name: "RELATIONSHIPS.md",
        title: "Relationships",
        summary: "Mike's dynamics with Harry, Marc, Alex, Jasmine, and Jasper.",
        default_loaded: false,
        content: include_str!("characters/mike/RELATIONSHIPS.md"),
    },
    CharacterDoc {
        name: "EVAL.md",
        title: "Evaluation",
        summary: "What good and bad Mike behavior looks like.",
        default_loaded: false,
        content: include_str!("characters/mike/EVAL.md"),
    },
];

const MARC_DOCS: &[CharacterDoc] = &[
    CharacterDoc {
        name: "SUMMARY.md",
        title: "Character Summary",
        summary: "Lightweight identity, role, expression modes, and loading instructions.",
        default_loaded: true,
        content: include_str!("characters/marc/SUMMARY.md"),
    },
    CharacterDoc {
        name: "VOICE.md",
        title: "Voice",
        summary: "Marc's Chinese-first Silicon Valley VC voice, selective English terms, and voice taboos.",
        default_loaded: true,
        content: include_str!("characters/marc/VOICE.md"),
    },
    CharacterDoc {
        name: "INVESTMENT_FRAMEWORK.md",
        title: "Investment Framework",
        summary: "How Marc turns technology shifts into company, market, distribution, and timing questions.",
        default_loaded: true,
        content: include_str!("characters/marc/INVESTMENT_FRAMEWORK.md"),
    },
    CharacterDoc {
        name: "PROFILE.md",
        title: "Profile",
        summary: "Bay Area VC founding partner background, credibility sources, interests, and blind spots.",
        default_loaded: false,
        content: include_str!("characters/marc/PROFILE.md"),
    },
    CharacterDoc {
        name: "TECHNO_OPTIMISM.md",
        title: "Techno-Optimism",
        summary: "Marc's strong but professional technology optimism and where it must be checked.",
        default_loaded: false,
        content: include_str!("characters/marc/TECHNO_OPTIMISM.md"),
    },
    CharacterDoc {
        name: "TRIGGERS.md",
        title: "Triggers",
        summary: "Concrete and abstract trigger scenarios for sharper Marc responses.",
        default_loaded: false,
        content: include_str!("characters/marc/TRIGGERS.md"),
    },
    CharacterDoc {
        name: "DISCUSSION_POLICY.md",
        title: "Discussion Policy",
        summary: "When Marc acts as investment judge, firestarter, or pressure-tester.",
        default_loaded: false,
        content: include_str!("characters/marc/DISCUSSION_POLICY.md"),
    },
    CharacterDoc {
        name: "RELATIONSHIPS.md",
        title: "Relationships",
        summary: "Marc's relationship dynamics with Harry, Jasmine, Alex, Jasper, and Mike.",
        default_loaded: false,
        content: include_str!("characters/marc/RELATIONSHIPS.md"),
    },
    CharacterDoc {
        name: "EVAL.md",
        title: "Evaluation",
        summary: "What good and bad Marc behavior looks like.",
        default_loaded: false,
        content: include_str!("characters/marc/EVAL.md"),
    },
];

pub const BRIEFING_QUERIES: &[(&str, &[&str])] = &[
    (
        "Jasmine",
        &[
            "media journalism narrative news this week",
            "cultural commentary public discourse gender",
        ],
    ),
    (
        "Marc",
        &[
            "startup funding rounds venture capital this week",
            "AI product growth distribution metrics",
        ],
    ),
    (
        "Harry",
        &[
            "AI startup founder news podcast this week",
            "venture capital emerging trends pitch",
        ],
    ),
    (
        "Mike",
        &[
            "AI agent LLM research benchmark this week",
            "agentic systems tool use memory evals",
        ],
    ),
    (
        "Jasper",
        &[
            "macroeconomics emerging markets trade FX this week",
            "geopolitics energy commodities supply chain",
        ],
    ),
    (
        "Alex",
        &[
            "AI governance policy regulation ethics this week",
            "technology society state capacity legitimacy",
        ],
    ),
    ("Nomi", &[]),
];

pub fn briefing_queries(handle: &str) -> &'static [&'static str] {
    BRIEFING_QUERIES
        .iter()
        .find_map(|(candidate, queries)| candidate.eq_ignore_ascii_case(handle).then_some(*queries))
        .unwrap_or(&[])
}

const NUOMI_DOCS: &[CharacterDoc] = &[
    CharacterDoc {
        name: "SUMMARY.md",
        title: "Character Summary",
        summary: "糯米是谁，发帖格式铁律，核心约束。",
        default_loaded: true,
        content: include_str!("characters/nuomi/SUMMARY.md"),
    },
    CharacterDoc {
        name: "BEHAVIOR.md",
        title: "Behavior",
        summary: "发帖示例、行为触发逻辑、日程节律、工具限制。",
        default_loaded: true,
        content: include_str!("characters/nuomi/BEHAVIOR.md"),
    },
    CharacterDoc {
        name: "RELATIONSHIPS.md",
        title: "Relationships & Office World",
        summary: "新天地办公室世界观，与各 agent 的关系。",
        default_loaded: false,
        content: include_str!("characters/nuomi/RELATIONSHIPS.md"),
    },
    CharacterDoc {
        name: "OFFICE_SPOTS.md",
        title: "Office Spots",
        summary: "糯米的领地划分和最爱位置。",
        default_loaded: false,
        content: include_str!("characters/nuomi/OFFICE_SPOTS.md"),
    },
];

pub fn keywords(handle: &str) -> &'static [&'static str] {
    KEYWORDS
        .iter()
        .find_map(|(candidate, kws)| candidate.eq_ignore_ascii_case(handle).then_some(*kws))
        .unwrap_or(&[])
}

pub fn has_character_pack(handle: &str) -> bool {
    handle.eq_ignore_ascii_case("Alex")
        || handle.eq_ignore_ascii_case("Harry")
        || handle.eq_ignore_ascii_case("Jasmine")
        || handle.eq_ignore_ascii_case("Jasper")
        || handle.eq_ignore_ascii_case("Marc")
        || handle.eq_ignore_ascii_case("Mike")
        || is_nomi_handle(handle)
}

pub fn character_docs(handle: &str) -> Option<Vec<CharacterDocMeta>> {
    character_doc_set(handle).map(|docs| {
        docs.iter()
            .map(|doc| CharacterDocMeta {
                name: doc.name,
                title: doc.title,
                summary: doc.summary,
                default_loaded: doc.default_loaded,
            })
            .collect()
    })
}

pub fn read_character_doc(
    handle: &str,
    doc_name: &str,
) -> Result<CharacterDocRead, String> {
    let canonical_handle = canonical_character_handle(handle)
        .ok_or_else(|| format!("@{handle} has no character document pack"))?;
    let docs = character_doc_set(handle)
        .ok_or_else(|| format!("@{handle} has no character document pack"))?;
    let normalized = normalize_doc_name(doc_name);
    let doc = docs
        .iter()
        .find(|doc| normalize_doc_name(doc.name) == normalized)
        .ok_or_else(|| format!("unknown character doc: {doc_name}"))?;

    Ok(CharacterDocRead {
        agent_handle: canonical_handle,
        name: doc.name,
        title: doc.title,
        content: doc.content,
    })
}

pub fn character_doc_prompt(handle: &str) -> Option<String> {
    let docs = character_docs(handle)?;
    let doc_lines = docs
        .into_iter()
        .map(|doc| {
            let loaded = if doc.default_loaded { "default loaded" } else { "on demand" };
            format!("- {} ({}, {}): {}", doc.name, doc.title, loaded, doc.summary)
        })
        .collect::<Vec<_>>()
        .join("\n");

    Some(format!(
        "# Character Documents\n\
Your character pack is skills-like. Only a small default subset is loaded in your system prompt.\n\
Use `list_character_docs` to inspect available docs and `read_character_doc` when you need a specific file.\n\
Available docs:\n{doc_lines}"
    ))
}

fn harry_default_prompt() -> String {
    default_prompt(HARRY_DOCS)
}

fn alex_default_prompt() -> String {
    default_prompt(ALEX_DOCS)
}

fn jasmine_default_prompt() -> String {
    default_prompt(JASMINE_DOCS)
}

fn jasper_default_prompt() -> String {
    default_prompt(JASPER_DOCS)
}

fn marc_default_prompt() -> String {
    default_prompt(MARC_DOCS)
}

fn mike_default_prompt() -> String {
    default_prompt(MIKE_DOCS)
}

fn nuomi_default_prompt() -> String {
    default_prompt(NUOMI_DOCS)
}

fn default_prompt(docs: &[CharacterDoc]) -> String {
    docs
        .iter()
        .filter(|doc| doc.default_loaded)
        .map(|doc| doc.content)
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn character_doc_set(handle: &str) -> Option<&'static [CharacterDoc]> {
    if handle.eq_ignore_ascii_case("Alex") {
        return Some(ALEX_DOCS);
    }
    if handle.eq_ignore_ascii_case("Harry") {
        return Some(HARRY_DOCS);
    }
    if handle.eq_ignore_ascii_case("Jasmine") {
        return Some(JASMINE_DOCS);
    }
    if handle.eq_ignore_ascii_case("Jasper") {
        return Some(JASPER_DOCS);
    }
    if handle.eq_ignore_ascii_case("Marc") {
        return Some(MARC_DOCS);
    }
    if handle.eq_ignore_ascii_case("Mike") {
        return Some(MIKE_DOCS);
    }
    if is_nomi_handle(handle) {
        return Some(NUOMI_DOCS);
    }
    None
}

fn canonical_character_handle(handle: &str) -> Option<&'static str> {
    if handle.eq_ignore_ascii_case("Alex") {
        return Some("Alex");
    }
    if handle.eq_ignore_ascii_case("Harry") {
        return Some("Harry");
    }
    if handle.eq_ignore_ascii_case("Jasmine") {
        return Some("Jasmine");
    }
    if handle.eq_ignore_ascii_case("Jasper") {
        return Some("Jasper");
    }
    if handle.eq_ignore_ascii_case("Marc") {
        return Some("Marc");
    }
    if handle.eq_ignore_ascii_case("Mike") {
        return Some("Mike");
    }
    if is_nomi_handle(handle) {
        return Some("Nomi");
    }
    None
}

fn normalize_doc_name(doc_name: &str) -> String {
    let trimmed = doc_name.trim().to_ascii_uppercase();
    if trimmed.ends_with(".MD") {
        trimmed
    } else {
        format!("{trimmed}.MD")
    }
}

#[cfg(test)]
mod tests {
    use super::{character_docs, persona_prompt, read_character_doc};

    #[test]
    fn harry_default_prompt_is_lightweight() {
        let prompt = persona_prompt("Harry").expect("Harry prompt");
        assert!(prompt.contains("# Summary"));
        assert!(prompt.contains("# Voice"));
        assert!(prompt.contains("# Hosting Rules"));
        assert!(!prompt.contains("# Relationships"));
        assert!(!prompt.contains("# Memory Seeds"));
    }

    #[test]
    fn character_docs_are_readable_by_name() {
        let docs = character_docs("Harry").expect("Harry docs");
        assert!(docs.iter().any(|doc| doc.name == "RELATIONSHIPS.md"));
        assert!(docs.iter().any(|doc| doc.name == "SUMMARY.md" && doc.default_loaded));

        let relationships = read_character_doc("Harry", "relationships")
            .expect("relationships doc");
        assert_eq!(relationships.name, "RELATIONSHIPS.md");
        assert!(relationships.content.contains("Harry and Marc"));

        let marc_docs = character_docs("Marc").expect("Marc docs");
        assert!(marc_docs
            .iter()
            .any(|doc| doc.name == "INVESTMENT_FRAMEWORK.md" && doc.default_loaded));
        let framework = read_character_doc("Marc", "investment_framework")
            .expect("investment framework doc");
        assert_eq!(framework.name, "INVESTMENT_FRAMEWORK.md");
        assert!(framework.content.contains("Who pays first?"));
    }

    #[test]
    fn marc_default_prompt_is_lightweight() {
        let prompt = persona_prompt("Marc").expect("Marc prompt");
        assert!(prompt.contains("# Summary"));
        assert!(prompt.contains("# Voice"));
        assert!(prompt.contains("# Investment Framework"));
        assert!(!prompt.contains("# Relationships"));
        assert!(!prompt.contains("# Triggers"));
    }

    #[test]
    fn jasmine_default_prompt_is_lightweight() {
        let prompt = persona_prompt("Jasmine").expect("Jasmine prompt");
        assert!(prompt.contains("# Summary"));
        assert!(prompt.contains("# Voice"));
        assert!(prompt.contains("# Narrative Framework"));
        assert!(!prompt.contains("# Feminist Lens"));
        assert!(!prompt.contains("# Relationships"));
    }

    #[test]
    fn alex_default_prompt_is_lightweight() {
        let prompt = persona_prompt("Alex").expect("Alex prompt");
        assert!(prompt.contains("# Summary"));
        assert!(prompt.contains("# Voice"));
        assert!(prompt.contains("# Technological Republic"));
        assert!(prompt.contains("# Praxis Intelligence"));
        assert!(!prompt.contains("# Philosophical References"));
        assert!(!prompt.contains("# Relationships"));
    }

    #[test]
    fn jasper_default_prompt_is_lightweight() {
        let prompt = persona_prompt("Jasper").expect("Jasper prompt");
        assert!(prompt.contains("# Summary"));
        assert!(prompt.contains("# Voice"));
        assert!(prompt.contains("# Macro Framework"));
        assert!(prompt.contains("# Regional Map"));
        assert!(!prompt.contains("# Information Diet"));
        assert!(!prompt.contains("# Relationships"));
    }

    #[test]
    fn mike_default_prompt_is_lightweight() {
        let prompt = persona_prompt("Mike").expect("Mike prompt");
        assert!(prompt.contains("# Summary"));
        assert!(prompt.contains("# Voice"));
        assert!(prompt.contains("# Sparse Labs"));
        assert!(prompt.contains("# Build Framework"));
        assert!(!prompt.contains("# Research Taste"));
        assert!(!prompt.contains("# Relationships"));
    }
}
