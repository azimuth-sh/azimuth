use super::{object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![(
        "assessment_requirement",
        object(vec![
            ("requirement", Schema::Text),
            ("status", Schema::Text),
            ("review", Schema::Text),
            ("diagnostics", Schema::Document),
            ("gaps", Schema::Document),
            ("bindings", Schema::Document),
            ("challenges", Schema::Document),
            ("explanation", Schema::Document),
            ("dependencies", Schema::Document),
            ("evaluated_at_ms", Schema::Number),
        ]),
    )]
}
