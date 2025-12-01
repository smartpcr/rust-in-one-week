//! API response types and utilities

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

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

pub type ApiResult<T> = Result<Json<ApiResponse<T>>, (StatusCode, Json<ApiResponse<()>>)>;

pub fn api_error(status: StatusCode, message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
    (status, Json(ApiResponse::error(message)))
}
