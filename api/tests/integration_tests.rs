//! Integration tests for the API

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use api::{create_router, ApiResponse, AppState};
use std::sync::Arc;

fn create_test_app() -> axum::Router {
    let state = Arc::new(AppState);
    create_router(state)
}

#[tokio::test]
async fn test_root_returns_200() {
    let app = create_test_app();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Windows Infrastructure Management API"));
}

#[tokio::test]
async fn test_health_returns_ok() {
    let app = create_test_app();

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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: ApiResponse<String> = serde_json::from_slice(&body).unwrap();
    assert!(response.success);
    assert_eq!(response.data, Some("ok".to_string()));
}

#[tokio::test]
async fn test_cluster_api_returns_not_implemented_on_non_windows() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/cluster/nodes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // On non-Windows, should return NOT_IMPLEMENTED
    #[cfg(not(windows))]
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

    // On Windows without a cluster, might return different status
    #[cfg(windows)]
    {
        // Just check we get some response
        let _ = response.status();
    }
}

#[tokio::test]
async fn test_hyperv_api_returns_not_implemented_on_non_windows() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/hyperv/vms")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // On non-Windows, should return NOT_IMPLEMENTED
    #[cfg(not(windows))]
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

    // On Windows without Hyper-V, might return different status
    #[cfg(windows)]
    {
        // Just check we get some response
        let _ = response.status();
    }
}

#[tokio::test]
async fn test_404_for_unknown_routes() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/unknown")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cluster_nodes_endpoint_exists() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/cluster/nodes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be 404
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_hyperv_host_endpoint_exists() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/hyperv/host")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should not be 404
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}
