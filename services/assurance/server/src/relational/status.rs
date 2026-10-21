use super::{object, Schema};

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
                        ("fields", Schema::Document),
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
                        ("before", Schema::Document),
                        ("after", Schema::Document),
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
