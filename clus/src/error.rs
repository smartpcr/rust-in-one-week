//! Error types for cluster operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClusError {
    #[error("Failed to open cluster: {0}")]
    OpenClusterFailed(String),

    #[error("Failed to open cluster node: {0}")]
    OpenNodeFailed(String),

    #[error("Failed to open cluster resource: {0}")]
    OpenResourceFailed(String),

    #[error("Failed to open cluster group: {0}")]
    OpenGroupFailed(String),

    #[error("Cluster operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid cluster handle")]
    InvalidHandle,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[cfg(windows)]
    #[error("Windows API error: {0}")]
    WindowsError(#[from] windows::core::Error),
}

pub type Result<T> = std::result::Result<T, ClusError>;
