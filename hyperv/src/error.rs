#[cfg(windows)]
use windows::core::Error as WinError;

use core::fmt;
use std::time::Duration;

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
    Hibernated,
    Migrating,
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
            VmStateError::Hibernated => write!(f, "Hibernated"),
            VmStateError::Migrating => write!(f, "Migrating"),
            VmStateError::Other(v) => write!(f, "State({})", v),
        }
    }
}

/// Classification of failure types for retry logic and error handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureType {
    /// Transient failure - operation may succeed if retried.
    Transient,
    /// Permanent failure - retrying will not help.
    Permanent,
    /// Resource busy - retry after delay.
    ResourceBusy,
    /// Authentication/authorization failure.
    AuthenticationFailed,
    /// Configuration error - fix configuration and retry.
    Configuration,
    /// Network-related failure.
    Network,
    /// Unknown failure type.
    Unknown,
}

impl fmt::Display for FailureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FailureType::Transient => write!(f, "Transient"),
            FailureType::Permanent => write!(f, "Permanent"),
            FailureType::ResourceBusy => write!(f, "ResourceBusy"),
            FailureType::AuthenticationFailed => write!(f, "AuthenticationFailed"),
            FailureType::Configuration => write!(f, "Configuration"),
            FailureType::Network => write!(f, "Network"),
            FailureType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// WMI Job state values for async operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum JobState {
    /// Job is queued.
    New = 2,
    /// Job is starting.
    Starting = 3,
    /// Job is running.
    Running = 4,
    /// Job is suspended.
    Suspended = 5,
    /// Job is shutting down.
    ShuttingDown = 6,
    /// Job completed successfully.
    Completed = 7,
    /// Job was terminated.
    Terminated = 8,
    /// Job was killed.
    Killed = 9,
    /// Job failed with exception.
    Exception = 10,
    /// Job is in service mode.
    Service = 11,
    /// Unknown state.
    Unknown = 0,
}

impl From<u16> for JobState {
    fn from(value: u16) -> Self {
        match value {
            2 => JobState::New,
            3 => JobState::Starting,
            4 => JobState::Running,
            5 => JobState::Suspended,
            6 => JobState::ShuttingDown,
            7 => JobState::Completed,
            8 => JobState::Terminated,
            9 => JobState::Killed,
            10 => JobState::Exception,
            11 => JobState::Service,
            _ => JobState::Unknown,
        }
    }
}

impl JobState {
    /// Check if job is still running.
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            JobState::New
                | JobState::Starting
                | JobState::Running
                | JobState::Suspended
                | JobState::ShuttingDown
        )
    }

    /// Check if job completed successfully.
    pub fn is_completed(&self) -> bool {
        *self == JobState::Completed
    }

    /// Check if job failed.
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            JobState::Terminated | JobState::Killed | JobState::Exception
        )
    }
}

impl fmt::Display for JobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobState::New => write!(f, "New"),
            JobState::Starting => write!(f, "Starting"),
            JobState::Running => write!(f, "Running"),
            JobState::Suspended => write!(f, "Suspended"),
            JobState::ShuttingDown => write!(f, "ShuttingDown"),
            JobState::Completed => write!(f, "Completed"),
            JobState::Terminated => write!(f, "Terminated"),
            JobState::Killed => write!(f, "Killed"),
            JobState::Exception => write!(f, "Exception"),
            JobState::Service => write!(f, "Service"),
            JobState::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Migration-specific error details.
#[derive(Debug, Clone)]
pub struct MigrationError {
    /// Source host.
    pub source_host: String,
    /// Destination host.
    pub destination_host: String,
    /// VM name being migrated.
    pub vm_name: String,
    /// Migration job ID if available.
    pub job_id: Option<String>,
    /// Percent complete when failed.
    pub percent_complete: Option<u32>,
    /// Error code from migration service.
    pub error_code: u32,
    /// Error description.
    pub error_description: String,
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Migration of VM '{}' from '{}' to '{}' failed (code {}): {}",
            self.vm_name,
            self.source_host,
            self.destination_host,
            self.error_code,
            self.error_description
        )?;
        if let Some(pct) = self.percent_complete {
            write!(f, " ({}% complete)", pct)?;
        }
        Ok(())
    }
}

/// Security-specific error details.
#[derive(Debug, Clone)]
pub struct SecurityError {
    /// Type of security operation that failed.
    pub operation: String,
    /// Error code.
    pub error_code: u32,
    /// Error description.
    pub error_description: String,
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Security operation '{}' failed (code {}): {}",
            self.operation, self.error_code, self.error_description
        )
    }
}

/// Hyper-V operation errors with typed context.
#[derive(Debug)]
pub enum Error {
    /// Failed to connect to WMI.
    #[cfg(windows)]
    WmiConnection(WinError),

    /// Failed to execute WMI query.
    #[cfg(windows)]
    WmiQuery { query: String, source: WinError },

    /// Failed to invoke WMI method.
    #[cfg(windows)]
    WmiMethod {
        class: &'static str,
        method: &'static str,
        source: WinError,
    },

    /// Failed to connect to remote machine.
    RemoteConnection {
        machine: String,
        message: String,
        failure_type: FailureType,
    },

    /// Authentication failed for remote connection.
    AuthenticationFailed {
        machine: String,
        username: String,
        message: String,
    },

    /// VM not found by name or ID.
    VmNotFound(String),

    /// Virtual switch not found.
    SwitchNotFound(String),

    /// VHD/VHDX file not found.
    VhdNotFound(String),

    /// Checkpoint not found.
    CheckpointNotFound { vm_name: String, checkpoint: String },

    /// Network adapter not found.
    NetworkAdapterNotFound { vm_name: String, adapter_id: String },

    /// Storage controller not found.
    ControllerNotFound {
        vm_name: String,
        controller_type: String,
        controller_number: u32,
    },

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

    /// Property not supported on this host/VM version.
    PropertyNotSupported { property: String, reason: String },

    /// WMI operation returned failure code.
    OperationFailed {
        operation: &'static str,
        return_value: u32,
        message: String,
        failure_type: FailureType,
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
        job_state: JobState,
    },

    /// Job timed out waiting for completion.
    JobTimeout {
        operation: &'static str,
        job_id: String,
        timeout: Duration,
        last_state: JobState,
        percent_complete: Option<u32>,
    },

    /// Migration operation failed.
    Migration(MigrationError),

    /// Security operation failed (TPM, SecureBoot, etc.).
    Security(SecurityError),

    /// Feature not available on this host.
    FeatureNotAvailable { feature: String, reason: String },

    /// VM version incompatible with requested operation.
    VmVersionIncompatible {
        vm_name: String,
        vm_version: String,
        required_version: String,
        operation: String,
    },

    /// GPU device not found.
    GpuNotFound(String),

    /// GPU partition unavailable.
    GpuPartitionUnavailable { gpu_id: String, message: String },

    /// DDA device not found or not compatible.
    DdaDeviceNotFound { location_path: String, message: String },

    /// DDA device already assigned.
    DdaDeviceAssigned {
        location_path: String,
        assigned_vm: String,
    },

    /// IO error (file operations, etc.).
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(windows)]
            Error::WmiConnection(e) => write!(f, "WMI connection failed: {e}"),
            #[cfg(windows)]
            Error::WmiQuery { query, source } => {
                write!(f, "WMI query failed: {query} - {source}")
            }
            #[cfg(windows)]
            Error::WmiMethod {
                class,
                method,
                source,
            } => {
                write!(f, "WMI method {class}.{method} failed: {source}")
            }
            Error::RemoteConnection {
                machine,
                message,
                failure_type,
            } => {
                write!(
                    f,
                    "Remote connection to '{}' failed ({}): {}",
                    machine, failure_type, message
                )
            }
            Error::AuthenticationFailed {
                machine,
                username,
                message,
            } => {
                write!(
                    f,
                    "Authentication failed for user '{}' on '{}': {}",
                    username, machine, message
                )
            }
            Error::VmNotFound(name) => write!(f, "VM not found: {name}"),
            Error::SwitchNotFound(name) => write!(f, "Virtual switch not found: {name}"),
            Error::VhdNotFound(path) => write!(f, "VHD not found: {path}"),
            Error::CheckpointNotFound {
                vm_name,
                checkpoint,
            } => {
                write!(
                    f,
                    "Checkpoint '{}' not found for VM '{}'",
                    checkpoint, vm_name
                )
            }
            Error::NetworkAdapterNotFound {
                vm_name,
                adapter_id,
            } => {
                write!(
                    f,
                    "Network adapter '{}' not found for VM '{}'",
                    adapter_id, vm_name
                )
            }
            Error::ControllerNotFound {
                vm_name,
                controller_type,
                controller_number,
            } => {
                write!(
                    f,
                    "{} controller {} not found for VM '{}'",
                    controller_type, controller_number, vm_name
                )
            }
            Error::InvalidState {
                vm_name,
                current,
                operation,
            } => {
                write!(f, "Cannot {operation} VM '{vm_name}' in state {current}")
            }
            Error::Validation { field, message } => {
                write!(f, "Validation failed for '{field}': {message}")
            }
            Error::MissingRequired(field) => {
                write!(f, "Required field missing: {field}")
            }
            Error::PropertyNotSupported { property, reason } => {
                write!(f, "Property '{}' not supported: {}", property, reason)
            }
            Error::OperationFailed {
                operation,
                return_value,
                message,
                failure_type,
            } => {
                write!(
                    f,
                    "Operation '{}' failed with code {} ({}): {}",
                    operation, return_value, failure_type, message
                )
            }
            Error::TypeConversion { property, expected } => {
                write!(f, "Cannot convert property '{property}' to {expected}")
            }
            Error::JobFailed {
                operation,
                error_code,
                error_description,
                job_state,
            } => {
                write!(
                    f,
                    "Job failed for '{}' in state {} (code {}): {}",
                    operation, job_state, error_code, error_description
                )
            }
            Error::JobTimeout {
                operation,
                job_id,
                timeout,
                last_state,
                percent_complete,
            } => {
                write!(
                    f,
                    "Job '{}' for '{}' timed out after {:?} in state {}",
                    job_id, operation, timeout, last_state
                )?;
                if let Some(pct) = percent_complete {
                    write!(f, " ({}% complete)", pct)?;
                }
                Ok(())
            }
            Error::Migration(e) => write!(f, "{}", e),
            Error::Security(e) => write!(f, "{}", e),
            Error::FeatureNotAvailable { feature, reason } => {
                write!(f, "Feature '{}' not available: {}", feature, reason)
            }
            Error::VmVersionIncompatible {
                vm_name,
                vm_version,
                required_version,
                operation,
            } => {
                write!(
                    f,
                    "VM '{}' version {} incompatible with '{}' (requires {})",
                    vm_name, vm_version, operation, required_version
                )
            }
            Error::GpuNotFound(id) => write!(f, "GPU not found: {}", id),
            Error::GpuPartitionUnavailable { gpu_id, message } => {
                write!(f, "GPU partition unavailable for '{}': {}", gpu_id, message)
            }
            Error::DdaDeviceNotFound {
                location_path,
                message,
            } => {
                write!(f, "DDA device not found '{}': {}", location_path, message)
            }
            Error::DdaDeviceAssigned {
                location_path,
                assigned_vm,
            } => {
                write!(
                    f,
                    "DDA device '{}' is already assigned to VM '{}'",
                    location_path, assigned_vm
                )
            }
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(windows)]
            Error::WmiConnection(e) => Some(e),
            #[cfg(windows)]
            Error::WmiQuery { source, .. } => Some(source),
            #[cfg(windows)]
            Error::WmiMethod { source, .. } => Some(source),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(windows)]
impl From<WinError> for Error {
    fn from(e: WinError) -> Self {
        Error::WmiConnection(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<MigrationError> for Error {
    fn from(e: MigrationError) -> Self {
        Error::Migration(e)
    }
}

impl From<SecurityError> for Error {
    fn from(e: SecurityError) -> Self {
        Error::Security(e)
    }
}

impl Error {
    /// Get the failure type classification for this error.
    pub fn failure_type(&self) -> FailureType {
        match self {
            #[cfg(windows)]
            Error::WmiConnection(_) => FailureType::Network,
            #[cfg(windows)]
            Error::WmiQuery { .. } => FailureType::Transient,
            #[cfg(windows)]
            Error::WmiMethod { .. } => FailureType::Unknown,
            Error::RemoteConnection { failure_type, .. } => *failure_type,
            Error::AuthenticationFailed { .. } => FailureType::AuthenticationFailed,
            Error::VmNotFound(_) => FailureType::Permanent,
            Error::SwitchNotFound(_) => FailureType::Permanent,
            Error::VhdNotFound(_) => FailureType::Permanent,
            Error::CheckpointNotFound { .. } => FailureType::Permanent,
            Error::NetworkAdapterNotFound { .. } => FailureType::Permanent,
            Error::ControllerNotFound { .. } => FailureType::Permanent,
            Error::InvalidState { .. } => FailureType::ResourceBusy,
            Error::Validation { .. } => FailureType::Configuration,
            Error::MissingRequired(_) => FailureType::Configuration,
            Error::PropertyNotSupported { .. } => FailureType::Permanent,
            Error::OperationFailed { failure_type, .. } => *failure_type,
            Error::TypeConversion { .. } => FailureType::Permanent,
            Error::JobFailed { .. } => FailureType::Unknown,
            Error::JobTimeout { .. } => FailureType::Transient,
            Error::Migration(_) => FailureType::Unknown,
            Error::Security(_) => FailureType::Permanent,
            Error::FeatureNotAvailable { .. } => FailureType::Permanent,
            Error::VmVersionIncompatible { .. } => FailureType::Permanent,
            Error::GpuNotFound(_) => FailureType::Permanent,
            Error::GpuPartitionUnavailable { .. } => FailureType::ResourceBusy,
            Error::DdaDeviceNotFound { .. } => FailureType::Permanent,
            Error::DdaDeviceAssigned { .. } => FailureType::ResourceBusy,
            Error::Io(_) => FailureType::Transient,
        }
    }

    /// Check if the error is transient and operation may succeed if retried.
    pub fn is_transient(&self) -> bool {
        matches!(
            self.failure_type(),
            FailureType::Transient | FailureType::ResourceBusy
        )
    }

    /// Check if the error is permanent and retrying will not help.
    pub fn is_permanent(&self) -> bool {
        self.failure_type() == FailureType::Permanent
    }

    /// Check if this is a resource busy error (retry after delay).
    pub fn is_resource_busy(&self) -> bool {
        self.failure_type() == FailureType::ResourceBusy
    }

    /// Create an operation failed error with unknown failure type.
    pub fn operation_failed(
        operation: &'static str,
        return_value: u32,
        message: impl Into<String>,
    ) -> Self {
        Error::OperationFailed {
            operation,
            return_value,
            message: message.into(),
            failure_type: FailureType::Unknown,
        }
    }

    /// Create an operation failed error with specific failure type.
    pub fn operation_failed_with_type(
        operation: &'static str,
        return_value: u32,
        message: impl Into<String>,
        failure_type: FailureType,
    ) -> Self {
        Error::OperationFailed {
            operation,
            return_value,
            message: message.into(),
            failure_type,
        }
    }

    /// Create a job failed error.
    pub fn job_failed(
        operation: &'static str,
        error_code: u32,
        error_description: impl Into<String>,
        job_state: JobState,
    ) -> Self {
        Error::JobFailed {
            operation,
            error_code,
            error_description: error_description.into(),
            job_state,
        }
    }

    /// Create a job timeout error.
    pub fn job_timeout(
        operation: &'static str,
        job_id: impl Into<String>,
        timeout: Duration,
        last_state: JobState,
        percent_complete: Option<u32>,
    ) -> Self {
        Error::JobTimeout {
            operation,
            job_id: job_id.into(),
            timeout,
            last_state,
            percent_complete,
        }
    }
}

/// Result type for Hyper-V operations.
pub type Result<T> = core::result::Result<T, Error>;
