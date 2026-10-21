use super::{map, object, Schema};

pub(super) fn snapshot() -> Schema {
    object(vec![
        ("captured_at_ms", Schema::Number),
        ("instance", Schema::Text),
        (
            "report",
            object(vec![
                ("format", Schema::Text),
                ("version", Schema::Number),
                (
                    "contexts",
                    map(object(vec![
                        ("contract_revision", Schema::Text),
                        (
                            "values",
                            map(Schema::OneOf(vec![
                                Schema::Text,
                                Schema::Boolean,
                                Schema::Number,
                            ])),
                        ),
                    ])),
                ),
            ]),
        ),
    ])
}

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        (
            "context_contract",
            object(vec![
                (
                    "contract",
                    object(vec![
                        ("name", Schema::Text),
                        ("fingerprint", Schema::Text),
                        ("fields", map(Schema::Text)),
                    ]),
                ),
                ("registered_by", Schema::Text),
                ("registered_at_ms", Schema::Number),
            ]),
        ),
        (
            "context_capture",
            object(vec![
                ("fingerprint", Schema::Text),
                (
                    "capture",
                    object(vec![
                        ("kind", Schema::Text),
                        ("run", Schema::Text),
                        ("activity", Schema::Text),
                        ("bundle_fingerprint", Schema::Text),
                        ("verification_target_fingerprint", Schema::Text),
                        ("component", Schema::Text),
                        ("instance", Schema::Text),
                        ("started_at_ms", Schema::Number),
                        ("finished_at_ms", Schema::Number),
                        ("before", snapshot()),
                        ("after", snapshot()),
                        ("trusted", Schema::Boolean),
                    ]),
                ),
                ("submitted_by", Schema::Text),
                ("received_at_ms", Schema::Number),
            ]),
        ),
        (
            "run_attribution",
            object(vec![
                ("run", Schema::Text),
                ("bundle_fingerprint", Schema::Text),
                ("submitted_by", Schema::Text),
                ("received_at_ms", Schema::Number),
            ]),
        ),
    ]
}
