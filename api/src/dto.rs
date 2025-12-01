//! Data Transfer Objects for API requests and responses

use serde::{Deserialize, Serialize};

// =============================================================================
// Cluster DTOs
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeDto {
    pub name: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupDto {
    pub name: String,
    pub state: String,
    pub owner_node: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceDto {
    pub name: String,
    pub state: String,
    pub owner_node: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvDto {
    pub name: String,
    pub state: String,
    pub owner_node: Option<String>,
    pub is_csv: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvPathQuery {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenanceModeRequest {
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterNameQuery {
    pub name: Option<String>,
}

// =============================================================================
// Hyper-V DTOs
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct VmDto {
    pub id: String,
    pub name: String,
    pub state: String,
    pub cpu_count: Option<u32>,
    pub memory_mb: Option<u64>,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVmRequest {
    pub name: String,
    pub memory_mb: u64,
    pub cpu_count: Option<u32>,
    pub generation: Option<u32>,
    pub vhd_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchDto {
    pub name: String,
    pub id: String,
    pub switch_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSwitchRequest {
    pub name: String,
    pub switch_type: String,
    pub network_adapter: Option<String>,
    pub allow_management_os: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VhdDto {
    pub path: String,
    pub format: String,
    pub vhd_type: String,
    pub max_size_bytes: u64,
    pub file_size_bytes: u64,
    pub parent_path: Option<String>,
    pub is_attached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVhdRequest {
    pub path: String,
    pub size_bytes: u64,
    pub vhd_type: Option<String>,
    pub block_size_bytes: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VhdPathRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResizeVhdRequest {
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffVhdRequest {
    pub path: String,
    pub parent_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitVhdRequest {
    pub path: String,
    pub partition_style: Option<String>,
    pub file_system: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotDto {
    pub name: String,
    pub id: String,
    pub vm_name: String,
    pub creation_time: Option<String>,
    pub parent_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSnapshotRequest {
    pub name: String,
    pub snapshot_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuDto {
    pub device_instance_id: String,
    pub name: String,
    pub description: String,
    pub manufacturer: String,
    pub supports_partitioning: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuAdapterDto {
    pub vm_name: String,
    pub instance_path: Option<String>,
    pub min_partition_vram: u64,
    pub max_partition_vram: u64,
    pub optimal_partition_vram: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddGpuRequest {
    pub instance_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigureGpuRequest {
    pub low_mmio_gb: u32,
    pub high_mmio_gb: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DdaSupportDto {
    pub is_supported: bool,
    pub is_server: bool,
    pub has_iommu: bool,
    pub cmdlet_available: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssignableDeviceDto {
    pub instance_id: String,
    pub name: String,
    pub location_path: String,
    pub is_assigned: bool,
    pub assigned_vm: Option<String>,
    pub is_dismounted: bool,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevicePathRequest {
    pub instance_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceLocationRequest {
    pub location_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskDto {
    pub controller_type: String,
    pub controller_number: u32,
    pub controller_location: u32,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttachDiskRequest {
    pub vhd_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetachDiskRequest {
    pub controller_number: u32,
    pub controller_location: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MountIsoRequest {
    pub iso_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootOrderRequest {
    pub devices: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportVmRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostInfoDto {
    pub computer_name: String,
    pub logical_processor_count: u32,
    pub memory_capacity_bytes: u64,
    pub vm_path: String,
    pub vhd_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkAdapterDto {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub link_speed: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowsEditionDto {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IsoPathQuery {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVhdxFromIsoRequest {
    pub iso_path: String,
    pub vhdx_path: String,
    pub size_gb: u64,
    pub edition_index: u32,
}
