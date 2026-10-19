use super::{list, map, object, Schema};

fn subject() -> Schema {
    object(vec![
        ("projectSnapshot", Schema::Text),
        ("revision", Schema::Text),
        ("artifactDigest", Schema::Text),
        ("deploymentId", Schema::Text),
        ("environment", Schema::Text),
        ("cohort", Schema::Text),
    ])
}

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    let reference = object(vec![
        ("spec", Schema::Text),
        ("claim", Schema::Text),
        ("contractFingerprint", Schema::Text),
    ]);
    let contribution = object(vec![
        ("area", Schema::Text),
        ("mount", Schema::Text),
        ("path", Schema::Text),
        ("enumerator", Schema::Text),
    ]);
    let area = object(vec![
        ("id", Schema::Text),
        (
            "mounts",
            list(object(vec![("id", Schema::Text), ("path", Schema::Text)])),
        ),
    ]);
    let contract = object(vec![
        ("contractFingerprint", Schema::Text),
        ("spec", Schema::Text),
        ("claim", Schema::Text),
        ("requirement", Schema::Text),
        ("criticality", Schema::Text),
        ("statement", Schema::Text),
        ("caseStatement", Schema::Text),
        ("domain", Schema::Text),
        (
            "verification",
            object(vec![
                ("strength", Schema::Text),
                ("scope", Schema::Text),
                ("quantification", Schema::Text),
                ("oracle", Schema::Text),
                ("residualRequired", Schema::Boolean),
                ("residual", Schema::Text),
                ("residualAcceptance", Schema::Text),
            ]),
        ),
        (
            "surface",
            object(vec![
                ("id", Schema::Text),
                ("contributions", list(contribution)),
            ]),
        ),
        ("obligatedAreas", list(area)),
    ]);
    vec![
        (
            "project_snapshot",
            object(vec![
                ("format", Schema::Text),
                ("version", Schema::Number),
                ("id", Schema::Text),
                ("project", Schema::Text),
                ("modelFingerprint", Schema::Text),
                ("claims", list(contract)),
            ]),
        ),
        (
            "definition",
            object(vec![
                ("id", Schema::Text),
                ("claim", reference),
                ("assertion", Schema::Text),
                ("scope", Schema::Text),
                ("quantification", Schema::Text),
                ("oracle", Schema::Text),
                ("stage", Schema::Text),
                ("inputs", list(Schema::Text)),
                ("requiredContext", map(Schema::Text)),
                ("declaredAt", Schema::Number),
            ]),
        ),
        (
            "qualification",
            object(vec![
                ("id", Schema::Text),
                ("definitionId", Schema::Text),
                ("definitionFingerprint", Schema::Text),
                ("verdict", Schema::Text),
                ("qualifiedAt", Schema::Number),
                ("rationale", Schema::Text),
            ]),
        ),
        (
            "observation",
            object(vec![
                ("id", Schema::Text),
                ("definitionId", Schema::Text),
                ("definitionFingerprint", Schema::Text),
                ("stage", Schema::Text),
                ("subject", subject()),
                ("context", map(Schema::Text)),
                ("observedAt", Schema::Number),
                ("expiresAt", Schema::Number),
                ("outcome", Schema::Text),
                ("report", Schema::Text),
            ]),
        ),
        (
            "challenge",
            object(vec![
                ("id", Schema::Text),
                ("source", Schema::Text),
                ("definitionId", Schema::Text),
                ("definitionFingerprint", Schema::Text),
                ("observedAt", Schema::Number),
                ("outcome", Schema::Text),
                ("report", Schema::Text),
            ]),
        ),
        (
            "gate_request",
            object(vec![
                ("definitionId", Schema::Text),
                ("stage", Schema::Text),
                ("subject", subject()),
                ("at", Schema::Number),
            ]),
        ),
        (
            "gate_decision",
            object(vec![
                ("status", Schema::Text),
                ("definitionFingerprint", Schema::Text),
                ("qualificationId", Schema::Text),
                ("observationId", Schema::Text),
                ("reasons", list(Schema::Text)),
                ("work", list(Schema::Text)),
            ]),
        ),
    ]
}
