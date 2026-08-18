//! components/aokf_skills.rs — the aokf-carried skill set, generated from
//! `assets/aokf/skills/`: every file of every skill directory. Most derive
//! from mattpocock/skills (MIT — the shipped licence file names them); the
//! rest are superdev's own.

use super::skills::SkillFiles;

macro_rules! asset {
    ($path:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/aokf/skills/",
            $path
        ))
    };
}

/// The MIT notice for the derived skills, shipped beside them.
pub(crate) const LICENSE_FILE: (&str, &str) = (
    "LICENSE-mattpocock-skills.md",
    asset!("LICENSE-mattpocock-skills.md"),
);

/// Every aokf-carried skill, alphabetical, each with every file of its
/// directory as (path relative to the skill directory, content).
pub(crate) const SKILLS: &[SkillFiles] = &[
    (
        "aokf-bootstrap",
        &[("SKILL.md", asset!("aokf-bootstrap/SKILL.md"))],
    ),
    (
        "aokf-maintain",
        &[("SKILL.md", asset!("aokf-maintain/SKILL.md"))],
    ),
    (
        "ask-way",
        &[
            ("PHASE-BOUNDARIES.md", asset!("ask-way/PHASE-BOUNDARIES.md")),
            ("SKILL.md", asset!("ask-way/SKILL.md")),
            ("agents/openai.yaml", asset!("ask-way/agents/openai.yaml")),
        ],
    ),
    (
        "code-review",
        &[
            ("SKILL.md", asset!("code-review/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("code-review/agents/openai.yaml"),
            ),
        ],
    ),
    (
        "codebase-design",
        &[
            ("DEEPENING.md", asset!("codebase-design/DEEPENING.md")),
            (
                "DESIGN-IT-TWICE.md",
                asset!("codebase-design/DESIGN-IT-TWICE.md"),
            ),
            ("SKILL.md", asset!("codebase-design/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("codebase-design/agents/openai.yaml"),
            ),
        ],
    ),
    (
        "diagnosing-bugs",
        &[
            ("SKILL.md", asset!("diagnosing-bugs/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("diagnosing-bugs/agents/openai.yaml"),
            ),
            (
                "scripts/hitl-loop.template.sh",
                asset!("diagnosing-bugs/scripts/hitl-loop.template.sh"),
            ),
        ],
    ),
    (
        "domain-modeling",
        &[
            (
                "DECISION-FORMAT.md",
                asset!("domain-modeling/DECISION-FORMAT.md"),
            ),
            (
                "GLOSSARY-FORMAT.md",
                asset!("domain-modeling/GLOSSARY-FORMAT.md"),
            ),
            ("SKILL.md", asset!("domain-modeling/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("domain-modeling/agents/openai.yaml"),
            ),
        ],
    ),
    (
        "grill-me",
        &[
            ("SKILL.md", asset!("grill-me/SKILL.md")),
            ("agents/openai.yaml", asset!("grill-me/agents/openai.yaml")),
        ],
    ),
    (
        "grilling",
        &[
            ("SKILL.md", asset!("grilling/SKILL.md")),
            ("agents/openai.yaml", asset!("grilling/agents/openai.yaml")),
        ],
    ),
    (
        "handoff",
        &[
            ("SKILL.md", asset!("handoff/SKILL.md")),
            ("agents/openai.yaml", asset!("handoff/agents/openai.yaml")),
        ],
    ),
    (
        "implement",
        &[
            ("SKILL.md", asset!("implement/SKILL.md")),
            ("agents/openai.yaml", asset!("implement/agents/openai.yaml")),
        ],
    ),
    (
        "improve-codebase-architecture",
        &[
            (
                "HTML-REPORT.md",
                asset!("improve-codebase-architecture/HTML-REPORT.md"),
            ),
            ("SKILL.md", asset!("improve-codebase-architecture/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("improve-codebase-architecture/agents/openai.yaml"),
            ),
        ],
    ),
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
    (
        "resolving-merge-conflicts",
        &[
            ("SKILL.md", asset!("resolving-merge-conflicts/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("resolving-merge-conflicts/agents/openai.yaml"),
            ),
        ],
    ),
    (
        "tdd",
        &[
            ("SKILL.md", asset!("tdd/SKILL.md")),
            ("agents/openai.yaml", asset!("tdd/agents/openai.yaml")),
            ("mocking.md", asset!("tdd/mocking.md")),
            ("tests.md", asset!("tdd/tests.md")),
        ],
    ),
    (
        "teach",
        &[
            ("GLOSSARY-FORMAT.md", asset!("teach/GLOSSARY-FORMAT.md")),
            (
                "LEARNING-RECORD-FORMAT.md",
                asset!("teach/LEARNING-RECORD-FORMAT.md"),
            ),
            ("MISSION-FORMAT.md", asset!("teach/MISSION-FORMAT.md")),
            ("RESOURCES-FORMAT.md", asset!("teach/RESOURCES-FORMAT.md")),
            ("SKILL.md", asset!("teach/SKILL.md")),
            ("agents/openai.yaml", asset!("teach/agents/openai.yaml")),
        ],
    ),
    (
        "to-plan",
        &[
            ("ISSUE-FORMAT.md", asset!("to-plan/ISSUE-FORMAT.md")),
            ("PLAN-FORMAT.md", asset!("to-plan/PLAN-FORMAT.md")),
            ("SKILL.md", asset!("to-plan/SKILL.md")),
            ("agents/openai.yaml", asset!("to-plan/agents/openai.yaml")),
        ],
    ),
    (
        "to-questionnaire",
        &[
            ("SKILL.md", asset!("to-questionnaire/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("to-questionnaire/agents/openai.yaml"),
            ),
        ],
    ),
    (
        "to-spec",
        &[
            ("SKILL.md", asset!("to-spec/SKILL.md")),
            ("SPEC-FORMAT.md", asset!("to-spec/SPEC-FORMAT.md")),
            ("agents/openai.yaml", asset!("to-spec/agents/openai.yaml")),
        ],
    ),
    (
        "triage",
        &[
            ("AGENT-BRIEF.md", asset!("triage/AGENT-BRIEF.md")),
            ("SKILL.md", asset!("triage/SKILL.md")),
            ("agents/openai.yaml", asset!("triage/agents/openai.yaml")),
        ],
    ),
    (
        "wait-what",
        &[
            ("SKILL.md", asset!("wait-what/SKILL.md")),
            ("agents/openai.yaml", asset!("wait-what/agents/openai.yaml")),
        ],
    ),
    (
        "wayfinder",
        &[
            ("SKILL.md", asset!("wayfinder/SKILL.md")),
            ("agents/openai.yaml", asset!("wayfinder/agents/openai.yaml")),
        ],
    ),
    (
        "wizard",
        &[
            ("SKILL.md", asset!("wizard/SKILL.md")),
            ("agents/openai.yaml", asset!("wizard/agents/openai.yaml")),
            ("template.sh", asset!("wizard/template.sh")),
        ],
    ),
    (
        "writing-for-agents",
        &[
            (
                "SKILL-MECHANICS.md",
                asset!("writing-for-agents/SKILL-MECHANICS.md"),
            ),
            ("SKILL.md", asset!("writing-for-agents/SKILL.md")),
            (
                "agents/openai.yaml",
                asset!("writing-for-agents/agents/openai.yaml"),
            ),
        ],
    ),
];
