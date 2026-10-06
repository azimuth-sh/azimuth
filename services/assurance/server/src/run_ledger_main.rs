use azimuth_assurance_server::{
    connect, migrate,
    run_ledger::{app, LedgerState, ProjectCredentials},
};
use std::collections::BTreeMap;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let credential_file =
        std::env::var("ASSURANCE_RUN_TOKENS_FILE").expect("ASSURANCE_RUN_TOKENS_FILE is required");
    let credential_bytes = std::fs::read(credential_file).expect("read project credentials");
    let credential_text =
        std::str::from_utf8(&credential_bytes).expect("project credentials must be UTF-8");
    azimuth::run::strict_json("project credentials", credential_text)
        .unwrap_or_else(|_| panic!("project credentials must have unique JSON keys"));
    let tokens: BTreeMap<String, ProjectCredentials> =
        serde_json::from_slice(&credential_bytes).expect("project credentials must be a JSON map");
    let address =
        std::env::var("ASSURANCE_RUN_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8081".into());
    let pool = connect(&database_url)
        .await
        .expect("connect Run ledger database");
    migrate(&pool).await.expect("migrate Run ledger database");
    let state = LedgerState::new(pool, tokens).expect("validate project credentials");
    let listener = TcpListener::bind(&address)
        .await
        .expect("bind Run ledger address");
    tracing::info!(address = %address, "Run ledger listening");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("install shutdown signal");
        })
        .await
        .expect("serve Run ledger");
}
