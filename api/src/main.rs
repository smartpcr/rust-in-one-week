//! Simple REST API using Axum
//!
//! Demonstrates:
//! - Basic routing (GET, POST, PUT, DELETE)
//! - JSON request/response handling
//! - Path and query parameters
//! - Shared state management
//! - Error handling

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Item stored in the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
}

/// Request body for creating/updating items
#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
}

/// Query parameters for listing items
#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    pub fn error(message: &str) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

/// Shared application state
#[derive(Debug, Default)]
pub struct AppState {
    items: RwLock<HashMap<u64, Item>>,
    next_id: RwLock<u64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
        }
    }
}

pub type SharedState = Arc<AppState>;

/// Create the application router
pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/items", get(list_items).post(create_item))
        .route(
            "/items/:id",
            get(get_item).put(update_item).delete(delete_item),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Root endpoint
async fn root() -> &'static str {
    "Welcome to the Axum API"
}

/// Health check endpoint
async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("ok"))
}

/// List all items with optional pagination
async fn list_items(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> Json<ApiResponse<Vec<Item>>> {
    let items = state.items.read().unwrap();
    let mut result: Vec<Item> = items.values().cloned().collect();

    // Sort by ID for consistent ordering
    result.sort_by_key(|i| i.id);

    // Apply pagination
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(100);

    let result: Vec<Item> = result.into_iter().skip(offset).take(limit).collect();

    Json(ApiResponse::success(result))
}

/// Get a single item by ID
async fn get_item(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<Item>>, (StatusCode, Json<ApiResponse<()>>)> {
    let items = state.items.read().unwrap();

    match items.get(&id) {
        Some(item) => Ok(Json(ApiResponse::success(item.clone()))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Item not found")),
        )),
    }
}

/// Create a new item
async fn create_item(
    State(state): State<SharedState>,
    Json(payload): Json<CreateItem>,
) -> impl IntoResponse {
    let id = {
        let mut next_id = state.next_id.write().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    };

    let item = Item {
        id,
        name: payload.name,
        description: payload.description,
        price: payload.price,
    };

    state.items.write().unwrap().insert(id, item.clone());

    (StatusCode::CREATED, Json(ApiResponse::success(item)))
}

/// Update an existing item
async fn update_item(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
    Json(payload): Json<CreateItem>,
) -> Result<Json<ApiResponse<Item>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut items = state.items.write().unwrap();

    if !items.contains_key(&id) {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Item not found")),
        ));
    }

    let item = Item {
        id,
        name: payload.name,
        description: payload.description,
        price: payload.price,
    };

    items.insert(id, item.clone());

    Ok(Json(ApiResponse::success(item)))
}

/// Delete an item
async fn delete_item(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<ApiResponse<&'static str>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut items = state.items.write().unwrap();

    match items.remove(&id) {
        Some(_) => Ok(Json(ApiResponse::success("Item deleted"))),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Item not found")),
        )),
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create shared state
    let state = Arc::new(AppState::new());

    // Build router
    let app = create_router(state);

    // Run server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// =============================================================================
// Integration Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        let state = Arc::new(AppState::new());
        create_router(state)
    }

    #[tokio::test]
    async fn test_root() {
        let app = create_test_app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"Welcome to the Axum API");
    }

    #[tokio::test]
    async fn test_health() {
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
        let json: ApiResponse<String> = serde_json::from_slice(&body).unwrap();
        assert!(json.success);
        assert_eq!(json.data.unwrap(), "ok");
    }

    #[tokio::test]
    async fn test_create_and_get_item() {
        let state = Arc::new(AppState::new());
        let app = create_router(state);

        // Create item
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"name": "Test Item", "description": "A test", "price": 9.99}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(create_response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<Item> = serde_json::from_slice(&body).unwrap();
        assert!(json.success);
        let created_item = json.data.unwrap();
        assert_eq!(created_item.name, "Test Item");
        assert_eq!(created_item.price, 9.99);

        // Get item
        let get_response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{}", created_item.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(get_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<Item> = serde_json::from_slice(&body).unwrap();
        assert!(json.success);
        let item = json.data.unwrap();
        assert_eq!(item.id, created_item.id);
        assert_eq!(item.name, "Test Item");
    }

    #[tokio::test]
    async fn test_list_items() {
        let state = Arc::new(AppState::new());
        let app = create_router(state);

        // Create multiple items
        for i in 1..=3 {
            app.clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/items")
                        .header("content-type", "application/json")
                        .body(Body::from(format!(
                            r#"{{"name": "Item {}", "price": {}.99}}"#,
                            i, i
                        )))
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        // List all items
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/items")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<Vec<Item>> = serde_json::from_slice(&body).unwrap();
        assert!(json.success);
        assert_eq!(json.data.unwrap().len(), 3);

        // Test pagination
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items?limit=2&offset=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<Vec<Item>> = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.data.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_update_item() {
        let state = Arc::new(AppState::new());
        let app = create_router(state);

        // Create item
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name": "Original", "price": 10.00}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<Item> = serde_json::from_slice(&body).unwrap();
        let item_id = json.data.unwrap().id;

        // Update item
        let update_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/items/{}", item_id))
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name": "Updated", "price": 20.00}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(update_response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(update_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<Item> = serde_json::from_slice(&body).unwrap();
        let updated = json.data.unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.price, 20.00);
    }

    #[tokio::test]
    async fn test_delete_item() {
        let state = Arc::new(AppState::new());
        let app = create_router(state);

        // Create item
        let create_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/items")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name": "To Delete", "price": 5.00}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<Item> = serde_json::from_slice(&body).unwrap();
        let item_id = json.data.unwrap().id;

        // Delete item
        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/items/{}", item_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(delete_response.status(), StatusCode::OK);

        // Verify item is gone
        let get_response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/items/{}", item_id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_not_found() {
        let app = create_test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/items/999")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: ApiResponse<()> = serde_json::from_slice(&body).unwrap();
        assert!(!json.success);
        assert!(json.error.is_some());
    }
}
