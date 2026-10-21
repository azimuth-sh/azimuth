use super::{object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        ("authority", authority()),
        ("review", review()),
        ("catalog", catalog()),
        ("package_fields", package_fields()),
    ]
}

fn authority() -> Schema {
    object(vec![
        ("checks", Schema::Document),
        ("scenarios", Schema::Document),
        ("bindings", Schema::Document),
        ("requirements", Schema::Document),
        ("project", Schema::Text),
        ("model_fingerprint", Schema::Text),
        ("fingerprint", Schema::Text),
    ])
}

fn review() -> Schema {
    let common = vec![
        ("kind", Schema::Text),
        ("reviewer", Schema::Text),
        ("reviewed_at_ms", Schema::Number),
        ("rationale", Schema::Text),
        ("fingerprint", Schema::Text),
        ("artifacts", Schema::Document),
        ("provenance", Schema::Document),
    ];
    let variants = vec![
        vec![
            ("requirement", Schema::Text),
            ("requirement_fingerprint", Schema::Text),
            ("dependencies", Schema::Document),
            ("decision", Schema::Text),
            ("challenge_basis", Schema::Document),
        ],
        vec![
            ("requirement", Schema::Text),
            ("requirement_fingerprint", Schema::Text),
            ("dependencies", Schema::Document),
            ("verification_target_fingerprint", Schema::Text),
            ("requirement_review_fingerprint", Schema::Text),
            ("basis", Schema::Document),
            ("conclusion", Schema::Text),
            ("residual_risks", Schema::Document),
            ("challenge_basis", Schema::Document),
        ],
        vec![
            ("check", Schema::Text),
            ("check_fingerprint", Schema::Text),
            ("decision", Schema::Text),
        ],
        vec![
            ("check", Schema::Text),
            ("scenario", Schema::Text),
            ("check_fingerprint", Schema::Text),
            ("scenario_fingerprint", Schema::Text),
            ("binding_fingerprint", Schema::Text),
            ("verification_target_fingerprint", Schema::Text),
            ("method_qualification_fingerprint", Schema::Text),
            ("basis", Schema::Document),
            ("decision", Schema::Text),
            ("limitations", Schema::Document),
        ],
        vec![
            ("requirement", Schema::Text),
            ("requirement_fingerprint", Schema::Text),
            ("verification_target_fingerprint", Schema::Text),
            ("scenarios", Schema::Document),
            ("mechanisms", Schema::Document),
            ("bindings", Schema::Document),
            ("conclusion", Schema::Text),
            ("residual_risks", Schema::Document),
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
        ("modules", Schema::Document),
        ("requirements", Schema::Document),
        ("scenarios", Schema::Document),
        ("checks", Schema::Document),
    ])
}

fn package_fields() -> Schema {
    object(vec![("fields", Schema::Document)])
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
                json!({"check":"check","scenario":"scenario","check_fingerprint":DIGEST,"scenario_fingerprint":DIGEST,"binding_fingerprint":DIGEST,"verification_target_fingerprint":DIGEST,"method_qualification_fingerprint":DIGEST,"basis":[{"run":DIGEST,"bundle_fingerprint":DIGEST,"check_result":DIGEST}],"decision":"rejected","limitations":["Limited execution context"]})
            }
            "requirement-judgment" => {
                json!({"requirement":"requirement","requirement_fingerprint":DIGEST,"verification_target_fingerprint":DIGEST,"scenarios":[{"id":"scenario","fingerprint":DIGEST}],"mechanisms":[],"bindings":[{"check":"check","scenario":"scenario","fingerprint":DIGEST,"applicability_fingerprint":null}],"conclusion":"unresolved","residual_risks":["Missing applicability"]})
            }
            "requirement-review" => {
                json!({"requirement":"requirement","requirement_fingerprint":DIGEST,"dependencies":{"checks":[],"scenarios":[],"requirements":[],"bindings":[]},"decision":"rejected","challenge_basis":[]})
            }
            "requirement-status" => {
                json!({"requirement":"requirement","requirement_fingerprint":DIGEST,"dependencies":{"checks":[],"scenarios":[],"requirements":[],"bindings":[]},"verification_target_fingerprint":DIGEST,"requirement_review_fingerprint":DIGEST,"basis":[],"conclusion":"unresolved","residual_risks":[],"challenge_basis":[]})
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
            "requirement-review",
            "requirement-status",
            "method-qualification",
            "applicability-decision",
            "requirement-judgment",
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
                "# Spec: module\n\n## Requirement: bounded\nCriticality: routine\n\nThe operation SHALL complete.\n\n### Scenario: completes\nGiven ordinary input, when invoked, then complete.\n",
            ).unwrap()],
            ..azimuth::model::Model::default()
        };
        let current = azimuth::assurance_review::authority(&model, "project")?;
        let current: Value = serde_json::from_str(&current.to_string_pretty())?;
        let mut historical = current.clone();
        historical["requirements"][0]
            .as_object_mut()
            .unwrap()
            .remove("verification_challenges");
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
        let mut unknown = review("requirement-review");
        unknown["unknown"] = true.into();
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
