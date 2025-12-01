//! Error types for Hyper-V operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HvError {
    #[error("Failed to initialize HCS: {0}")]
    HcsInitFailed(String),

    #[error("VM not found: {0}")]
    VmNotFound(String),

    #[error("Virtual switch not found: {0}")]
    SwitchNotFound(String),

    #[error("VHD not found: {0}")]
    VhdNotFound(String),

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("VM is in invalid state for this operation: {0}")]
    InvalidState(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Failed to connect to Hyper-V WMI: {0}")]
    ConnectionFailed(String),

    #[error("Timeout waiting for operation: {0}")]
    Timeout(String),

    #[error("HCS error: {0}")]
    HcsError(String),

    #[cfg(windows)]
    #[error("Windows API error: {0}")]
    WindowsError(#[from] windows::core::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HvError>;

#[cfg(windows)]
impl From<serde_json::Error> for HvError {
    fn from(e: serde_json::Error) -> Self {
        HvError::JsonError(e.to_string())
    }
}
