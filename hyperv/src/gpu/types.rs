//! GPU-related types for Hyper-V.

use std::fmt;

/// GPU partition adapter status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum GpuPartitionStatus {
    /// Unknown status.
    Unknown = 0,
    /// OK - GPU partition is available.
    Ok = 1,
    /// Error state.
    Error = 2,
    /// Degraded performance.
    Degraded = 3,
    /// GPU partition is in use.
    InUse = 4,
    /// Starting up.
    Starting = 5,
    /// Stopping.
    Stopping = 6,
    /// Service mode.
    Service = 7,
}

impl From<u16> for GpuPartitionStatus {
    fn from(value: u16) -> Self {
        match value {
            1 => GpuPartitionStatus::Ok,
            2 => GpuPartitionStatus::Error,
            3 => GpuPartitionStatus::Degraded,
            4 => GpuPartitionStatus::InUse,
            5 => GpuPartitionStatus::Starting,
            6 => GpuPartitionStatus::Stopping,
            7 => GpuPartitionStatus::Service,
            _ => GpuPartitionStatus::Unknown,
        }
    }
}

impl fmt::Display for GpuPartitionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuPartitionStatus::Unknown => write!(f, "Unknown"),
            GpuPartitionStatus::Ok => write!(f, "OK"),
            GpuPartitionStatus::Error => write!(f, "Error"),
            GpuPartitionStatus::Degraded => write!(f, "Degraded"),
            GpuPartitionStatus::InUse => write!(f, "In Use"),
            GpuPartitionStatus::Starting => write!(f, "Starting"),
            GpuPartitionStatus::Stopping => write!(f, "Stopping"),
            GpuPartitionStatus::Service => write!(f, "Service"),
        }
    }
}

/// DDA device assignment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum DdaDeviceStatus {
    /// Unknown status.
    Unknown = 0,
    /// Device is available for assignment.
    Available = 1,
    /// Device is assigned to a VM.
    Assigned = 2,
    /// Device is not compatible with DDA.
    NotCompatible = 3,
    /// Device is in error state.
    Error = 4,
}

impl From<u16> for DdaDeviceStatus {
    fn from(value: u16) -> Self {
        match value {
            1 => DdaDeviceStatus::Available,
            2 => DdaDeviceStatus::Assigned,
            3 => DdaDeviceStatus::NotCompatible,
            4 => DdaDeviceStatus::Error,
            _ => DdaDeviceStatus::Unknown,
        }
    }
}

impl fmt::Display for DdaDeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DdaDeviceStatus::Unknown => write!(f, "Unknown"),
            DdaDeviceStatus::Available => write!(f, "Available"),
            DdaDeviceStatus::Assigned => write!(f, "Assigned"),
            DdaDeviceStatus::NotCompatible => write!(f, "Not Compatible"),
            DdaDeviceStatus::Error => write!(f, "Error"),
        }
    }
}

/// Information about a partitionable GPU on the host.
#[derive(Debug, Clone)]
pub struct PartitionableGpu {
    /// GPU hardware ID (PCI device path).
    pub id: String,
    /// Friendly name of the GPU.
    pub name: String,
    /// Driver version.
    pub driver_version: Option<String>,
    /// Total number of partitions available.
    pub total_partition_count: u32,
    /// Number of partitions currently in use.
    pub partitions_in_use: u32,
    /// Minimum partition count that can be set.
    pub min_partition_count: u32,
    /// Maximum partition count that can be set.
    pub max_partition_count: u32,
    /// Optimal partition count for best performance.
    pub optimal_partition_count: u32,
    /// Whether the GPU supports partitioning.
    pub is_partitionable: bool,
    /// Current operational status.
    pub status: GpuPartitionStatus,
    /// VRAM per partition in MB.
    pub vram_per_partition_mb: Option<u64>,
    /// Encode engines per partition.
    pub encode_per_partition: Option<u32>,
    /// Decode engines per partition.
    pub decode_per_partition: Option<u32>,
    /// Compute engines per partition.
    pub compute_per_partition: Option<u32>,
}

impl PartitionableGpu {
    /// Get the number of available (unused) partitions.
    pub fn available_partitions(&self) -> u32 {
        self.total_partition_count.saturating_sub(self.partitions_in_use)
    }

    /// Check if the GPU has available partitions.
    pub fn has_available_partitions(&self) -> bool {
        self.available_partitions() > 0
    }

    /// Check if the GPU is healthy and operational.
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, GpuPartitionStatus::Ok | GpuPartitionStatus::InUse)
    }
}

impl fmt::Display for PartitionableGpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}/{} partitions available)",
            self.name,
            self.available_partitions(),
            self.total_partition_count
        )
    }
}

/// GPU partition assigned to a VM.
#[derive(Debug, Clone)]
pub struct GpuPartition {
    /// Partition instance ID.
    pub instance_id: String,
    /// Parent GPU hardware ID.
    pub gpu_id: String,
    /// Parent GPU name.
    pub gpu_name: String,
    /// Partition index on the GPU.
    pub partition_index: u32,
    /// VM ID this partition is assigned to.
    pub vm_id: String,
    /// VRAM allocated to this partition in MB.
    pub vram_mb: Option<u64>,
    /// Encode engines allocated.
    pub encode_engines: Option<u32>,
    /// Decode engines allocated.
    pub decode_engines: Option<u32>,
    /// Compute engines allocated.
    pub compute_engines: Option<u32>,
}

impl fmt::Display for GpuPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GPU-P {} partition {} ({})",
            self.gpu_name, self.partition_index, self.instance_id
        )
    }
}

/// Settings for GPU partition assignment.
#[derive(Debug, Clone)]
pub struct GpuPartitionSettings {
    /// GPU hardware ID to assign partition from.
    pub gpu_id: String,
    /// Minimum VRAM in MB (0 for no minimum).
    pub min_vram_mb: u64,
    /// Maximum VRAM in MB (0 for no limit).
    pub max_vram_mb: u64,
    /// Optimal VRAM in MB.
    pub optimal_vram_mb: u64,
    /// Minimum encode engines.
    pub min_encode: u32,
    /// Maximum encode engines.
    pub max_encode: u32,
    /// Optimal encode engines.
    pub optimal_encode: u32,
    /// Minimum decode engines.
    pub min_decode: u32,
    /// Maximum decode engines.
    pub max_decode: u32,
    /// Optimal decode engines.
    pub optimal_decode: u32,
    /// Minimum compute engines.
    pub min_compute: u32,
    /// Maximum compute engines.
    pub max_compute: u32,
    /// Optimal compute engines.
    pub optimal_compute: u32,
}

impl Default for GpuPartitionSettings {
    fn default() -> Self {
        Self {
            gpu_id: String::new(),
            min_vram_mb: 0,
            max_vram_mb: 0,
            optimal_vram_mb: 0,
            min_encode: 0,
            max_encode: 0,
            optimal_encode: 0,
            min_decode: 0,
            max_decode: 0,
            optimal_decode: 0,
            min_compute: 0,
            max_compute: 0,
            optimal_compute: 0,
        }
    }
}

impl GpuPartitionSettings {
    /// Create settings for a specific GPU with default resource allocation.
    pub fn for_gpu(gpu_id: impl Into<String>) -> Self {
        Self {
            gpu_id: gpu_id.into(),
            ..Default::default()
        }
    }

    /// Create a new builder.
    pub fn builder() -> GpuPartitionSettingsBuilder {
        GpuPartitionSettingsBuilder::new()
    }
}

/// Builder for GPU partition settings.
#[derive(Debug, Clone, Default)]
pub struct GpuPartitionSettingsBuilder {
    settings: GpuPartitionSettings,
}

impl GpuPartitionSettingsBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the GPU hardware ID.
    pub fn gpu_id(mut self, id: impl Into<String>) -> Self {
        self.settings.gpu_id = id.into();
        self
    }

    /// Set VRAM limits (min, max, optimal) in MB.
    pub fn vram(mut self, min_mb: u64, max_mb: u64, optimal_mb: u64) -> Self {
        self.settings.min_vram_mb = min_mb;
        self.settings.max_vram_mb = max_mb;
        self.settings.optimal_vram_mb = optimal_mb;
        self
    }

    /// Set VRAM to a specific value for min/max/optimal.
    pub fn vram_mb(mut self, mb: u64) -> Self {
        self.settings.min_vram_mb = mb;
        self.settings.max_vram_mb = mb;
        self.settings.optimal_vram_mb = mb;
        self
    }

    /// Set encode engine limits.
    pub fn encode(mut self, min: u32, max: u32, optimal: u32) -> Self {
        self.settings.min_encode = min;
        self.settings.max_encode = max;
        self.settings.optimal_encode = optimal;
        self
    }

    /// Set decode engine limits.
    pub fn decode(mut self, min: u32, max: u32, optimal: u32) -> Self {
        self.settings.min_decode = min;
        self.settings.max_decode = max;
        self.settings.optimal_decode = optimal;
        self
    }

    /// Set compute engine limits.
    pub fn compute(mut self, min: u32, max: u32, optimal: u32) -> Self {
        self.settings.min_compute = min;
        self.settings.max_compute = max;
        self.settings.optimal_compute = optimal;
        self
    }

    /// Build the settings.
    pub fn build(self) -> GpuPartitionSettings {
        self.settings
    }
}

/// Information about a DDA-capable device.
#[derive(Debug, Clone)]
pub struct DdaDevice {
    /// Device instance path (PCI location).
    pub location_path: String,
    /// Device instance ID.
    pub instance_id: String,
    /// Friendly name of the device.
    pub friendly_name: String,
    /// Device class (e.g., "Display", "3D Video Controller").
    pub device_class: String,
    /// Vendor name.
    pub vendor: Option<String>,
    /// Device current status.
    pub status: DdaDeviceStatus,
    /// VM name if assigned (None if available).
    pub assigned_vm: Option<String>,
    /// Virtual function if this is an SR-IOV VF.
    pub is_virtual_function: bool,
    /// MMIO space required (low).
    pub mmio_space_required_low: u64,
    /// MMIO space required (high, above 4GB).
    pub mmio_space_required_high: u64,
}

impl DdaDevice {
    /// Check if the device is available for assignment.
    pub fn is_available(&self) -> bool {
        self.status == DdaDeviceStatus::Available && self.assigned_vm.is_none()
    }

    /// Check if the device requires high MMIO space (above 4GB).
    pub fn requires_high_mmio(&self) -> bool {
        self.mmio_space_required_high > 0
    }

    /// Get total MMIO space required.
    pub fn total_mmio_space(&self) -> u64 {
        self.mmio_space_required_low + self.mmio_space_required_high
    }
}

impl fmt::Display for DdaDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.friendly_name, self.location_path)?;
        if let Some(ref vm) = self.assigned_vm {
            write!(f, " (assigned to {})", vm)?;
        }
        Ok(())
    }
}

/// Settings for DDA device assignment.
#[derive(Debug, Clone)]
pub struct DdaDeviceSettings {
    /// Device location path.
    pub location_path: String,
    /// Virtual function slot (for SR-IOV devices).
    pub virtual_function_slot: Option<u32>,
    /// Low MMIO space to allocate (0 for auto).
    pub mmio_space_low: u64,
    /// High MMIO space to allocate (0 for auto).
    pub mmio_space_high: u64,
}

impl DdaDeviceSettings {
    /// Create settings for a device at the given location path.
    pub fn new(location_path: impl Into<String>) -> Self {
        Self {
            location_path: location_path.into(),
            virtual_function_slot: None,
            mmio_space_low: 0,
            mmio_space_high: 0,
        }
    }

    /// Set MMIO space allocation.
    pub fn with_mmio(mut self, low: u64, high: u64) -> Self {
        self.mmio_space_low = low;
        self.mmio_space_high = high;
        self
    }

    /// Set virtual function slot for SR-IOV.
    pub fn with_vf_slot(mut self, slot: u32) -> Self {
        self.virtual_function_slot = Some(slot);
        self
    }
}

/// GPU assignment type for a VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuAssignmentType {
    /// No GPU assigned.
    None,
    /// GPU-P (GPU Partitioning) - shared GPU.
    Partition,
    /// DDA (Discrete Device Assignment) - dedicated GPU.
    Dda,
    /// RemoteFX (legacy, deprecated).
    RemoteFx,
}

impl fmt::Display for GpuAssignmentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpuAssignmentType::None => write!(f, "None"),
            GpuAssignmentType::Partition => write!(f, "GPU-P"),
            GpuAssignmentType::Dda => write!(f, "DDA"),
            GpuAssignmentType::RemoteFx => write!(f, "RemoteFX"),
        }
    }
}

/// Summary of GPU assignments for a VM.
#[derive(Debug, Clone, Default)]
pub struct VmGpuSummary {
    /// Number of GPU partitions assigned.
    pub partition_count: u32,
    /// Number of DDA devices assigned.
    pub dda_count: u32,
    /// GPU partition details.
    pub partitions: Vec<GpuPartition>,
    /// DDA device details.
    pub dda_devices: Vec<DdaDevice>,
}

impl VmGpuSummary {
    /// Check if any GPU is assigned.
    pub fn has_gpu(&self) -> bool {
        self.partition_count > 0 || self.dda_count > 0
    }

    /// Get the primary assignment type.
    pub fn assignment_type(&self) -> GpuAssignmentType {
        if self.dda_count > 0 {
            GpuAssignmentType::Dda
        } else if self.partition_count > 0 {
            GpuAssignmentType::Partition
        } else {
            GpuAssignmentType::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_partition_status() {
        assert_eq!(GpuPartitionStatus::from(1), GpuPartitionStatus::Ok);
        assert_eq!(GpuPartitionStatus::from(4), GpuPartitionStatus::InUse);
        assert_eq!(GpuPartitionStatus::from(99), GpuPartitionStatus::Unknown);
    }

    #[test]
    fn test_dda_device_status() {
        assert_eq!(DdaDeviceStatus::from(1), DdaDeviceStatus::Available);
        assert_eq!(DdaDeviceStatus::from(2), DdaDeviceStatus::Assigned);
    }

    #[test]
    fn test_partitionable_gpu() {
        let gpu = PartitionableGpu {
            id: "PCI\\VEN_10DE".to_string(),
            name: "NVIDIA GeForce RTX 3080".to_string(),
            driver_version: Some("31.0.15.1234".to_string()),
            total_partition_count: 8,
            partitions_in_use: 3,
            min_partition_count: 1,
            max_partition_count: 16,
            optimal_partition_count: 8,
            is_partitionable: true,
            status: GpuPartitionStatus::Ok,
            vram_per_partition_mb: Some(1024),
            encode_per_partition: Some(1),
            decode_per_partition: Some(1),
            compute_per_partition: Some(1),
        };

        assert_eq!(gpu.available_partitions(), 5);
        assert!(gpu.has_available_partitions());
        assert!(gpu.is_healthy());
    }

    #[test]
    fn test_gpu_partition_settings_builder() {
        let settings = GpuPartitionSettings::builder()
            .gpu_id("GPU-123")
            .vram(512, 2048, 1024)
            .encode(0, 2, 1)
            .decode(0, 2, 1)
            .compute(0, 4, 2)
            .build();

        assert_eq!(settings.gpu_id, "GPU-123");
        assert_eq!(settings.min_vram_mb, 512);
        assert_eq!(settings.max_vram_mb, 2048);
        assert_eq!(settings.optimal_vram_mb, 1024);
    }

    #[test]
    fn test_dda_device() {
        let device = DdaDevice {
            location_path: "PCIROOT(0)#PCI(0100)".to_string(),
            instance_id: "PCI\\VEN_10DE".to_string(),
            friendly_name: "NVIDIA GeForce RTX 3080".to_string(),
            device_class: "Display".to_string(),
            vendor: Some("NVIDIA".to_string()),
            status: DdaDeviceStatus::Available,
            assigned_vm: None,
            is_virtual_function: false,
            mmio_space_required_low: 0x1000_0000,
            mmio_space_required_high: 0x2_0000_0000,
        };

        assert!(device.is_available());
        assert!(device.requires_high_mmio());
        assert_eq!(device.total_mmio_space(), 0x2_1000_0000);
    }

    #[test]
    fn test_vm_gpu_summary() {
        let mut summary = VmGpuSummary::default();
        assert!(!summary.has_gpu());
        assert_eq!(summary.assignment_type(), GpuAssignmentType::None);

        summary.partition_count = 1;
        assert!(summary.has_gpu());
        assert_eq!(summary.assignment_type(), GpuAssignmentType::Partition);

        summary.dda_count = 1;
        assert_eq!(summary.assignment_type(), GpuAssignmentType::Dda);
    }
}
