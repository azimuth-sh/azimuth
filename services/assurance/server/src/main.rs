use azimuth_assurance_server::{app, connect, migrate, AppState};
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "azimuth_assurance_server=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://azimuth:azimuth@localhost:5432/azimuth".into());
    let address = std::env::var("ASSURANCE_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let pool = connect(&database_url)
        .await
        .expect("connect to assurance database");
    migrate(&pool).await.expect("migrate assurance database");
    let listener = TcpListener::bind(&address)
        .await
        .expect("bind assurance service address");
    tracing::info!(address = %address, "assurance service listening");
    axum::serve(listener, app(AppState::new(pool)))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve assurance API");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("install Ctrl-C handler");
}
