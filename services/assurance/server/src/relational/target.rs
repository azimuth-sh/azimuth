use super::{list, map, object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    let run = super::run::schemas()
        .into_iter()
        .find(|(kind, _)| *kind == "run")
        .unwrap()
        .1;
    let Schema::Object(fields) = run else {
        unreachable!()
    };
    let Schema::Object(mut plan) = fields
        .iter()
        .find(|(name, _)| *name == "plan")
        .unwrap()
        .1
        .clone()
    else {
        unreachable!()
    };
    plan.retain(|(name, _)| *name != "fingerprint");
    let Schema::Object(provenance) = fields
        .iter()
        .find(|(name, _)| *name == "provenance")
        .unwrap()
        .1
        .clone()
    else {
        unreachable!()
    };
    let Schema::Object(mut adapter) = provenance
        .iter()
        .find(|(name, _)| *name == "adapter")
        .unwrap()
        .1
        .clone()
    else {
        unreachable!()
    };
    let routes = adapter
        .iter()
        .find(|(name, _)| *name == "routes")
        .unwrap()
        .1
        .clone();
    adapter.retain(|(name, _)| !["launch_fingerprint", "routes", "import_inputs"].contains(name));
    let mut schemas = vec![(
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
            (
                "targets",
                list(object(vec![
                    ("component", Schema::Text),
                    ("image", Schema::Text),
                    ("instance", Schema::Text),
                ])),
            ),
            (
                "check",
                object(vec![
                    (
                        "definition",
                        object(vec![
                            ("plan", Schema::Object(plan)),
                            ("adapters", list(Schema::Object(adapter))),
                            ("routes", routes),
                        ]),
                    ),
                    ("outcome", Schema::Text),
                    ("details", Schema::Text),
                ]),
            ),
            (
                "capture",
                object(vec![
                    ("component", Schema::Text),
                    ("instance", Schema::Text),
                    ("started_at_ms", Schema::Number),
                    ("finished_at_ms", Schema::Number),
                    ("before", super::status::snapshot()),
                    ("after", super::status::snapshot()),
                ]),
            ),
        ]),
    )];
    let capability = object(vec![
        ("id", Schema::Text),
        ("classes", list(Schema::Text)),
        ("challenge_forms", list(Schema::Text)),
        ("semantic_settings", map(Schema::Text)),
        ("fingerprint", Schema::Text),
    ]);
    schemas.push((
        "target_normalizer",
        object(vec![
            (
                "description",
                object(vec![
                    ("format", Schema::Text),
                    ("version", Schema::Number),
                    ("protocol_version", Schema::Number),
                    ("id", Schema::Text),
                    ("provider_family", Schema::Text),
                    ("adapter_version", Schema::Text),
                    ("build", Schema::Text),
                    (
                        "content",
                        object(vec![
                            ("executable_digest", Schema::Text),
                            (
                                "resources",
                                list(object(vec![("id", Schema::Text), ("digest", Schema::Text)])),
                            ),
                        ]),
                    ),
                    ("adapter_fingerprint", Schema::Text),
                    ("capabilities", list(capability.clone())),
                    ("descriptor_fingerprint", Schema::Text),
                ]),
            ),
            (
                "configuration",
                object(vec![
                    ("fingerprint", Schema::Text),
                    ("semantic_settings", map(Schema::Text)),
                    ("environment", object(vec![("literals", map(Schema::Text))])),
                    (
                        "limits",
                        object(vec![
                            ("timeout_ms", Schema::Number),
                            ("stdout_bytes", Schema::Number),
                            ("stderr_bytes", Schema::Number),
                        ]),
                    ),
                    ("capabilities", list(capability)),
                ]),
            ),
        ]),
    ));
    schemas
}
