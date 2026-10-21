use super::{list, object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        ("verification_target", verification_target()),
        (
            "run",
            object(vec![
                ("format", Schema::Text),
                ("version", Schema::Number),
                ("run_id", Schema::Text),
                ("bundle_revision", Schema::Number),
                ("corrects", Schema::Text),
                ("correction_reason", Schema::Text),
                ("bundle_fingerprint", Schema::Text),
                ("verification_target", Schema::Document),
                ("verification_target_fingerprint", Schema::Text),
                ("planned_at_ms", Schema::Number),
                ("started_at_ms", Schema::Number),
                ("finished_at_ms", Schema::Number),
                ("status", Schema::Text),
                (
                    "plan",
                    object(vec![
                        ("model_fingerprint", Schema::Text),
                        ("required_execution_context", Schema::Document),
                        (
                            "checks",
                            list(object(vec![
                                ("id", Schema::Text),
                                ("fingerprint", Schema::Text),
                                ("scenarios", Schema::Document),
                                ("implementations", Schema::Document),
                                ("units", Schema::Document),
                            ])),
                        ),
                        ("challenges", Schema::Document),
                        ("fingerprint", Schema::Text),
                    ]),
                ),
                ("actual_selection", Schema::Document),
                ("provenance", Schema::Document),
                ("artifacts", Schema::Document),
                ("diagnostics", Schema::Document),
                ("activities", Schema::Document),
                ("check_executions", Schema::Document),
                (
                    "verification_challenge_executions",
                    list(object(vec![
                        ("challenge", Schema::Text),
                        ("verification_challenge", Schema::Document),
                        ("target", Schema::Document),
                        ("units", Schema::Document),
                        ("result", Schema::Document),
                    ])),
                ),
                ("contributions", list(Schema::Ref("run"))),
            ]),
        ),
    ]
}

fn verification_target() -> Schema {
    object(vec![
        ("kind", Schema::Text),
        (
            "repositories",
            list(object(vec![
                ("id", Schema::Text),
                ("revision", Schema::Text),
                ("content_fingerprint", Schema::Text),
            ])),
        ),
        (
            "artifacts",
            list(object(vec![("id", Schema::Text), ("digest", Schema::Text)])),
        ),
        ("environment", Schema::Text),
        ("deployment", Schema::Text),
        ("deployment_fingerprint", Schema::Text),
        ("service", Schema::Text),
        (
            "services",
            list(object(vec![
                ("service", Schema::Text),
                ("deployment", Schema::Text),
                ("deployment_fingerprint", Schema::Text),
            ])),
        ),
        ("window_start_ms", Schema::Number),
        ("window_end_ms", Schema::Number),
    ])
}
