//! Release-owned consumer resources embedded in the CLI.

use crate::json::Json;

pub const MIGRATION_LINE: &str = "alpha-requirement-scenario";

/// The only earlier line with a supported account rewrite into [`MIGRATION_LINE`].
pub const PREVIOUS_MIGRATION_LINE: &str = "alpha-claim-case";

pub struct WorkflowDescriptor {
    pub id: &'static str,
    pub source: &'static str,
}

pub const WORKFLOWS: &[WorkflowDescriptor] = &[WorkflowDescriptor {
    id: "change-development",
    source: include_str!("../resources/workflows/change-development.json"),
}];

pub const RESOURCE_MANIFEST: &str = include_str!("../resources/manifest.json");
pub const PROJECT_README: &str = include_str!("../resources/templates/init/README.md");
/// PreToolUse hooks that attach a harness-basis descriptor to `azimuth workflow update`.
pub const CLAUDE_CODE_HOOKS: &str =
    include_str!("../resources/integrations/claude-code/settings.json");
pub const CODEX_HOOKS: &str = include_str!("../resources/integrations/codex/hooks.json");
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
        id: "azimuth-workflow",
        source: include_str!("../resources/skills/azimuth-workflow/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-workflow/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-author-proposal",
        source: include_str!("../resources/skills/azimuth-author-proposal/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-author-proposal/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-review-proposal",
        source: include_str!("../resources/skills/azimuth-review-proposal/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-review-proposal/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-author-spec-delta",
        source: include_str!("../resources/skills/azimuth-author-spec-delta/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-author-spec-delta/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-review-spec-delta",
        source: include_str!("../resources/skills/azimuth-review-spec-delta/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-review-spec-delta/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-author-design-delta",
        source: include_str!("../resources/skills/azimuth-author-design-delta/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-author-design-delta/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-review-design-delta",
        source: include_str!("../resources/skills/azimuth-review-design-delta/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-review-design-delta/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-author-verification-delta",
        source: include_str!("../resources/skills/azimuth-author-verification-delta/SKILL.md"),
        openai: include_str!(
            "../resources/skills/azimuth-author-verification-delta/agents/openai.yaml"
        ),
    },
    SkillResource {
        id: "azimuth-review-verification-delta",
        source: include_str!("../resources/skills/azimuth-review-verification-delta/SKILL.md"),
        openai: include_str!(
            "../resources/skills/azimuth-review-verification-delta/agents/openai.yaml"
        ),
    },
    SkillResource {
        id: "azimuth-author-engineering-plan",
        source: include_str!("../resources/skills/azimuth-author-engineering-plan/SKILL.md"),
        openai: include_str!(
            "../resources/skills/azimuth-author-engineering-plan/agents/openai.yaml"
        ),
    },
    SkillResource {
        id: "azimuth-implement-change",
        source: include_str!("../resources/skills/azimuth-implement-change/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-implement-change/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-review-requirement",
        source: include_str!("../resources/skills/azimuth-review-requirement/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-review-requirement/agents/openai.yaml"),
    },
    SkillResource {
        id: "azimuth-author-change-outcome",
        source: include_str!("../resources/skills/azimuth-author-change-outcome/SKILL.md"),
        openai: include_str!(
            "../resources/skills/azimuth-author-change-outcome/agents/openai.yaml"
        ),
    },
    SkillResource {
        id: "azimuth-archive",
        source: include_str!("../resources/skills/azimuth-archive/SKILL.md"),
        openai: include_str!("../resources/skills/azimuth-archive/agents/openai.yaml"),
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
        id: "facet-decisions",
        format_version: 1,
        accepted: &["independently reviewed unchanged facets"],
        rejected: &["arbitrary absent baseline paths"],
        prose: include_str!("../resources/references/facet-decisions.md"),
    },
    ReferenceDescriptor {
        id: "retained-inputs",
        format_version: 1,
        accepted: &["portable native dependency closure"],
        rejected: &["ignored local scratch as durable evidence"],
        prose: include_str!("../resources/references/retained-inputs.md"),
    },
    ReferenceDescriptor {
        id: "authoring-reviews",
        format_version: 1,
        accepted: &["stable authored issues and independent decisions"],
        rejected: &["parsing grants semantic acceptance"],
        prose: include_str!("../resources/references/authoring-reviews.md"),
    },
    ReferenceDescriptor {
        id: "deployment-requirement-status",
        format_version: 2,
        accepted: &[
            "exact release lineage",
            "activity-bound effective contexts",
            "reviewed controls",
            "fresh deployment Monitors",
        ],
        rejected: &[
            "unused baseline capture",
            "model equality implies applicability",
            "unreviewed failure mapping",
            "replay renews freshness",
        ],
        prose: include_str!("../resources/references/deployment-requirement-status.md"),
    },
    ReferenceDescriptor {
        id: "release-publication",
        format_version: 2,
        accepted: &[
            "release built from the model on main",
            "immutable source/build/deployment provenance",
            "portable native records",
            "unavailable supporting evidence",
        ],
        rejected: &[
            "release acceptance",
            "deployment owned by one Change",
            "silent semantic merge",
            "CI invokes an agent",
            "rewritten native history",
        ],
        prose: include_str!("../resources/references/release-publication.md"),
    },
    ReferenceDescriptor {
        id: "assurance-reviews",
        format_version: 2,
        accepted: &[
            "Markdown review bundles",
            "exact review-input packets",
            "native review compilation",
            "local candidate snapshots",
            "independent reviewer provenance",
        ],
        rejected: &[
            "invented reviewer facts",
            "local compilation authenticates identity",
            "compilation publishes to AAS",
        ],
        prose: include_str!("../resources/references/assurance-reviews.md"),
    },
    ReferenceDescriptor {
        id: "workflow",
        format_version: 3,
        accepted: &[
            "pinned declarative definitions",
            "standard change-development workflow",
            "validated target inventory expansion",
            "explicit task graph",
            "read-only next",
            "conditional update",
            "independent reviews",
            "provenance sibling of each result",
            "predecessor-linked restarts",
        ],
        rejected: &[
            "implicit agent execution",
            "file-presence approval",
            "candidate registration publishes to AAS",
            "version 2 instances outside provenance export",
        ],
        prose: include_str!("../resources/references/workflow.md"),
    },
    ReferenceDescriptor {
        id: "work-provenance",
        format_version: 1,
        accepted: &[
            "actor names one execution instance",
            "agent descriptor with per-field basis",
            "committed provenance/ledger.json generated at finalize",
            "append-only Failure Attribution",
            "JSONL dataset",
        ],
        rejected: &[
            "provenance as an assessment input",
            "guessed descriptor values",
            "role or model encoded in actor",
            "hand-edited ledger",
            "telemetry in the ledger",
        ],
        prose: include_str!("../resources/references/work-provenance.md"),
    },
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
            "exact affected Requirement ids",
        ],
        rejected: &[
            "implicit implementation approval",
            "empty completion conditions",
        ],
        prose: include_str!("../resources/references/proposal.md"),
    },
    ReferenceDescriptor {
        id: "intent-delta",
        format_version: 5,
        accepted: &[
            "# Intent delta: <spec-id>",
            "## Add requirement: <id>",
            "### Add scenario: <id>",
            "## Replace requirement: <id>",
            "## Remove requirement: <id>",
            "## Remove scenario: <requirement-id>/<scenario-id>",
            "From: sha256:<digest>",
            "Criticality: routine",
        ],
        rejected: &["Claim", "Case", "rename", "silent Scenario removal"],
        prose: include_str!("../resources/references/intent-delta.md"),
    },
    ReferenceDescriptor {
        id: "design",
        format_version: 4,
        accepted: &[
            "## Requirement: <id>",
            "Mechanism: <id>",
            "Enforcement: type | schema | constraint | choke-point | middleware | guard",
            "Binding: <identity>",
        ],
        rejected: &["path-derived identity", "unscoped speculative mechanism"],
        prose: include_str!("../resources/references/design.md"),
    },
    ReferenceDescriptor {
        id: "verification",
        format_version: 4,
        accepted: &[
            "non-routine Requirement verification only",
            "Decision Policy",
            "Challenge Schedule: current",
        ],
        rejected: &[
            "verification declarations for routine Requirements",
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
        ("0.1.0-alpha.5", "0.1.0-alpha.6") => Some(include_str!(
            "../resources/migrations/0.1.0-alpha.5-to-0.1.0-alpha.6.md"
        )),
        ("0.1.0-alpha.6", "0.1.0-alpha.7") => Some(include_str!(
            "../resources/migrations/0.1.0-alpha.6-to-0.1.0-alpha.7.md"
        )),
        _ => None,
    }
}

pub fn migration_line_reference(from: &str, to: &str) -> Option<&'static str> {
    match (from, to) {
        ("alpha-claim-case", "alpha-requirement-scenario") => Some(include_str!(
            "../resources/migrations/alpha-claim-case-to-alpha-requirement-scenario.md"
        )),
        _ => None,
    }
}
