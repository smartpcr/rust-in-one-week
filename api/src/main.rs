//! REST API for Windows Failover Cluster and Hyper-V management

use std::{net::SocketAddr, sync::Arc};

use api::{create_router, AppState};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState);
    let app = create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
