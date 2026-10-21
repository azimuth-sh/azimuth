pub(crate) mod relational;
pub mod run_ledger;

use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    let document_storage = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns \
         WHERE table_schema = 'public' AND (udt_name IN ('json', 'jsonb') \
         OR column_name IN ('payload', 'request', 'decision')))",
    )
    .fetch_one(pool)
    .await
    .map_err(sqlx::migrate::MigrateError::Execute)?;
    if document_storage {
        return Err(sqlx::migrate::MigrateError::Execute(sqlx::Error::Protocol(
            "fresh relational database required; original database retained".into(),
        )));
    }
    sqlx::migrate!("./relational_migrations").run(pool).await?;
    relational::initialize(pool)
        .await
        .map_err(sqlx::migrate::MigrateError::Execute)?;
    sqlx::raw_sql(&relational::assessment_triggers())
        .execute(pool)
        .await
        .map_err(sqlx::migrate::MigrateError::Execute)?;
    Ok(())
}
