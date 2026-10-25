use azimuth_assurance_server::{connect, migrate};
use std::error::Error;
use testcontainers_modules::{postgres::Postgres, testcontainers::runners::AsyncRunner};

#[tokio::test]
async fn fresh_storage_contains_no_document_columns_and_initializes_idempotently(
) -> Result<(), Box<dyn Error>> {
    let postgres = Postgres::default().start().await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let pool = connect(&format!(
        "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
    ))
    .await?;
    migrate(&pool).await?;
    migrate(&pool).await?;
    let document_columns: Vec<String> = sqlx::query_scalar(
        "SELECT table_name || '.' || column_name FROM information_schema.columns \
         WHERE table_schema = 'public' AND (udt_name IN ('json', 'jsonb') \
         OR column_name IN ('payload', 'request', 'decision')) ORDER BY 1",
    )
    .fetch_all(&pool)
    .await?;
    assert!(
        document_columns.is_empty(),
        "document columns: {document_columns:?}"
    );
    let versions: Vec<i32> =
        sqlx::query_scalar("SELECT version FROM assurance_storage_schema ORDER BY version")
            .fetch_all(&pool)
            .await?;
    assert_eq!(versions, vec![10]);
    let renamed_tables: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public' \
         AND table_name IN ('assessment_requirements', 'assessment_scenarios', 'definition_requirements')",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(renamed_tables, 3);
    Ok(())
}

#[tokio::test]
async fn original_document_database_is_rejected_without_mutation() -> Result<(), Box<dyn Error>> {
    let postgres = Postgres::default().start().await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let pool = connect(&format!(
        "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
    ))
    .await?;
    sqlx::query("CREATE TABLE observations (id text PRIMARY KEY, payload jsonb NOT NULL)")
        .execute(&pool)
        .await?;
    sqlx::query(
        "INSERT INTO observations VALUES ('retained', '{\"outcome\":\"satisfied\"}'::jsonb)",
    )
    .execute(&pool)
    .await?;
    let error = migrate(&pool)
        .await
        .expect_err("an original database must not be initialized");
    assert!(error
        .to_string()
        .contains("fresh relational database required; original database retained"));
    let retained: String =
        sqlx::query_scalar("SELECT payload->>'outcome' FROM observations WHERE id = 'retained'")
            .fetch_one(&pool)
            .await?;
    assert_eq!(retained, "satisfied");
    let migration_table: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public._sqlx_migrations')::text")
            .fetch_one(&pool)
            .await?;
    assert!(migration_table.is_none());
    let tables: Vec<String> = sqlx::query_scalar("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name")
        .fetch_all(&pool)
        .await?;
    assert_eq!(tables, vec!["observations"]);
    Ok(())
}

#[tokio::test]
async fn publications_stored_before_revisions_become_revision_one_with_no_sources(
) -> Result<(), Box<dyn Error>> {
    let postgres = Postgres::default().start().await?;
    let port = postgres.get_host_port_ipv4(5432).await?;
    let pool = connect(&format!(
        "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
    ))
    .await?;
    let all = sqlx::migrate!("./relational_migrations");
    let before = sqlx::migrate::Migrator {
        migrations: all
            .migrations
            .iter()
            .filter(|migration| migration.version < 8)
            .cloned()
            .collect::<Vec<_>>()
            .into(),
        ..sqlx::migrate::Migrator::DEFAULT
    };
    before.run(&pool).await?;
    let commit = "a".repeat(40);
    sqlx::query("INSERT INTO assurance_authorities(project,fingerprint,registered_by) VALUES('p','m','x')")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT INTO model_revisions(project,fingerprint,registered_by) VALUES('p','m','x')")
        .execute(&pool)
        .await?;
    sqlx::query("INSERT INTO main_publications(project,commit_sha,main_sequence,model_revision,ratification_status,ratification_verified_at_ms,published_by) VALUES('p',$1,7,'m','ratified',0,'x')")
        .bind(&commit)
        .execute(&pool)
        .await?;
    all.run(&pool).await?;
    let (revision, record_key, sources): (i32, String, serde_json::Value) = sqlx::query_as(
        "SELECT publication_revision,main_publication_record_key(commit_sha,publication_revision),main_publication_sources(project,commit_sha,publication_revision) FROM main_publications",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!((revision, record_key), (1, commit.clone()));
    assert_eq!(sources, serde_json::json!({}));
    sqlx::query("INSERT INTO main_publications(project,commit_sha,main_sequence,publication_revision,model_revision,ratification_status,ratification_verified_at_ms,published_by) VALUES('p',$1,7,2,'m','ratified',0,'x')")
        .bind(&commit)
        .execute(&pool)
        .await?;
    let newest: i32 = sqlx::query_scalar(
        "SELECT publication_revision FROM main_publications WHERE project='p' ORDER BY main_sequence DESC,publication_revision DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(newest, 2);
    let duplicate = sqlx::query("INSERT INTO main_publications(project,commit_sha,main_sequence,publication_revision,model_revision,ratification_status,ratification_verified_at_ms,published_by) VALUES('p',$1,7,2,'m','ratified',0,'x')")
        .bind(&commit)
        .execute(&pool)
        .await;
    assert!(duplicate.is_err());
    Ok(())
}
