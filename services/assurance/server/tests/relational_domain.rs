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
    let metadata_tables: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public' \
         AND table_name IN ('projects', 'project_model_snapshots', 'evidence_definitions', \
         'qualifications', 'observations', 'challenges', 'gate_decisions')",
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(metadata_tables, 7);
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
