//! Release-owned consumer resources embedded in the CLI.

use crate::json::Json;

pub const MIGRATION_LINE: &str = "alpha-claim-case";
pub const RESOURCE_MANIFEST: &str = include_str!("../resources/manifest.json");
pub const PROJECT_README: &str = include_str!("../resources/templates/init/README.md");
pub const DEFAULT_STANDARDS: &str =
    include_str!("../resources/templates/init/standards-verification.md");
pub const DEFAULT_WORKSPACE: &str = include_str!("../resources/templates/init/workspace.json");
pub const EXPLORATION_TEMPLATE: &str =
    include_str!("../resources/templates/exploration/exploration.md");
pub const PROPOSAL_TEMPLATE: &str = include_str!("../resources/templates/change/proposal.md");
pub const PLAN_TEMPLATE: &str = include_str!("../resources/templates/change/plan.md");

pub struct SkillResource {
    pub id: &'static str,
    pub source: &'static str,
    pub openai: &'static str,
}

pub const SKILLS: &[SkillResource] = &[
    SkillResource {
        id: "azimuth-explore",
        source: include_str!("../resources/skills/azimuth-explore/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-explore/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-propose",
        source: include_str!("../resources/skills/azimuth-propose/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-propose/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-apply",
        source: include_str!("../resources/skills/azimuth-apply/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-apply/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-archive",
        source: include_str!("../resources/skills/azimuth-archive/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-archive/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-coordinate",
        source: include_str!("../resources/skills/azimuth-coordinate/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-coordinate/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-maintain",
        source: include_str!("../resources/skills/azimuth-maintain/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-maintain/agents/openai.yaml"),
    },
];

pub struct ReferenceDescriptor {
    pub id: &'static str,
    pub format_version: u32,
    pub accepted: &'static [&'static str],
    pub rejected: &'static [&'static str],
    pub prose: &'static str,
}

pub const REFERENCES: &[ReferenceDescriptor] = &[
    ReferenceDescriptor {
        id: "exploration",
        format_version: 1,
        accepted: &[
            "exploration.md",
            "Status: exploring | approved",
            "Created: YYYY-MM-DD",
        ],
        rejected: &["dated active id", "implicit archival"],
        prose: include_str!("../resources/references/exploration.md"),
    },
    ReferenceDescriptor {
        id: "proposal",
        format_version: 1,
        accepted: &[
            "Status: proposed",
            "Intent delta: none with Because",
            "exact affected Claim ids",
        ],
        rejected: &[
            "implicit implementation approval",
            "empty completion conditions",
        ],
        prose: include_str!("../resources/references/proposal.md"),
    },
    ReferenceDescriptor {
        id: "intent-delta",
        format_version: 3,
        accepted: &[
            "# Intent delta: <spec-id>",
            "## Add claim: <id>",
            "### Add case: <id>",
            "Criticality: routine",
        ],
        rejected: &[
            "Requirement",
            "Scenario",
            "remove",
            "rename",
            "add Case to existing Claim",
        ],
        prose: include_str!("../resources/references/intent-delta.md"),
    },
    ReferenceDescriptor {
        id: "design",
        format_version: 3,
        accepted: &[
            "## Claim: <id>",
            "Mechanism: <id>",
            "Enforcement: type | schema | constraint | choke-point | middleware | guard",
            "Binding: <identity>",
        ],
        rejected: &["path-derived identity", "unscoped speculative mechanism"],
        prose: include_str!("../resources/references/design.md"),
    },
    ReferenceDescriptor {
        id: "verification",
        format_version: 3,
        accepted: &[
            "non-routine Claim verification only",
            "Decision Policy",
            "Challenge Schedule: current",
        ],
        rejected: &[
            "verification declarations for routine Claims",
            "clean result as positive evidence",
        ],
        prose: include_str!("../resources/references/verification.md"),
    },
    ReferenceDescriptor {
        id: "work-packages",
        format_version: 1,
        accepted: &["Status", "Depends on", "Owns", "Objective", "Evidence"],
        rejected: &["overlapping paths", "cycles", "worker finalization"],
        prose: include_str!("../resources/references/work-packages.md"),
    },
    ReferenceDescriptor {
        id: "outcome",
        format_version: 1,
        accepted: &[
            "delivered behavior",
            "departures",
            "verification performed",
            "residuals",
        ],
        rejected: &[
            "manufactured execution fact",
            "implicit production exposure",
        ],
        prose: include_str!("../resources/references/outcome.md"),
    },
    ReferenceDescriptor {
        id: "migration",
        format_version: 1,
        accepted: &[
            "automatic",
            "review-required",
            "unsupported",
            "content-addressed plan",
        ],
        rejected: &[
            "partial apply",
            "placeholder insertion",
            "historical syntax in normal validation",
        ],
        prose: include_str!("../resources/references/migration.md"),
    },
];

pub fn reference(id: &str) -> Option<&'static ReferenceDescriptor> {
    REFERENCES.iter().find(|item| item.id == id)
}

pub fn reference_json(reference: &ReferenceDescriptor) -> Json {
    Json::obj(vec![
        ("format", Json::str("azimuth-reference")),
        ("schemaVersion", Json::Num(1.0)),
        ("releaseVersion", Json::str(env!("CARGO_PKG_VERSION"))),
        ("migrationLine", Json::str(MIGRATION_LINE)),
        ("id", Json::str(reference.id)),
        (
            "artifactFormatVersion",
            Json::Num(reference.format_version as f64),
        ),
        (
            "accepted",
            Json::Arr(reference.accepted.iter().copied().map(Json::str).collect()),
        ),
        (
            "rejected",
            Json::Arr(reference.rejected.iter().copied().map(Json::str).collect()),
        ),
        ("guidance", Json::str(reference.prose)),
    ])
}

pub fn migration_reference(from: &str, to: &str) -> Option<&'static str> {
    match (from, to) {
        ("0.1.0-alpha.3", "0.1.0-alpha.4") => Some(include_str!(
            "../resources/migrations/0.1.0-alpha.3-to-0.1.0-alpha.4.md"
        )),
        ("0.1.0-alpha.4", "0.1.0-alpha.5") => Some(include_str!(
            "../resources/migrations/0.1.0-alpha.4-to-0.1.0-alpha.5.md"
        )),
        _ => None,
    }
}
