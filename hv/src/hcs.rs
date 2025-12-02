//! HCS (Host Compute Service) wrapper utilities
//!
//! This module provides safe wrappers around Windows HCS APIs for compute system operations.
//! Uses windows-rs bindings directly instead of hcs-rs.

use crate::error::{HvError, Result};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::time::Duration;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::HostComputeSystem::{
    HcsCloseComputeSystem, HcsCloseOperation, HcsCreateComputeSystem, HcsCreateOperation,
    HcsEnumerateComputeSystems, HcsGetComputeSystemProperties, HcsGetOperationResult,
    HcsOpenComputeSystem, HcsPauseComputeSystem, HcsResumeComputeSystem, HcsSaveComputeSystem,
    HcsShutDownComputeSystem, HcsStartComputeSystem, HcsTerminateComputeSystem,
    HcsWaitForOperationResult, HCS_OPERATION, HCS_SYSTEM,
};

/// Default timeout for HCS operations (5 minutes)
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// Convert Rust string to wide string (UTF-16 null-terminated)
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Convert wide string pointer to Rust String
unsafe fn from_wide(ptr: PWSTR) -> String {
    if ptr.0.is_null() {
        return String::new();
    }
    let mut len = 0;
    while *ptr.0.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr.0, len);
    String::from_utf16_lossy(slice)
}

/// Free a wide string allocated by HCS
unsafe fn free_hcs_string(ptr: PWSTR) {
    if !ptr.0.is_null() {
        // HCS strings are allocated with CoTaskMemAlloc, free with CoTaskMemFree
        windows::Win32::System::Com::CoTaskMemFree(Some(ptr.0 as *const _));
    }
}

/// RAII wrapper for HCS_OPERATION
pub struct HcsOperation(HCS_OPERATION);

impl HcsOperation {
    /// Create a new HCS operation
    pub fn new() -> Result<Self> {
        unsafe {
            let op = HcsCreateOperation(None, None);
            if op.is_invalid() {
                return Err(HvError::HcsError(
                    "Failed to create HCS operation".to_string(),
                ));
            }
            Ok(HcsOperation(op))
        }
    }

    /// Get the raw handle
    pub fn handle(&self) -> HCS_OPERATION {
        self.0
    }

    /// Wait for operation result with timeout
    pub fn wait(&self, timeout_ms: u32) -> Result<String> {
        unsafe {
            let mut result_doc: PWSTR = PWSTR::null();
            let hr = HcsWaitForOperationResult(self.0, timeout_ms, Some(&mut result_doc));

            if hr.is_err() {
                if !result_doc.0.is_null() {
                    free_hcs_string(result_doc);
                }
                return Err(HvError::HcsError(format!("HCS operation failed: {:?}", hr)));
            }

            let result = if result_doc.0.is_null() {
                String::new()
            } else {
                let s = from_wide(result_doc);
                free_hcs_string(result_doc);
                s
            };

            Ok(result)
        }
    }

    /// Get operation result without waiting
    pub fn result(&self) -> Result<String> {
        unsafe {
            let mut result_doc: PWSTR = PWSTR::null();
            let hr = HcsGetOperationResult(self.0, Some(&mut result_doc));

            if hr.is_err() {
                if !result_doc.0.is_null() {
                    free_hcs_string(result_doc);
                }
                return Err(HvError::HcsError(format!(
                    "Failed to get operation result: {:?}",
                    hr
                )));
            }

            let result = if result_doc.0.is_null() {
                String::new()
            } else {
                let s = from_wide(result_doc);
                free_hcs_string(result_doc);
                s
            };

            Ok(result)
        }
    }
}

impl Drop for HcsOperation {
    fn drop(&mut self) {
        unsafe {
            HcsCloseOperation(self.0);
        }
    }
}

/// RAII wrapper for HCS_SYSTEM (compute system handle)
pub struct HcsSystem(HCS_SYSTEM);

impl HcsSystem {
    /// Get the raw handle
    pub fn handle(&self) -> HCS_SYSTEM {
        self.0
    }

    /// Check if handle is valid
    pub fn is_valid(&self) -> bool {
        !self.0.is_invalid()
    }

    /// Start the compute system
    pub fn start(&self, options: Option<&str>) -> Result<()> {
        let op = HcsOperation::new()?;
        let options_wide = options.map(to_wide);
        let options_ptr = options_wide
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        unsafe {
            let hr = HcsStartComputeSystem(self.0, op.handle(), options_ptr);
            if hr.is_err() {
                return Err(HvError::HcsError(format!(
                    "Failed to start compute system: {:?}",
                    hr
                )));
            }
        }

        op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;
        Ok(())
    }

    /// Shutdown the compute system gracefully
    pub fn shutdown(&self, options: Option<&str>) -> Result<()> {
        let op = HcsOperation::new()?;
        let options_wide = options.map(to_wide);
        let options_ptr = options_wide
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        unsafe {
            let hr = HcsShutDownComputeSystem(self.0, op.handle(), options_ptr);
            if hr.is_err() {
                return Err(HvError::HcsError(format!(
                    "Failed to shutdown compute system: {:?}",
                    hr
                )));
            }
        }

        op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;
        Ok(())
    }

    /// Terminate the compute system forcefully
    pub fn terminate(&self) -> Result<()> {
        let op = HcsOperation::new()?;

        unsafe {
            let hr = HcsTerminateComputeSystem(self.0, op.handle(), PCWSTR::null());
            if hr.is_err() {
                return Err(HvError::HcsError(format!(
                    "Failed to terminate compute system: {:?}",
                    hr
                )));
            }
        }

        op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;
        Ok(())
    }

    /// Pause the compute system
    pub fn pause(&self, options: Option<&str>) -> Result<()> {
        let op = HcsOperation::new()?;
        let options_wide = options.map(to_wide);
        let options_ptr = options_wide
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        unsafe {
            let hr = HcsPauseComputeSystem(self.0, op.handle(), options_ptr);
            if hr.is_err() {
                return Err(HvError::HcsError(format!(
                    "Failed to pause compute system: {:?}",
                    hr
                )));
            }
        }

        op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;
        Ok(())
    }

    /// Resume the compute system
    pub fn resume(&self) -> Result<()> {
        let op = HcsOperation::new()?;

        unsafe {
            let hr = HcsResumeComputeSystem(self.0, op.handle(), PCWSTR::null());
            if hr.is_err() {
                return Err(HvError::HcsError(format!(
                    "Failed to resume compute system: {:?}",
                    hr
                )));
            }
        }

        op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;
        Ok(())
    }

    /// Save the compute system state
    pub fn save(&self, options: Option<&str>) -> Result<()> {
        let op = HcsOperation::new()?;
        let options_wide = options.map(to_wide);
        let options_ptr = options_wide
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        unsafe {
            let hr = HcsSaveComputeSystem(self.0, op.handle(), options_ptr);
            if hr.is_err() {
                return Err(HvError::HcsError(format!(
                    "Failed to save compute system: {:?}",
                    hr
                )));
            }
        }

        op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;
        Ok(())
    }

    /// Get compute system properties
    pub fn get_properties(&self, query: Option<&str>) -> Result<Option<String>> {
        let op = HcsOperation::new()?;
        let query_wide = query.map(to_wide);
        let query_ptr = query_wide
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        unsafe {
            let hr = HcsGetComputeSystemProperties(self.0, op.handle(), query_ptr);
            if hr.is_err() {
                return Err(HvError::HcsError(format!(
                    "Failed to get properties: {:?}",
                    hr
                )));
            }
        }

        let result = op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

impl Drop for HcsSystem {
    fn drop(&mut self) {
        unsafe {
            HcsCloseComputeSystem(self.0);
        }
    }
}

/// HCS compute system info from enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComputeSystemInfo {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub system_type: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub runtime_os_type: Option<String>,
}

/// Enumerate all compute systems (VMs and containers)
pub fn enumerate_compute_systems(query: Option<&str>) -> Result<Vec<ComputeSystemInfo>> {
    let query_json = query.unwrap_or(r#"{"Owners": null}"#);
    let query_wide = to_wide(query_json);

    let op = HcsOperation::new()?;

    unsafe {
        let hr = HcsEnumerateComputeSystems(PCWSTR(query_wide.as_ptr()), op.handle());
        if hr.is_err() {
            return Err(HvError::HcsError(format!(
                "Failed to enumerate compute systems: {:?}",
                hr
            )));
        }
    }

    let result = op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;

    if result.is_empty() || result == "null" {
        return Ok(Vec::new());
    }

    let systems: Vec<ComputeSystemInfo> =
        serde_json::from_str(&result).map_err(|e| HvError::JsonError(e.to_string()))?;

    Ok(systems)
}

/// Open an existing compute system by ID
pub fn open_compute_system(id: &str) -> Result<HcsSystem> {
    let id_wide = to_wide(id);

    let system = unsafe {
        let result = HcsOpenComputeSystem(
            PCWSTR(id_wide.as_ptr()),
            0x1F0FFF, // GENERIC_ALL access
        );
        match result {
            Ok(sys) => sys,
            Err(e) => {
                return Err(HvError::HcsError(format!(
                    "Failed to open compute system {}: {:?}",
                    id, e
                )));
            }
        }
    };

    Ok(HcsSystem(system))
}

/// Create a new compute system
pub fn create_compute_system(id: &str, configuration: &str) -> Result<HcsSystem> {
    let id_wide = to_wide(id);
    let config_wide = to_wide(configuration);
    let op = HcsOperation::new()?;

    let system = unsafe {
        let result = HcsCreateComputeSystem(
            PCWSTR(id_wide.as_ptr()),
            PCWSTR(config_wide.as_ptr()),
            op.handle(),
            None, // No security descriptor
        );
        match result {
            Ok(sys) => sys,
            Err(e) => {
                return Err(HvError::HcsError(format!(
                    "Failed to create compute system {}: {:?}",
                    id, e
                )));
            }
        }
    };

    // Wait for creation to complete
    op.wait(DEFAULT_TIMEOUT.as_millis() as u32)?;

    Ok(HcsSystem(system))
}

/// HCS VM configuration schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VmConfiguration {
    #[serde(rename = "SchemaVersion")]
    pub schema_version: SchemaVersion,
    pub owner: String,
    #[serde(rename = "ShouldTerminateOnLastHandleClosed")]
    pub should_terminate_on_last_handle_closed: bool,
    pub virtual_machine: VirtualMachineConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VirtualMachineConfig {
    #[serde(rename = "StopOnReset")]
    pub stop_on_reset: bool,
    pub chipset: ChipsetConfig,
    pub compute_topology: ComputeTopologyConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<DevicesConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChipsetConfig {
    pub uefi: Option<UefiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UefiConfig {
    #[serde(rename = "BootThis")]
    pub boot_this: Option<BootConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BootConfig {
    pub device_type: String,
    pub device_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComputeTopologyConfig {
    pub memory: MemoryConfig,
    pub processor: ProcessorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MemoryConfig {
    #[serde(rename = "SizeInMB")]
    pub size_in_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProcessorConfig {
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DevicesConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scsi: Option<ScsiConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_adapters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<GpuConfiguration>,
}

/// GPU configuration for HCS VM
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GpuConfiguration {
    /// Assignment mode for GPU resources
    #[serde(rename = "AssignmentMode")]
    pub assignment_mode: GpuAssignmentMode,
    /// Request for GPU assignment
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "AssignmentRequest")]
    pub assignment_request: Option<GpuAssignmentRequest>,
    /// Allow vendor extension (for GPU-PV)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "AllowVendorExtension")]
    pub allow_vendor_extension: Option<bool>,
}

/// GPU assignment mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuAssignmentMode {
    /// GPU disabled
    Disabled,
    /// Default GPU assignment
    Default,
    /// Specific GPU list assignment
    List,
    /// Mirror host GPU configuration
    Mirror,
}

/// GPU assignment request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GpuAssignmentRequest {
    /// List of GPU device instance paths
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Gpus")]
    pub gpus: Option<Vec<GpuDevice>>,
}

/// GPU device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GpuDevice {
    /// Device instance path
    #[serde(rename = "DeviceInstancePath")]
    pub device_instance_path: String,
    /// Minimum partition count
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MinPartition")]
    pub min_partition: Option<u64>,
    /// Maximum partition count
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "MaxPartition")]
    pub max_partition: Option<u64>,
    /// Optimal partition count
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "OptimalPartition")]
    pub optimal_partition: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ScsiConfig {
    pub attachments: std::collections::HashMap<String, ScsiAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ScsiAttachment {
    #[serde(rename = "Type")]
    pub attachment_type: String,
    pub path: String,
}

impl VmConfiguration {
    /// Create a new Gen2 VM configuration
    pub fn new_gen2(name: &str, memory_mb: u64, cpu_count: u32) -> Self {
        VmConfiguration {
            schema_version: SchemaVersion { major: 2, minor: 1 },
            owner: name.to_string(),
            should_terminate_on_last_handle_closed: true,
            virtual_machine: VirtualMachineConfig {
                stop_on_reset: true,
                chipset: ChipsetConfig {
                    uefi: Some(UefiConfig { boot_this: None }),
                },
                compute_topology: ComputeTopologyConfig {
                    memory: MemoryConfig {
                        size_in_mb: memory_mb,
                    },
                    processor: ProcessorConfig { count: cpu_count },
                },
                devices: None,
            },
        }
    }

    /// Add a VHD to the configuration
    pub fn with_vhd(mut self, vhd_path: &str) -> Self {
        let mut attachments = std::collections::HashMap::new();
        attachments.insert(
            "0".to_string(),
            ScsiAttachment {
                attachment_type: "VirtualHardDisk".to_string(),
                path: vhd_path.to_string(),
            },
        );

        self.virtual_machine.devices = Some(DevicesConfig {
            scsi: Some(ScsiConfig { attachments }),
            network_adapters: None,
            gpu: None,
        });

        self
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| HvError::JsonError(e.to_string()))
    }
}
