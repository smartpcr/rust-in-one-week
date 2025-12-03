//! REST API library for Windows Failover Cluster and Hyper-V management
//!
//! Provides REST endpoints for:
//! - Failover Cluster: nodes, groups, resources, CSV
//! - Hyper-V: VMs, VHDs, snapshots, switches, GPU (GPU-P and DDA)

pub mod config;
pub mod dto;
pub mod handlers;
pub mod response;
pub mod routes;
pub mod service;

use std::sync::Arc;

use axum::{routing::get, Json, Router};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use config::Config;
pub use dto::*;
pub use response::{ApiResponse, ApiResult};

// =============================================================================
// Tracing Initialization
// =============================================================================

/// Initialize tracing/logging with the given filter level
pub fn init_tracing(filter: &str) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// =============================================================================
// Shared State
// =============================================================================

#[derive(Default)]
pub struct AppState;

pub type SharedState = Arc<AppState>;

// =============================================================================
// Router
// =============================================================================

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/api/v1", api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

fn api_routes() -> Router<SharedState> {
    Router::new()
        .nest("/cluster", routes::cluster_routes())
        .nest("/hyperv", routes::hyperv_routes())
}

// =============================================================================
// Root Endpoints
// =============================================================================

async fn root() -> &'static str {
    "Windows Infrastructure Management API - Use /api/v1/cluster or /api/v1/hyperv"
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("ok"))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_root_endpoint() {
        let state = Arc::new(AppState);
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = Arc::new(AppState);
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_response_success() {
        let response: ApiResponse<&str> = ApiResponse::success("test");
        assert!(response.success);
        assert_eq!(response.data, Some("test"));
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_api_response_error() {
        let response: ApiResponse<()> = ApiResponse::error("test error");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}
