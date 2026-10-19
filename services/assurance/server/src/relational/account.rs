use super::{list, object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        ("authority", authority()),
        ("review", review()),
        ("catalog", catalog()),
        ("package_fields", package_fields()),
        ("package_type", package_type()),
    ]
}

fn identity() -> Schema {
    object(vec![("id", Schema::Text), ("fingerprint", Schema::Text)])
}

fn binding() -> Schema {
    object(vec![
        ("check", Schema::Text),
        ("case", Schema::Text),
        ("check_fingerprint", Schema::Text),
        ("case_fingerprint", Schema::Text),
        ("contribution", Schema::Text),
        ("fingerprint", Schema::Text),
    ])
}

fn challenger() -> Schema {
    object(vec![
        ("id", Schema::Text),
        ("fingerprint", Schema::Text),
        ("form", Schema::Text),
        ("targets", list(Schema::Text)),
        ("claims", list(Schema::Text)),
        ("mechanisms", list(Schema::Text)),
        ("checks", list(Schema::Text)),
        ("searches_for", Schema::Text),
        ("rationale", Schema::Text),
        ("required_scope", list(Schema::Text)),
        (
            "implementations",
            list(object(vec![
                ("package", Schema::Text),
                ("contract", Schema::Text),
                ("entity", Schema::Text),
                ("site", Schema::Text),
                ("file", Schema::Text),
                ("lang", Schema::Text),
                ("source_fingerprint", Schema::Text),
                (
                    "inputs",
                    list(object(vec![
                        ("file", Schema::Text),
                        ("fingerprint", Schema::Text),
                    ])),
                ),
                ("schema", Schema::Ref("package_fields")),
                ("area", Schema::Text),
                ("address_kind", Schema::Text),
                ("address", Schema::Text),
                ("mount", Schema::Text),
            ])),
        ),
    ])
}

fn claim() -> Schema {
    object(vec![
        ("id", Schema::Text),
        ("criticality", Schema::Text),
        ("fingerprint", Schema::Text),
        ("cases", list(Schema::Text)),
        ("mechanisms", list(identity())),
        (
            "bindings",
            list(object(vec![
                ("check", Schema::Text),
                ("case", Schema::Text),
            ])),
        ),
        ("gaps", list(Schema::Text)),
        ("challengers", list(challenger())),
    ])
}

fn dependencies() -> Schema {
    object(vec![
        ("checks", list(identity())),
        (
            "cases",
            list(object(vec![
                ("id", Schema::Text),
                ("claim", Schema::Text),
                ("fingerprint", Schema::Text),
            ])),
        ),
        ("bindings", list(binding())),
        ("claims", list(claim())),
    ])
}

fn authority() -> Schema {
    let Schema::Object(mut fields) = dependencies() else {
        unreachable!()
    };
    fields.extend([
        ("project", Schema::Text),
        ("model_fingerprint", Schema::Text),
        ("fingerprint", Schema::Text),
    ]);
    object(fields)
}

fn basis() -> Schema {
    list(object(vec![
        ("run", Schema::Text),
        ("bundle_fingerprint", Schema::Text),
        ("observation", Schema::Text),
    ]))
}

fn review() -> Schema {
    let common = vec![
        ("kind", Schema::Text),
        ("reviewer", Schema::Text),
        ("reviewed_at_ms", Schema::Number),
        ("rationale", Schema::Text),
        ("fingerprint", Schema::Text),
        (
            "artifacts",
            list(object(vec![
                ("id", Schema::Text),
                ("digest", Schema::Text),
                ("locator", Schema::Text),
            ])),
        ),
        (
            "provenance",
            object(vec![
                ("agent", Schema::Text),
                ("model", Schema::Text),
                ("reassesses", Schema::Text),
            ]),
        ),
    ];
    let challenge_basis = || {
        list(object(vec![
            ("run", Schema::Text),
            ("bundle_fingerprint", Schema::Text),
            ("result_fingerprint", Schema::Text),
            ("target_fingerprint", Schema::Text),
        ]))
    };
    let variants = vec![
        vec![
            ("claim", Schema::Text),
            ("claim_fingerprint", Schema::Text),
            ("dependencies", dependencies()),
            ("decision", Schema::Text),
            ("challenge_basis", challenge_basis()),
        ],
        vec![
            ("claim", Schema::Text),
            ("claim_fingerprint", Schema::Text),
            ("dependencies", dependencies()),
            ("subject_fingerprint", Schema::Text),
            ("claim_review_fingerprint", Schema::Text),
            ("basis", basis()),
            ("conclusion", Schema::Text),
            ("residual_risks", list(Schema::Text)),
            ("challenge_basis", challenge_basis()),
        ],
        vec![
            ("check", Schema::Text),
            ("check_fingerprint", Schema::Text),
            ("decision", Schema::Text),
        ],
        vec![
            ("check", Schema::Text),
            ("case", Schema::Text),
            ("check_fingerprint", Schema::Text),
            ("case_fingerprint", Schema::Text),
            ("binding_fingerprint", Schema::Text),
            ("subject_fingerprint", Schema::Text),
            ("method_qualification_fingerprint", Schema::Text),
            ("basis", basis()),
            ("decision", Schema::Text),
            ("limitations", list(Schema::Text)),
        ],
        vec![
            ("claim", Schema::Text),
            ("claim_fingerprint", Schema::Text),
            ("subject_fingerprint", Schema::Text),
            ("cases", list(identity())),
            ("mechanisms", list(identity())),
            (
                "bindings",
                list(object(vec![
                    ("check", Schema::Text),
                    ("case", Schema::Text),
                    ("fingerprint", Schema::Text),
                    ("applicability_fingerprint", Schema::Text),
                ])),
            ),
            ("conclusion", Schema::Text),
            ("residual_risks", list(Schema::Text)),
        ],
    ];
    Schema::OneOf(
        variants
            .into_iter()
            .map(|fields| {
                let mut result = common.clone();
                result.extend(fields);
                object(result)
            })
            .collect(),
    )
}

fn catalog() -> Schema {
    object(vec![
        (
            "modules",
            list(object(vec![
                ("id", Schema::Text),
                ("title", Schema::Text),
                ("claims", list(Schema::Text)),
                ("parent", Schema::Text),
            ])),
        ),
        (
            "claims",
            list(object(vec![
                ("id", Schema::Text),
                ("statement", Schema::Text),
            ])),
        ),
        (
            "cases",
            list(object(vec![
                ("id", Schema::Text),
                ("description", Schema::Text),
            ])),
        ),
        (
            "checks",
            list(object(vec![
                ("id", Schema::Text),
                ("description", Schema::Text),
            ])),
        ),
    ])
}

fn package_fields() -> Schema {
    object(vec![(
        "fields",
        list(object(vec![
            ("name", Schema::Text),
            ("schema", Schema::Ref("package_type")),
        ])),
    )])
}

fn package_type() -> Schema {
    Schema::OneOf(vec![
        object(vec![("kind", Schema::Text)]),
        object(vec![("kind", Schema::Text), ("values", list(Schema::Text))]),
        object(vec![
            ("kind", Schema::Text),
            ("items", Schema::Ref("package_type")),
        ]),
        object(vec![
            ("kind", Schema::Text),
            ("keys", Schema::Ref("package_type")),
            ("values", Schema::Ref("package_type")),
        ]),
        object(vec![
            ("kind", Schema::Text),
            (
                "fields",
                list(object(vec![
                    ("name", Schema::Text),
                    ("schema", Schema::Ref("package_type")),
                ])),
            ),
        ]),
        object(vec![
            ("kind", Schema::Text),
            ("value", Schema::Ref("package_type")),
        ]),
    ])
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

    const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn fingerprint_record(mut value: Value) -> Value {
        let native = azimuth::json::parse(&value.to_string()).unwrap();
        value["fingerprint"] = azimuth::assurance_review::record_fingerprint(&native)
            .unwrap()
            .into();
        value
    }

    fn review(kind: &str) -> Value {
        let mut value = json!({
            "kind": kind, "reviewer": "reviewer", "reviewed_at_ms": 100,
            "rationale": "Independent examination of the exact declared account.",
            "artifacts": [{"id":"notes","digest":DIGEST,"locator":"reviews/notes.md"}],
            "provenance": {"agent":null,"model":"review-model","reassesses":null}
        });
        let fields = match kind {
            "method-qualification" => {
                json!({"check":"check","check_fingerprint":DIGEST,"decision":"accepted"})
            }
            "applicability-decision" => {
                json!({"check":"check","case":"case","check_fingerprint":DIGEST,"case_fingerprint":DIGEST,"binding_fingerprint":DIGEST,"subject_fingerprint":DIGEST,"method_qualification_fingerprint":DIGEST,"basis":[{"run":DIGEST,"bundle_fingerprint":DIGEST,"observation":DIGEST}],"decision":"rejected","limitations":["Limited execution context"]})
            }
            "claim-judgment" => {
                json!({"claim":"claim","claim_fingerprint":DIGEST,"subject_fingerprint":DIGEST,"cases":[{"id":"case","fingerprint":DIGEST}],"mechanisms":[],"bindings":[{"check":"check","case":"case","fingerprint":DIGEST,"applicability_fingerprint":null}],"conclusion":"unresolved","residual_risks":["Missing applicability"]})
            }
            "claim-review" => {
                json!({"claim":"claim","claim_fingerprint":DIGEST,"dependencies":{"checks":[],"cases":[],"claims":[],"bindings":[]},"decision":"rejected","challenge_basis":[]})
            }
            "claim-health" => {
                json!({"claim":"claim","claim_fingerprint":DIGEST,"dependencies":{"checks":[],"cases":[],"claims":[],"bindings":[]},"subject_fingerprint":DIGEST,"claim_review_fingerprint":DIGEST,"basis":[],"conclusion":"unresolved","residual_risks":[],"challenge_basis":[]})
            }
            _ => unreachable!(),
        };
        value
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        fingerprint_record(value)
    }

    #[tokio::test]
    async fn retained_review_kinds_round_trip_with_native_fingerprints(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().start().await?;
        let port = postgres.get_host_port_ipv4(5432).await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
        ))
        .await?;
        crate::relational::initialize(&pool).await?;
        for kind in [
            "claim-review",
            "claim-health",
            "method-qualification",
            "applicability-decision",
            "claim-judgment",
        ] {
            let value = review(kind);
            let native = azimuth::json::parse(&value.to_string())?;
            azimuth::assurance_review::validate_record(&native)?;
            let key = value["fingerprint"].as_str().unwrap();
            let mut connection = pool.acquire().await?;
            crate::relational::store(&mut connection, "review", "project", key, &value).await?;
            drop(connection);
            let restored = crate::relational::load(&pool, "review", "project", key)
                .await?
                .unwrap();
            assert_eq!(restored, value, "{kind}");
            azimuth::assurance_review::validate_record(&azimuth::json::parse(
                &restored.to_string(),
            )?)?;
        }
        let model = azimuth::model::Model {
            specs: vec![azimuth::spec::parse_spec(
                "spec.md",
                "# Spec: module\n\n## Claim: bounded\nCriticality: routine\n\nThe operation SHALL complete.\n\n### Case: completes\nGiven ordinary input, when invoked, then complete.\n",
            ).unwrap()],
            ..azimuth::model::Model::default()
        };
        let current = azimuth::assurance_review::authority(&model, "project")?;
        let current: Value = serde_json::from_str(&current.to_string_pretty())?;
        let mut historical = current.clone();
        historical["claims"][0]
            .as_object_mut()
            .unwrap()
            .remove("challengers");
        historical = fingerprint_record(historical);
        for value in [current, historical] {
            azimuth::assurance_review::validate_authority(&azimuth::json::parse(
                &value.to_string(),
            )?)?;
            let key = value["fingerprint"].as_str().unwrap();
            let mut connection = pool.acquire().await?;
            crate::relational::store(&mut connection, "authority", "project", key, &value).await?;
            drop(connection);
            assert_eq!(
                crate::relational::load(&pool, "authority", "project", key).await?,
                Some(value)
            );
        }
        let mut unknown = review("claim-review");
        unknown["dependencies"]["unknown"] = true.into();
        let registry = crate::relational::registry();
        let root = registry.roots["review"];
        assert!(!crate::relational::fits(
            registry,
            &registry.nodes[root].schema,
            &unknown,
            0
        ));
        Ok(())
    }
}
