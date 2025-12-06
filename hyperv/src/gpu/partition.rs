//! GPU-P (GPU Partitioning) operations for Hyper-V.
//!
//! GPU-P allows sharing a physical GPU among multiple VMs through partitioning.
//! Each partition gets a dedicated slice of GPU resources (VRAM, encode/decode engines).

use super::types::{
    GpuPartition, GpuPartitionSettings, GpuPartitionStatus, PartitionableGpu, VmGpuSummary,
};
use crate::error::{Error, FailureType, Result};

#[cfg(windows)]
use crate::wmi::{JobWaiter, WbemClassObjectExt, WmiConnection};
#[cfg(windows)]
use std::time::Duration;

/// GPU partition manager for host-level GPU-P operations.
pub struct GpuPartitionManager<'a> {
    #[cfg(windows)]
    conn: &'a WmiConnection,
    #[cfg(not(windows))]
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> GpuPartitionManager<'a> {
    /// Create a new GPU partition manager.
    #[cfg(windows)]
    pub fn new(conn: &'a WmiConnection) -> Self {
        Self { conn }
    }

    /// Create a new GPU partition manager (non-Windows stub).
    #[cfg(not(windows))]
    pub fn new(_conn: &'a ()) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// List all partitionable GPUs on the host.
    #[cfg(windows)]
    pub fn list_partitionable_gpus(&self) -> Result<Vec<PartitionableGpu>> {
        let query = "SELECT * FROM Msvm_PartitionableGpu";
        let results = self.conn.query(query)?;

        let mut gpus = Vec::new();
        for obj in results {
            let gpu = self.parse_partitionable_gpu(&obj)?;
            gpus.push(gpu);
        }
        Ok(gpus)
    }

    /// List all partitionable GPUs on the host (non-Windows stub).
    #[cfg(not(windows))]
    pub fn list_partitionable_gpus(&self) -> Result<Vec<PartitionableGpu>> {
        Ok(Vec::new())
    }

    /// Get a specific partitionable GPU by ID.
    #[cfg(windows)]
    pub fn get_gpu(&self, gpu_id: &str) -> Result<PartitionableGpu> {
        let query = format!(
            "SELECT * FROM Msvm_PartitionableGpu WHERE Name = '{}'",
            gpu_id.replace('\'', "''")
        );
        let result = self.conn.query_first(&query)?;

        match result {
            Some(obj) => self.parse_partitionable_gpu(&obj),
            None => Err(Error::OperationFailed {
                failure_type: FailureType::Permanent,
                operation: "GetPartitionableGpu",
                return_value: 0,
                message: format!("GPU not found: {}", gpu_id),
            }),
        }
    }

    /// Get a specific partitionable GPU by ID (non-Windows stub).
    #[cfg(not(windows))]
    pub fn get_gpu(&self, gpu_id: &str) -> Result<PartitionableGpu> {
        Err(Error::OperationFailed {
            failure_type: FailureType::Permanent,
            operation: "GetPartitionableGpu",
            return_value: 0,
            message: format!("GPU not found: {}", gpu_id),
        })
    }

    /// Get GPU partition availability status.
    ///
    /// Returns (total_partitions, adapters_assigned, partitions_in_use).
    #[cfg(windows)]
    pub fn get_partition_availability(&self) -> Result<(u32, u32, u32)> {
        let gpus = self.list_partitionable_gpus()?;

        let total_partitions: u32 = gpus.iter().map(|g| g.total_partition_count).sum();
        let in_use: u32 = gpus.iter().map(|g| g.partitions_in_use).sum();
        let adapters_assigned = gpus.iter().filter(|g| g.partitions_in_use > 0).count() as u32;

        Ok((total_partitions, adapters_assigned, in_use))
    }

    /// Get GPU partition availability status (non-Windows stub).
    #[cfg(not(windows))]
    pub fn get_partition_availability(&self) -> Result<(u32, u32, u32)> {
        Ok((0, 0, 0))
    }

    /// Set the partition count for a GPU.
    ///
    /// This changes how the physical GPU is divided among potential VMs.
    #[cfg(windows)]
    pub fn set_partition_count(&self, gpu_id: &str, count: u32) -> Result<()> {
        // Get the GPU
        let gpu = self.get_gpu(gpu_id)?;

        // Validate partition count
        if count < gpu.min_partition_count || count > gpu.max_partition_count {
            return Err(Error::Validation {
                field: "partition_count",
                message: format!(
                    "Partition count {} is out of range [{}, {}]",
                    count, gpu.min_partition_count, gpu.max_partition_count
                ),
            });
        }

        // Get the GPU object and modify its partition count
        let query = format!(
            "SELECT * FROM Msvm_PartitionableGpu WHERE Name = '{}'",
            gpu_id.replace('\'', "''")
        );
        let gpu_obj = self
            .conn
            .query_first(&query)?
            .ok_or_else(|| Error::OperationFailed {
                failure_type: FailureType::Permanent,
                operation: "SetPartitionCount",
                return_value: 0,
                message: format!("GPU not found: {}", gpu_id),
            })?;

        gpu_obj.put_u32("CurrentPartitionCount", count)?;

        // The change takes effect after the next reboot or when all partitions are released
        Ok(())
    }

    /// Set the partition count for a GPU (non-Windows stub).
    #[cfg(not(windows))]
    pub fn set_partition_count(&self, _gpu_id: &str, _count: u32) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "GPU-P".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }

    /// Check if a GPU's partition count matches the expected value.
    #[cfg(windows)]
    pub fn is_partition_count_equal(&self, gpu_id: &str, expected: u32) -> Result<bool> {
        let gpu = self.get_gpu(gpu_id)?;
        Ok(gpu.total_partition_count == expected)
    }

    /// Check if a GPU's partition count matches the expected value (non-Windows stub).
    #[cfg(not(windows))]
    pub fn is_partition_count_equal(&self, _gpu_id: &str, _expected: u32) -> Result<bool> {
        Ok(false)
    }

    /// Get a GPU property by name.
    #[cfg(windows)]
    pub fn get_gpu_property(&self, gpu_id: &str, property_name: &str) -> Result<Option<String>> {
        let query = format!(
            "SELECT {} FROM Msvm_PartitionableGpu WHERE Name = '{}'",
            property_name,
            gpu_id.replace('\'', "''")
        );
        let result = self.conn.query_first(&query)?;

        match result {
            Some(obj) => obj.get_string_prop(property_name),
            None => Err(Error::OperationFailed {
                failure_type: FailureType::Permanent,
                operation: "GetGpuProperty",
                return_value: 0,
                message: format!("GPU not found: {}", gpu_id),
            }),
        }
    }

    /// Get a GPU property by name (non-Windows stub).
    #[cfg(not(windows))]
    pub fn get_gpu_property(&self, _gpu_id: &str, _property_name: &str) -> Result<Option<String>> {
        Ok(None)
    }

    #[cfg(windows)]
    fn parse_partitionable_gpu(
        &self,
        obj: &windows::Win32::System::Wmi::IWbemClassObject,
    ) -> Result<PartitionableGpu> {
        let id = obj.get_string_prop("Name")?.unwrap_or_default();
        let name = obj
            .get_string_prop("ElementName")?
            .unwrap_or_else(|| id.clone());

        Ok(PartitionableGpu {
            id,
            name,
            driver_version: obj.get_string_prop("DriverVersion")?,
            total_partition_count: obj.get_u32("CurrentPartitionCount")?.unwrap_or(0),
            partitions_in_use: obj.get_u32("PartitionsInUse")?.unwrap_or(0),
            min_partition_count: obj.get_u32("MinPartitionCount")?.unwrap_or(1),
            max_partition_count: obj.get_u32("MaxPartitionCount")?.unwrap_or(1),
            optimal_partition_count: obj.get_u32("OptimalPartitionCount")?.unwrap_or(1),
            is_partitionable: obj.get_bool("IsPartitionable")?.unwrap_or(false),
            status: GpuPartitionStatus::from(obj.get_u16("OperationalStatus")?.unwrap_or(0)),
            vram_per_partition_mb: obj.get_u64("DedicatedVideoMemoryPerPartition")?,
            encode_per_partition: obj.get_u32("EncodeCapabilityPerPartition")?,
            decode_per_partition: obj.get_u32("DecodeCapabilityPerPartition")?,
            compute_per_partition: obj.get_u32("ComputeCapabilityPerPartition")?,
        })
    }
}

/// GPU partition operations for a specific VM.
pub struct VmGpuPartition<'a> {
    #[cfg(windows)]
    conn: &'a WmiConnection,
    #[cfg(not(windows))]
    _phantom: std::marker::PhantomData<&'a ()>,
    vm_id: String,
}

impl<'a> VmGpuPartition<'a> {
    /// Create GPU partition operations for a VM.
    #[cfg(windows)]
    pub fn new(conn: &'a WmiConnection, vm_id: impl Into<String>) -> Self {
        Self {
            conn,
            vm_id: vm_id.into(),
        }
    }

    /// Create GPU partition operations for a VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn new(_conn: &'a (), vm_id: impl Into<String>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            vm_id: vm_id.into(),
        }
    }

    /// Get the VM ID.
    pub fn vm_id(&self) -> &str {
        &self.vm_id
    }

    /// List GPU partitions assigned to this VM.
    #[cfg(windows)]
    pub fn list_partitions(&self) -> Result<Vec<GpuPartition>> {
        // Query GPU partition settings for this VM
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SystemDevice \
             ResultClass=Msvm_GpuPartition",
            self.vm_id.replace('\'', "''")
        );
        let results = self.conn.query(&query)?;

        let mut partitions = Vec::new();
        for obj in results {
            let partition = self.parse_gpu_partition(&obj)?;
            partitions.push(partition);
        }
        Ok(partitions)
    }

    /// List GPU partitions assigned to this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn list_partitions(&self) -> Result<Vec<GpuPartition>> {
        Ok(Vec::new())
    }

    /// Get a summary of GPU assignments for this VM.
    #[cfg(windows)]
    pub fn get_summary(&self) -> Result<VmGpuSummary> {
        let partitions = self.list_partitions()?;

        Ok(VmGpuSummary {
            partition_count: partitions.len() as u32,
            dda_count: 0, // DDA is handled separately
            partitions,
            dda_devices: Vec::new(),
        })
    }

    /// Get a summary of GPU assignments for this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn get_summary(&self) -> Result<VmGpuSummary> {
        Ok(VmGpuSummary::default())
    }

    /// Add a GPU partition to this VM.
    #[cfg(windows)]
    pub fn add_partition(&self, settings: &GpuPartitionSettings) -> Result<()> {
        // Get VSMS
        let vsms = self.conn.get_singleton("Msvm_VirtualSystemManagementService")?;
        let vsms_path = vsms.get_path()?;

        // Get current VSSD
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState \
             ResultClass=Msvm_VirtualSystemSettingData",
            self.vm_id.replace('\'', "''")
        );
        let vssd_results = self.conn.query(&query)?;
        if vssd_results.is_empty() {
            return Err(Error::VmNotFound(self.vm_id.clone()));
        }
        let vssd = &vssd_results[0];
        let vssd_path = vssd.get_path()?;

        // Get default GPU partition setting data
        let gpu_default = self
            .conn
            .get_default_resource("Microsoft:Hyper-V:Gpu Partition")?;

        // Configure GPU partition settings
        gpu_default.put_string("HostResource", &settings.gpu_id)?;

        if settings.min_vram_mb > 0 {
            gpu_default.put_u64("MinimumVideoMemory", settings.min_vram_mb * 1024 * 1024)?;
        }
        if settings.max_vram_mb > 0 {
            gpu_default.put_u64("MaximumVideoMemory", settings.max_vram_mb * 1024 * 1024)?;
        }
        if settings.optimal_vram_mb > 0 {
            gpu_default.put_u64("OptimalVideoMemory", settings.optimal_vram_mb * 1024 * 1024)?;
        }

        if settings.min_encode > 0 {
            gpu_default.put_u32("MinimumEncodeCap", settings.min_encode)?;
        }
        if settings.max_encode > 0 {
            gpu_default.put_u32("MaximumEncodeCap", settings.max_encode)?;
        }
        if settings.optimal_encode > 0 {
            gpu_default.put_u32("OptimalEncodeCap", settings.optimal_encode)?;
        }

        if settings.min_decode > 0 {
            gpu_default.put_u32("MinimumDecodeCap", settings.min_decode)?;
        }
        if settings.max_decode > 0 {
            gpu_default.put_u32("MaximumDecodeCap", settings.max_decode)?;
        }
        if settings.optimal_decode > 0 {
            gpu_default.put_u32("OptimalDecodeCap", settings.optimal_decode)?;
        }

        if settings.min_compute > 0 {
            gpu_default.put_u32("MinimumComputeCap", settings.min_compute)?;
        }
        if settings.max_compute > 0 {
            gpu_default.put_u32("MaximumComputeCap", settings.max_compute)?;
        }
        if settings.optimal_compute > 0 {
            gpu_default.put_u32("OptimalComputeCap", settings.optimal_compute)?;
        }

        // Set parent to VSSD
        gpu_default.put_string("Parent", &vssd_path)?;

        let gpu_text = gpu_default.get_text()?;

        // Call AddResourceSettings
        let in_params = self
            .conn
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params.put_string("AffectedConfiguration", &vssd_path)?;
        in_params.put_string_array("ResourceSettings", &[&gpu_text])?;

        let out_params = self
            .conn
            .exec_method(&vsms_path, "AddResourceSettings", Some(&in_params))?;

        // Check result
        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);
        match return_value {
            0 => Ok(()),
            4096 => {
                let job_path = out_params.get_string_prop("Job")?.ok_or_else(|| {
                    Error::operation_failed("AddResourceSettings", 4096, "No job path returned")
                })?;
                let waiter = JobWaiter::with_timeout(self.conn, Duration::from_secs(60));
                waiter.wait_for_job(&job_path, "AddGpuPartition")?;
                Ok(())
            }
            code => Err(Error::operation_failed(
                "AddResourceSettings",
                code,
                "Failed to add GPU partition",
            )),
        }
    }

    /// Add a GPU partition to this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn add_partition(&self, _settings: &GpuPartitionSettings) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "GPU-P".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }

    /// Remove a GPU partition from this VM.
    #[cfg(windows)]
    pub fn remove_partition(&self, partition_instance_id: &str) -> Result<()> {
        // Get VSMS
        let vsms = self.conn.get_singleton("Msvm_VirtualSystemManagementService")?;
        let vsms_path = vsms.get_path()?;

        // Get the GPU partition object
        let query = format!(
            "SELECT * FROM Msvm_GpuPartitionSettingData WHERE InstanceID = '{}'",
            partition_instance_id.replace('\'', "''")
        );
        let result = self.conn.query_first(&query)?;
        let partition_obj = result.ok_or_else(|| Error::OperationFailed {
            failure_type: FailureType::Permanent,
            operation: "RemoveGpuPartition",
            return_value: 0,
            message: format!("GPU partition not found: {}", partition_instance_id),
        })?;
        let partition_path = partition_obj.get_path()?;

        // Call RemoveResourceSettings
        let in_params = self.conn.get_method_params(
            "Msvm_VirtualSystemManagementService",
            "RemoveResourceSettings",
        )?;
        in_params.put_string_array("ResourceSettings", &[&partition_path])?;

        let out_params = self
            .conn
            .exec_method(&vsms_path, "RemoveResourceSettings", Some(&in_params))?;

        // Check result
        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);
        match return_value {
            0 => Ok(()),
            4096 => {
                let job_path = out_params.get_string_prop("Job")?.ok_or_else(|| {
                    Error::operation_failed("RemoveResourceSettings", 4096, "No job path returned")
                })?;
                let waiter = JobWaiter::with_timeout(self.conn, Duration::from_secs(60));
                waiter.wait_for_job(&job_path, "RemoveGpuPartition")?;
                Ok(())
            }
            code => Err(Error::operation_failed(
                "RemoveResourceSettings",
                code,
                "Failed to remove GPU partition",
            )),
        }
    }

    /// Remove a GPU partition from this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn remove_partition(&self, _partition_instance_id: &str) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "GPU-P".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }

    /// Remove all GPU partitions from this VM.
    #[cfg(windows)]
    pub fn remove_all_partitions(&self) -> Result<u32> {
        let partitions = self.list_partitions()?;
        let count = partitions.len() as u32;

        for partition in partitions {
            self.remove_partition(&partition.instance_id)?;
        }

        Ok(count)
    }

    /// Remove all GPU partitions from this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn remove_all_partitions(&self) -> Result<u32> {
        Ok(0)
    }

    #[cfg(windows)]
    fn parse_gpu_partition(
        &self,
        obj: &windows::Win32::System::Wmi::IWbemClassObject,
    ) -> Result<GpuPartition> {
        let instance_id = obj.get_string_prop("InstanceID")?.unwrap_or_default();
        let host_resource = obj.get_string_array("HostResource")?;
        let gpu_id = host_resource
            .and_then(|arr| arr.first().cloned())
            .unwrap_or_default();

        Ok(GpuPartition {
            instance_id,
            gpu_id: gpu_id.clone(),
            gpu_name: gpu_id, // Will be enriched separately if needed
            partition_index: obj.get_u32("PartitionId")?.unwrap_or(0),
            vm_id: self.vm_id.clone(),
            vram_mb: obj.get_u64("CurrentVideoMemory")?.map(|v| v / 1024 / 1024),
            encode_engines: obj.get_u32("CurrentEncodeCap")?,
            decode_engines: obj.get_u32("CurrentDecodeCap")?,
            compute_engines: obj.get_u32("CurrentComputeCap")?,
        })
    }
}

/// Validate GPU partition settings before assignment.
#[cfg(windows)]
pub fn validate_gpu_partition_settings(
    conn: &WmiConnection,
    settings: &GpuPartitionSettings,
) -> Result<bool> {
    let manager = GpuPartitionManager::new(conn);
    let gpu = manager.get_gpu(&settings.gpu_id)?;

    // Check if GPU has available partitions
    if !gpu.has_available_partitions() {
        return Err(Error::OperationFailed {
            failure_type: FailureType::ResourceBusy,
            operation: "ValidateGpuPartitionSettings",
            return_value: 0,
            message: format!("No partitions available on GPU: {}", settings.gpu_id),
        });
    }

    // Check VRAM requirements
    if let Some(vram_per_partition) = gpu.vram_per_partition_mb {
        if settings.min_vram_mb > vram_per_partition {
            return Err(Error::Validation {
                field: "min_vram_mb",
                message: format!(
                    "Requested minimum VRAM {} MB exceeds available {} MB per partition",
                    settings.min_vram_mb, vram_per_partition
                ),
            });
        }
    }

    Ok(true)
}

/// Validate GPU partition settings before assignment (non-Windows stub).
#[cfg(not(windows))]
pub fn validate_gpu_partition_settings(
    _conn: &(),
    _settings: &GpuPartitionSettings,
) -> Result<bool> {
    Err(Error::FeatureNotAvailable {
        feature: "GPU-P".to_string(),
        reason: "Not available on this platform".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_partition_settings() {
        let settings = GpuPartitionSettings::builder()
            .gpu_id("GPU-123")
            .vram(512, 2048, 1024)
            .build();

        assert_eq!(settings.gpu_id, "GPU-123");
        assert_eq!(settings.min_vram_mb, 512);
        assert_eq!(settings.max_vram_mb, 2048);
    }

    #[test]
    fn test_gpu_partition_settings_for_gpu() {
        let settings = GpuPartitionSettings::for_gpu("GPU-456");
        assert_eq!(settings.gpu_id, "GPU-456");
        assert_eq!(settings.min_vram_mb, 0); // Default
    }
}
