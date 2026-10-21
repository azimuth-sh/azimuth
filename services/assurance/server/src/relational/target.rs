use super::{object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        (
            "target_observation",
            object(vec![
                ("format", Schema::Text),
                ("version", Schema::Number),
                ("kind", Schema::Text),
                ("source", Schema::Text),
                ("execution_id", Schema::Text),
                ("environment", Schema::Text),
                ("started_at_ms", Schema::Number),
                ("finished_at_ms", Schema::Number),
                ("targets", Schema::Document),
                ("check", Schema::Document),
                ("capture", Schema::Document),
            ]),
        ),
        (
            "target_normalizer",
            object(vec![
                ("description", Schema::Document),
                ("configuration", Schema::Document),
                ("capabilities", Schema::Document),
            ]),
        ),
    ]
}
