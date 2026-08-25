//! components/aokf_skills.rs — the aokf-carried skill set, generated from
//! `assets/knowledge/skills/`: every file of every skill directory.

use super::skills::SkillFiles;

macro_rules! asset {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/knowledge/skills/",
            $path
        ))
    };
}

/// Every aokf-carried skill, alphabetical, each with every file of its
/// directory as (path relative to the skill directory, content).
pub(crate) const SKILLS: &[SkillFiles] = &[
    ("accept", &[("SKILL.md", asset!("accept/SKILL.md"))]),
    ("adhoc-plan", &[("SKILL.md", asset!("adhoc-plan/SKILL.md"))]),
    ("bootstrap", &[("SKILL.md", asset!("bootstrap/SKILL.md"))]),
    ("brainstorm", &[("SKILL.md", asset!("brainstorm/SKILL.md"))]),
    ("build", &[("SKILL.md", asset!("build/SKILL.md"))]),
    (
        "feature-plan",
        &[("SKILL.md", asset!("feature-plan/SKILL.md"))],
    ),
    ("frame", &[("SKILL.md", asset!("frame/SKILL.md"))]),
    ("grill-me", &[("SKILL.md", asset!("grill-me/SKILL.md"))]),
    (
        "handoff",
        &[
            ("SKILL.md", asset!("handoff/SKILL.md")),
            ("agents/openai.yaml", asset!("handoff/agents/openai.yaml")),
        ],
    ),
    (
        "how-do-i",
        &[
            (
                "SESSION-BOUNDARIES.md",
                asset!("how-do-i/SESSION-BOUNDARIES.md"),
            ),
            ("SKILL.md", asset!("how-do-i/SKILL.md")),
            ("agents/openai.yaml", asset!("how-do-i/agents/openai.yaml")),
        ],
    ),
    ("integrate", &[("SKILL.md", asset!("integrate/SKILL.md"))]),
    (
        "interface-design",
        &[("SKILL.md", asset!("interface-design/SKILL.md"))],
    ),
    ("maintain", &[("SKILL.md", asset!("maintain/SKILL.md"))]),
    (
        "prototype",
        &[
            ("LOGIC.md", asset!("prototype/LOGIC.md")),
            ("SKILL.md", asset!("prototype/SKILL.md")),
            ("UI.md", asset!("prototype/UI.md")),
            ("agents/openai.yaml", asset!("prototype/agents/openai.yaml")),
        ],
    ),
    (
        "research",
        &[
            ("SKILL.md", asset!("research/SKILL.md")),
            ("agents/openai.yaml", asset!("research/agents/openai.yaml")),
        ],
    ),
    ("spec", &[("SKILL.md", asset!("spec/SKILL.md"))]),
    ("verify", &[("SKILL.md", asset!("verify/SKILL.md"))]),
];
