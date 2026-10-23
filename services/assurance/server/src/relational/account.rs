use super::{list, object, Schema};

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
        (
            "requirement_intents",
            list(object(vec![
                ("id", Schema::Text),
                ("fingerprint", Schema::Text),
            ])),
        ),
        ("project", Schema::Text),
        ("model_fingerprint", Schema::Text),
        ("fingerprint", Schema::Text),
    ])
}

// Requirement Review is the only stored review kind; every field is a typed column or child row so
// review storage holds no JSON text.
fn review() -> Schema {
    object(vec![
        ("kind", Schema::Text),
        ("reviewer", Schema::Text),
        ("reviewed_at_ms", Schema::Number),
        ("rationale", Schema::Text),
        (
            "artifacts",
            list(object(vec![
                ("id", Schema::Text),
                ("digest", Schema::Text),
                ("locator", Schema::Text),
            ])),
        ),
        ("requirement", Schema::Text),
        ("requirement_fingerprint", Schema::Text),
        ("dependencies_fingerprint", Schema::Text),
        ("decision", Schema::Text),
        ("reassesses", Schema::Text),
        (
            "challenge_basis",
            list(object(vec![
                ("run", Schema::Text),
                ("bundle_fingerprint", Schema::Text),
                ("result_fingerprint", Schema::Text),
                ("target_fingerprint", Schema::Text),
            ])),
        ),
        ("fingerprint", Schema::Text),
    ])
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

    fn requirement_review(reassesses: Option<&str>) -> Value {
        fingerprint_record(json!({
            "kind": "requirement-review", "reviewer": "reviewer", "reviewed_at_ms": 100,
            "rationale": "Independent examination of the exact declared account.",
            "artifacts": [{"id":"notes","digest":DIGEST,"locator":"reviews/notes.md"}],
            "requirement": "requirement", "requirement_fingerprint": DIGEST,
            "dependencies_fingerprint": DIGEST, "decision": "rejected",
            "reassesses": reassesses,
            "challenge_basis": [{"run":DIGEST,"bundle_fingerprint":DIGEST,"result_fingerprint":DIGEST,"target_fingerprint":DIGEST}]
        }))
    }

    fn review_nodes() -> Vec<usize> {
        let registry = crate::relational::registry();
        let mut nodes = vec![registry.roots["review"]];
        let mut index = 0;
        while index < nodes.len() {
            nodes.extend(registry.nodes[nodes[index]].children.iter().copied());
            index += 1;
        }
        nodes
    }

    #[test]
    fn requirement_review_storage_holds_no_json_text() {
        let registry = crate::relational::registry();
        for node in review_nodes() {
            let node = &registry.nodes[node];
            let documents: Vec<&str> = match &node.schema {
                super::Schema::Document => vec!["(value)"],
                super::Schema::Object(fields) => fields
                    .iter()
                    .filter(|(_, schema)| matches!(schema, super::Schema::Document))
                    .map(|(field, _)| *field)
                    .collect(),
                _ => Vec::new(),
            };
            assert!(
                documents.is_empty(),
                "{} stores {documents:?} as JSON text",
                node.name
            );
            assert!(
                !matches!(node.schema, super::Schema::Ref(_)),
                "{} is not a plain Requirement Review structure",
                node.name
            );
        }
    }

    #[tokio::test]
    async fn requirement_reviews_and_authorities_round_trip_with_native_fingerprints(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().start().await?;
        let port = postgres.get_host_port_ipv4(5432).await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
        ))
        .await?;
        crate::relational::initialize(&pool).await?;
        let original = requirement_review(None);
        let successor = requirement_review(Some(original["fingerprint"].as_str().unwrap()));
        let mut empty = requirement_review(None);
        empty["challenge_basis"] = json!([]);
        let empty = fingerprint_record(empty);
        for value in [original, successor, empty] {
            let native = azimuth::json::parse(&value.to_string())?;
            azimuth::assurance_review::validate_record(&native)?;
            let key = value["fingerprint"].as_str().unwrap();
            let mut connection = pool.acquire().await?;
            crate::relational::store(&mut connection, "review", "project", key, &value).await?;
            drop(connection);
            let restored = crate::relational::load(&pool, "review", "project", key)
                .await?
                .unwrap();
            assert_eq!(restored, value);
            azimuth::assurance_review::validate_record(&azimuth::json::parse(
                &restored.to_string(),
            )?)?;
        }
        let status = json!({
            "kind":"requirement-status","reviewer":"reviewer","reviewed_at_ms":100,
            "rationale":"Status.","artifacts":[],"requirement":"requirement",
            "requirement_fingerprint":DIGEST,"verification_target_fingerprint":DIGEST,
            "requirement_review_fingerprint":DIGEST,"basis":[],"conclusion":"unresolved",
            "residual_risks":[],"challenge_basis":[],"fingerprint":DIGEST
        });
        let mut connection = pool.acquire().await?;
        let error = crate::relational::store(&mut connection, "review", "project", DIGEST, &status)
            .await
            .expect_err("authored Requirement Status has no storage");
        assert!(error.to_string().contains("undeclared field"), "{error}");
        drop(connection);
        let model = azimuth::model::Model {
            specs: vec![azimuth::spec::parse_spec(
                "spec.md",
                "# Spec: module\n\n## Requirement: bounded\nCriticality: routine\n\nThe operation SHALL complete.\n\n### Scenario: completes\nGiven ordinary input, when invoked, then complete.\n",
            ).unwrap()],
            ..azimuth::model::Model::default()
        };
        let current = azimuth::assurance_review::authority(&model, "project")?;
        let current: Value = serde_json::from_str(&current.to_string_pretty())?;
        assert!(
            current["requirement_intents"].is_array(),
            "the authority projection carries Requirement intent fingerprints: {current}"
        );
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
        Ok(())
    }
}
