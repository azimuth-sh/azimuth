use super::{list, map, object, Schema};

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
                ("verification_target", verification_target()),
                ("verification_target_fingerprint", Schema::Text),
                ("planned_at_ms", Schema::Number),
                ("started_at_ms", Schema::Number),
                ("finished_at_ms", Schema::Number),
                ("status", Schema::Text),
                (
                    "plan",
                    object(vec![
                        ("model_fingerprint", Schema::Text),
                        ("required_execution_context", map(Schema::Text)),
                        ("checks", list(check_selection())),
                        ("challenges", list(challenge_selection())),
                        ("fingerprint", Schema::Text),
                    ]),
                ),
                (
                    "actual_selection",
                    object(vec![
                        ("execution_context", map(Schema::Text)),
                        ("plan_fingerprint", Schema::Text),
                        ("checks", list(check_selection())),
                        ("challenges", list(challenge_selection())),
                        ("fingerprint", Schema::Text),
                    ]),
                ),
                ("provenance", provenance()),
                (
                    "artifacts",
                    list(object(vec![
                        ("id", Schema::Text),
                        ("kind", Schema::Text),
                        ("media_type", Schema::Text),
                        ("digest", Schema::Text),
                        ("size_bytes", Schema::Number),
                        (
                            "locator",
                            object(vec![("kind", Schema::Text), ("value", Schema::Text)]),
                        ),
                    ])),
                ),
                (
                    "diagnostics",
                    list(object(vec![
                        ("id", Schema::Text),
                        ("class", Schema::Text),
                        ("severity", Schema::Text),
                        ("code", Schema::Text),
                        ("message", Schema::Text),
                        (
                            "scope",
                            object(vec![
                                ("kind", Schema::Text),
                                ("id", Schema::Text),
                                ("check", Schema::Text),
                                ("verification_challenge_fingerprint", Schema::Text),
                                ("target_fingerprint", Schema::Text),
                            ]),
                        ),
                        ("artifacts", list(Schema::Text)),
                        ("details", map(Schema::Text)),
                    ])),
                ),
                (
                    "activities",
                    list(object(vec![
                        ("id", Schema::Text),
                        ("status", Schema::Text),
                        ("started_at_ms", Schema::Number),
                        ("finished_at_ms", Schema::Number),
                        ("artifacts", list(Schema::Text)),
                        ("diagnostics", list(Schema::Text)),
                        ("attributes", map(Schema::Text)),
                    ])),
                ),
                (
                    "check_executions",
                    list(object(vec![
                        ("check", identity()),
                        (
                            "units",
                            list(object(vec![
                                ("id", Schema::Text),
                                (
                                    "attempts",
                                    list(object(vec![
                                        ("ordinal", Schema::Number),
                                        ("activity", Schema::Text),
                                        ("outcomes", map(Schema::Text)),
                                    ])),
                                ),
                            ])),
                        ),
                        (
                            "results",
                            list(object(vec![
                                ("scenario", Schema::Text),
                                ("outcome", Schema::Text),
                                ("observed_at_ms", Schema::Number),
                                ("fingerprint", Schema::Text),
                                ("artifacts", list(Schema::Text)),
                                ("diagnostics", list(Schema::Text)),
                            ])),
                        ),
                    ])),
                ),
                (
                    "verification_challenge_executions",
                    list(object(vec![
                        ("challenge", Schema::Text),
                        ("verification_challenge", identity()),
                        ("target", target()),
                        (
                            "units",
                            list(object(vec![
                                ("id", Schema::Text),
                                (
                                    "attempts",
                                    list(object(vec![
                                        ("ordinal", Schema::Number),
                                        ("activity", Schema::Text),
                                        ("outcome", Schema::Text),
                                    ])),
                                ),
                            ])),
                        ),
                        (
                            "result",
                            object(vec![
                                ("outcome", Schema::Text),
                                ("observed_at_ms", Schema::Number),
                                ("fingerprint", Schema::Text),
                                ("objections", list(Schema::Text)),
                                ("artifacts", list(Schema::Text)),
                                ("diagnostics", list(Schema::Text)),
                            ]),
                        ),
                    ])),
                ),
                ("contributions", list(Schema::Ref("run"))),
            ]),
        ),
    ]
}

fn identity() -> Schema {
    object(vec![("id", Schema::Text), ("fingerprint", Schema::Text)])
}

fn target() -> Schema {
    object(vec![
        ("kind", Schema::Text),
        ("id", Schema::Text),
        ("fingerprint", Schema::Text),
    ])
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

fn work_unit() -> Schema {
    object(vec![
        ("id", Schema::Text),
        ("parameters", map(Schema::Text)),
    ])
}

fn check_selection() -> Schema {
    object(vec![
        ("id", Schema::Text),
        ("fingerprint", Schema::Text),
        ("scenarios", list(Schema::Text)),
        (
            "implementations",
            list(object(vec![
                ("identity", Schema::Text),
                ("source_fingerprint", Schema::Text),
            ])),
        ),
        ("units", list(work_unit())),
    ])
}

fn challenge_selection() -> Schema {
    object(vec![
        ("id", Schema::Text),
        ("verification_challenge", identity()),
        ("target", target()),
        ("lane", Schema::Text),
        (
            "scope",
            object(vec![
                ("anchors", list(target())),
                ("inputs", list(target())),
                ("fingerprint", Schema::Text),
            ]),
        ),
        ("units", list(work_unit())),
    ])
}

fn provenance() -> Schema {
    object(vec![
        ("mode", Schema::Text),
        (
            "source",
            object(vec![
                ("system", Schema::Text),
                ("execution", Schema::Text),
                ("uri", Schema::Text),
            ]),
        ),
        (
            "normalizer",
            object(vec![
                ("id", Schema::Text),
                ("version", Schema::Text),
                ("build_fingerprint", Schema::Text),
            ]),
        ),
        (
            "adapter",
            object(vec![
                ("id", Schema::Text),
                ("adapter_version", Schema::Text),
                ("adapter_fingerprint", Schema::Text),
                ("descriptor_fingerprint", Schema::Text),
                ("configuration_fingerprint", Schema::Text),
                ("launch_fingerprint", Schema::Text),
                (
                    "routes",
                    list(object(vec![
                        (
                            "selection",
                            object(vec![("kind", Schema::Text), ("id", Schema::Text)]),
                        ),
                        (
                            "capability",
                            object(vec![
                                ("address", Schema::Text),
                                ("class", Schema::Text),
                                ("challenge_form", Schema::Text),
                                ("fingerprint", Schema::Text),
                            ]),
                        ),
                        (
                            "inputs",
                            list(object(vec![
                                ("kind", Schema::Text),
                                ("id", Schema::Text),
                                ("fingerprint", Schema::Text),
                                (
                                    "source",
                                    object(vec![
                                        ("kind", Schema::Text),
                                        ("file", Schema::Text),
                                        ("language", Schema::Text),
                                        ("site", Schema::Text),
                                        ("artifact_kind", Schema::Text),
                                        ("identity", Schema::Text),
                                        ("unique", Schema::Boolean),
                                        ("columns", list(Schema::Text)),
                                        ("predicate", Schema::Text),
                                        ("enumerator_kind", Schema::Text),
                                        ("member_kind", Schema::Text),
                                    ]),
                                ),
                            ])),
                        ),
                    ])),
                ),
                (
                    "import_inputs",
                    list(object(vec![
                        ("id", Schema::Text),
                        ("digest", Schema::Text),
                        ("size_bytes", Schema::Number),
                    ])),
                ),
            ]),
        ),
        ("generated_at_ms", Schema::Number),
        ("principal", Schema::Text),
        ("attributes", map(Schema::Text)),
    ])
}
