use core::fmt;

/// VM enabled state (copy for error module to avoid circular dependency).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmStateError {
    Unknown,
    Running,
    Off,
    ShuttingDown,
    Paused,
    Suspended,
    Starting,
    Stopping,
    Other(u16),
}

impl fmt::Display for VmStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmStateError::Unknown => write!(f, "Unknown"),
            VmStateError::Running => write!(f, "Running"),
            VmStateError::Off => write!(f, "Off"),
            VmStateError::ShuttingDown => write!(f, "Shutting Down"),
            VmStateError::Paused => write!(f, "Paused"),
            VmStateError::Suspended => write!(f, "Saved"),
            VmStateError::Starting => write!(f, "Starting"),
            VmStateError::Stopping => write!(f, "Stopping"),
            VmStateError::Other(v) => write!(f, "State({})", v),
        }
    }
}

/// Hyper-V operation errors with typed context.
#[derive(Debug)]
pub enum Error {
    /// Failed to connect to WMI.
    WmiConnection(windows_core::Error),

    /// Failed to execute WMI query.
    WmiQuery {
        query: String,
        source: windows_core::Error,
    },

    /// Failed to invoke WMI method.
    WmiMethod {
        class: &'static str,
        method: &'static str,
        source: windows_core::Error,
    },

    /// VM not found by name or ID.
    VmNotFound(String),

    /// Virtual switch not found.
    SwitchNotFound(String),

    /// VHD/VHDX file not found.
    VhdNotFound(String),

    /// Operation invalid for current VM state.
    InvalidState {
        vm_name: String,
        current: VmStateError,
        operation: &'static str,
    },

    /// Property validation failed.
    Validation {
        field: &'static str,
        message: String,
    },

    /// Required property missing.
    MissingRequired(&'static str),

    /// WMI operation returned failure code.
    OperationFailed {
        operation: &'static str,
        return_value: u32,
        message: String,
    },

    /// Failed to convert WMI VARIANT to expected type.
    TypeConversion {
        property: &'static str,
        expected: &'static str,
    },

    /// Job failed during async operation.
    JobFailed {
        operation: &'static str,
        error_code: u32,
        error_description: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::WmiConnection(e) => write!(f, "WMI connection failed: {e}"),
            Error::WmiQuery { query, source } => {
                write!(f, "WMI query failed: {query} - {source}")
            }
            Error::WmiMethod {
                class,
                method,
                source,
            } => {
                write!(f, "WMI method {class}.{method} failed: {source}")
            }
            Error::VmNotFound(name) => write!(f, "VM not found: {name}"),
            Error::SwitchNotFound(name) => write!(f, "Virtual switch not found: {name}"),
            Error::VhdNotFound(path) => write!(f, "VHD not found: {path}"),
            Error::InvalidState {
                vm_name,
                current,
                operation,
            } => {
                write!(
                    f,
                    "Cannot {operation} VM '{vm_name}' in state {current}"
                )
            }
            Error::Validation { field, message } => {
                write!(f, "Validation failed for '{field}': {message}")
            }
            Error::MissingRequired(field) => {
                write!(f, "Required field missing: {field}")
            }
            Error::OperationFailed {
                operation,
                return_value,
                message,
            } => {
                write!(
                    f,
                    "Operation '{operation}' failed with code {return_value}: {message}"
                )
            }
            Error::TypeConversion { property, expected } => {
                write!(f, "Cannot convert property '{property}' to {expected}")
            }
            Error::JobFailed {
                operation,
                error_code,
                error_description,
            } => {
                write!(
                    f,
                    "Job failed for '{operation}' (code {error_code}): {error_description}"
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::WmiConnection(e) => Some(e),
            Error::WmiQuery { source, .. } => Some(source),
            Error::WmiMethod { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<windows_core::Error> for Error {
    fn from(e: windows_core::Error) -> Self {
        Error::WmiConnection(e)
    }
}

/// Result type for Hyper-V operations.
pub type Result<T> = core::result::Result<T, Error>;
