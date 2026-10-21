use super::{list, object, Schema};

fn fact() -> Schema {
    object(vec![
        ("kind", Schema::Text),
        ("run", Schema::Text),
        ("bundle_fingerprint", Schema::Text),
        ("fingerprint", Schema::Text),
        ("observation", Schema::Text),
        ("check", Schema::Text),
        ("case", Schema::Text),
        ("observed_at_ms", Schema::Number),
        ("selected_in_basis", Schema::Boolean),
        ("accepted_basis", Schema::Boolean),
        ("diagnostics", list(Schema::Text)),
        ("subject_fingerprint", Schema::Text),
        ("outcome", Schema::Text),
        ("applicable", Schema::Boolean),
        ("component", Schema::Text),
        ("instance", Schema::Text),
    ])
}
fn decision() -> Schema {
    object(vec![
        ("status", Schema::Text),
        ("fingerprint", Schema::Text),
        ("reviewer", Schema::Text),
        ("reviewed_at_ms", Schema::Number),
        ("rationale", Schema::Text),
    ])
}
pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![(
        "assessment_claim",
        object(vec![
            ("claim", Schema::Text),
            ("status", Schema::Text),
            ("review", Schema::Text),
            ("diagnostics", list(Schema::Text)),
            ("gaps", list(Schema::Text)),
            (
                "bindings",
                list(object(vec![
                    ("check", Schema::Text),
                    ("case", Schema::Text),
                    ("status", Schema::Text),
                    (
                        "monitor",
                        object(vec![
                            ("id", Schema::Text),
                            ("interval_seconds", Schema::Number),
                            ("freshness_seconds", Schema::Number),
                        ]),
                    ),
                    ("reasons", list(Schema::Text)),
                ])),
            ),
            ("challenges", list(Schema::Text)),
            (
                "explanation",
                object(vec![
                    ("summary", Schema::Text),
                    ("status", Schema::Text),
                    (
                        "reasons",
                        list(object(vec![
                            ("code", Schema::Text),
                            ("category", Schema::Text),
                            ("message", Schema::Text),
                        ])),
                    ),
                    ("review", decision()),
                    ("health", decision()),
                    (
                        "coverage",
                        list(object(vec![
                            ("check", Schema::Text),
                            ("case", Schema::Text),
                            ("status", Schema::Text),
                            ("observations", list(fact())),
                        ])),
                    ),
                    (
                        "evidence",
                        object(vec![
                            ("supporting", list(fact())),
                            ("violations", list(fact())),
                            ("inconclusive", list(fact())),
                            ("unassessed", list(fact())),
                        ]),
                    ),
                    ("residual_risks", list(Schema::Text)),
                    ("dependencies", list(fact())),
                    ("adverse", list(fact())),
                ]),
            ),
            ("dependencies", list(fact())),
            ("evaluated_at_ms", Schema::Number),
        ]),
    )]
}
