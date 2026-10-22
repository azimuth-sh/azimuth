use super::{list, object, Schema};

pub(super) fn schemas() -> Vec<(&'static str, Schema)> {
    vec![
        ("release", publication()),
        ("change", change()),
        ("deployment", deployment()),
        ("deployment_observation", deployment_observation()),
    ]
}

fn change() -> Schema {
    object(vec![
        ("id", Schema::Text),
        ("revision", Schema::Text),
        ("metadata", Schema::Document),
    ])
}

fn publication() -> Schema {
    object(vec![
        ("format", Schema::Text),
        ("version", Schema::Number),
        ("project", Schema::Text),
        (
            "release",
            object(vec![
                ("fingerprint", Schema::Text),
                ("label", Schema::Text),
                ("model_revision", Schema::Text),
                ("changes", list(Schema::Ref("change"))),
                ("sources", Schema::Document),
                ("builds", Schema::Document),
                ("artifacts", Schema::Document),
                ("deployments", list(Schema::Ref("deployment"))),
                ("components", Schema::Document),
                ("required_reviews", Schema::Document),
                ("integration", Schema::Document),
                ("unit_inputs", Schema::Document),
            ]),
        ),
        (
            "snapshot",
            object(vec![
                ("project", Schema::Text),
                ("model_revision", Schema::Text),
                ("authority", Schema::Ref("authority")),
                ("reviews", list(Schema::Ref("review"))),
                ("bundles", list(Schema::Ref("run"))),
            ]),
        ),
        ("catalog", Schema::Ref("catalog")),
        ("evidence", Schema::Document),
    ])
}

fn deployment() -> Schema {
    object(vec![
        ("id", Schema::Text),
        ("environment", Schema::Text),
        ("configuration_fingerprint", Schema::Text),
        ("artifacts", Schema::Document),
        ("verification_target", Schema::Ref("verification_target")),
        ("verification_target_fingerprint", Schema::Text),
    ])
}

fn deployment_observation() -> Schema {
    object(vec![
        ("format", Schema::Text),
        ("version", Schema::Number),
        ("observed_at_ms", Schema::Number),
        ("valid_for_seconds", Schema::Number),
        ("state", Schema::Text),
        ("release_fingerprint", Schema::Text),
        ("applied_revision", Schema::Text),
        ("rollout_id", Schema::Text),
        ("configuration_fingerprint", Schema::Text),
        ("components", Schema::Document),
        ("diagnostics", Schema::Document),
    ])
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

    const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn native(value: &Value) -> azimuth::json::Json {
        azimuth::json::parse(&value.to_string()).unwrap()
    }

    fn publication() -> Value {
        let authority =
            azimuth::assurance_review::authority(&azimuth::model::Model::default(), "project")
                .unwrap();
        let authority: Value = serde_json::from_str(&authority.to_string_pretty()).unwrap();
        let verification_target = json!({"kind":"deployment","environment":"production","deployment":"deployment","deployment_fingerprint":DIGEST,"artifacts":[{"id":"service","digest":DIGEST}]});
        let verification_target_fingerprint = azimuth::run::verification_target_fingerprint(
            &azimuth::run::verification_target_from_json(&native(&verification_target)).unwrap(),
        );
        let request = json!({
            "format":"azimuth-release-request","version":2,"project":"project","label":"Release 1",
            "changes":[{"id":"change","revision":DIGEST,"metadata":{
                "request":{"title":"Bounded change","ref":"https://example.com/issues/1"},
                "repositories":[{"repository":"source","commit":"a".repeat(40),"pr":"https://example.com/pull/1"}],
                "contributions":[{"kind":"implementation","ref":"https://example.com/commit/a","digest":DIGEST,"author":"author","agent":"agent","model":"model"}]
            }}],
            "sources":[{"repository":"source","commit":"a".repeat(40),"content_fingerprint":DIGEST}],
            "builds":[{"id":"build","sources":["source"],"toolchain":{"rust":"1.90","node":"24"},"configuration_fingerprint":DIGEST,"artifacts":[DIGEST]}],
            "artifacts":[{"digest":DIGEST,"kind":"oci"}],"components":[{"id":"service","digest":DIGEST}],
            "deployments":[{"id":"deployment","environment":"production","configuration_fingerprint":DIGEST,"artifacts":[DIGEST],"verification_target":verification_target,"verification_target_fingerprint":verification_target_fingerprint}],
            "snapshot":{"project":"project","model_revision":authority["fingerprint"],"authority":authority,"reviews":[],"bundles":[]},
            "evidence":[{"digest":DIGEST,"media_type":"text/plain","availability":"unavailable"}],
            "catalog":{"modules":[],"requirements":[],"scenarios":[],"checks":[]},"required_reviews":[]
        });
        let prepared = azimuth::release::prepare(&native(&request)).unwrap();
        serde_json::from_str(&prepared.to_string_pretty()).unwrap()
    }

    #[tokio::test]
    async fn publication_and_deployment_records_round_trip_atomically(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let postgres = Postgres::default().start().await?;
        let port = postgres.get_host_port_ipv4(5432).await?;
        let pool = crate::connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
        ))
        .await?;
        crate::relational::initialize(&pool).await?;
        let package = publication();
        let key = package["release"]["fingerprint"].as_str().unwrap();
        let mut transaction = pool.begin().await?;
        crate::relational::store(&mut transaction, "release", "project", key, &package).await?;
        transaction.commit().await?;
        let restored = crate::relational::load(&pool, "release", "project", key)
            .await?
            .unwrap();
        assert_eq!(restored, package);
        azimuth::release::verify(&native(&restored))?;
        for (kind, value) in [
            ("authority", &package["snapshot"]["authority"]),
            ("catalog", &package["catalog"]),
            ("change", &package["release"]["changes"][0]),
            ("deployment", &package["release"]["deployments"][0]),
        ] {
            let mut connection = pool.acquire().await?;
            crate::relational::store(&mut connection, kind, "project", "explicit-key", value)
                .await?;
            drop(connection);
            assert_eq!(
                crate::relational::load(&pool, kind, "project", "explicit-key")
                    .await?
                    .as_ref(),
                Some(value)
            );
        }
        for state in ["unknown", "complete"] {
            let observation = json!({
                "format":"azimuth-deployment-observation","version":1,"observed_at_ms":100,"valid_for_seconds":30,"state":state,
                "release_fingerprint":if state=="complete" {Some(key)} else {None},
                "applied_revision":null,"rollout_id":"rollout","configuration_fingerprint":DIGEST,
                "components":[{"name":"service","image":"registry/service@sha256:abc","template_fingerprint":DIGEST,"ready":state=="complete"}],
                "diagnostics":["First diagnostic","Second diagnostic"]
            });
            let mut connection = pool.acquire().await?;
            crate::relational::store(
                &mut connection,
                "deployment_observation",
                "project",
                state,
                &observation,
            )
            .await?;
            drop(connection);
            assert_eq!(
                crate::relational::load(&pool, "deployment_observation", "project", state).await?,
                Some(observation)
            );
        }
        let mut transaction = pool.begin().await?;
        crate::relational::store(
            &mut transaction,
            "change",
            "project",
            "rolled-back",
            &package["release"]["changes"][0],
        )
        .await?;
        let mut conflicting = package.clone();
        conflicting["release"]["label"] = "Conflicting replacement".into();
        assert!(crate::relational::store(
            &mut transaction,
            "release",
            "project",
            key,
            &conflicting
        )
        .await
        .is_err());
        transaction.rollback().await?;
        assert!(
            crate::relational::load(&pool, "change", "project", "rolled-back")
                .await?
                .is_none()
        );
        assert_eq!(
            crate::relational::load(&pool, "release", "project", key).await?,
            Some(package)
        );
        Ok(())
    }
}
