//! REST API for Windows Failover Cluster and Hyper-V management
//!
//! Provides exhaustive REST endpoints for:
//! - Failover Cluster: nodes, groups, resources, CSV
//! - Hyper-V: VMs, VHDs, snapshots, switches, GPU (GPU-P and DDA)

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// =============================================================================
// API Response Types
// =============================================================================

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

    pub fn ok() -> ApiResponse<&'static str> {
        ApiResponse::success("ok")
    }
}

type ApiResult<T> = Result<Json<ApiResponse<T>>, (StatusCode, Json<ApiResponse<()>>)>;

fn api_error(status: StatusCode, message: &str) -> (StatusCode, Json<ApiResponse<()>>) {
    (status, Json(ApiResponse::error(message)))
}

// =============================================================================
// Shared State
// =============================================================================

#[derive(Default)]
pub struct AppState;

pub type SharedState = Arc<AppState>;

// =============================================================================
// Main Router
// =============================================================================

pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/api/v1", api_routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

fn api_routes() -> Router<SharedState> {
    Router::new()
        .nest("/cluster", cluster_routes())
        .nest("/hyperv", hyperv_routes())
}

// =============================================================================
// Root Endpoints
// =============================================================================

async fn root() -> &'static str {
    "Windows Infrastructure Management API - Use /api/v1/cluster or /api/v1/hyperv"
}

async fn health() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("ok"))
}

// =============================================================================
// Cluster Routes
// =============================================================================

fn cluster_routes() -> Router<SharedState> {
    Router::new()
        // Cluster info
        .route("/", get(cluster_info))
        .route("/connect/:name", get(cluster_connect))
        // Nodes
        .route("/nodes", get(cluster_list_nodes))
        .route("/nodes/:name", get(cluster_get_node))
        .route("/nodes/:name/pause", axum::routing::post(cluster_pause_node))
        .route("/nodes/:name/resume", axum::routing::post(cluster_resume_node))
        // Groups
        .route("/groups", get(cluster_list_groups))
        .route("/groups/:name", get(cluster_get_group))
        .route("/groups/:name/online", axum::routing::post(cluster_group_online))
        .route("/groups/:name/offline", axum::routing::post(cluster_group_offline))
        .route("/groups/:name/move/:target_node", axum::routing::post(cluster_move_group))
        // Resources
        .route("/resources", get(cluster_list_resources))
        .route("/resources/:name", get(cluster_get_resource))
        .route("/resources/:name/online", axum::routing::post(cluster_resource_online))
        .route("/resources/:name/offline", axum::routing::post(cluster_resource_offline))
        // CSV
        .route("/csv", get(cluster_list_csv))
        .route("/csv/check-path", get(cluster_csv_check_path))
        .route("/csv/:name/maintenance", axum::routing::post(cluster_csv_maintenance))
}

// =============================================================================
// Hyper-V Routes
// =============================================================================

fn hyperv_routes() -> Router<SharedState> {
    Router::new()
        // Host info
        .route("/host", get(hyperv_host_info))
        .route("/adapters", get(hyperv_list_adapters))
        // VMs
        .route("/vms", get(hyperv_list_vms).post(hyperv_create_vm))
        .route("/vms/:name", get(hyperv_get_vm).delete(hyperv_delete_vm))
        .route("/vms/:name/start", axum::routing::post(hyperv_start_vm))
        .route("/vms/:name/stop", axum::routing::post(hyperv_stop_vm))
        .route("/vms/:name/force-stop", axum::routing::post(hyperv_force_stop_vm))
        .route("/vms/:name/pause", axum::routing::post(hyperv_pause_vm))
        .route("/vms/:name/resume", axum::routing::post(hyperv_resume_vm))
        .route("/vms/:name/save", axum::routing::post(hyperv_save_vm))
        .route("/vms/:name/reset", axum::routing::post(hyperv_reset_vm))
        .route("/vms/:name/export", axum::routing::post(hyperv_export_vm))
        // VM Disks
        .route("/vms/:name/disks", get(hyperv_vm_disks))
        .route("/vms/:name/disks/attach", axum::routing::post(hyperv_attach_disk))
        .route("/vms/:name/disks/detach", axum::routing::post(hyperv_detach_disk))
        // VM DVD/ISO
        .route("/vms/:name/dvd", get(hyperv_vm_dvd))
        .route("/vms/:name/dvd/mount", axum::routing::post(hyperv_mount_iso))
        .route("/vms/:name/dvd/eject", axum::routing::post(hyperv_eject_iso))
        .route("/vms/:name/boot-order", axum::routing::post(hyperv_set_boot_order))
        // VM Snapshots
        .route("/vms/:name/snapshots", get(hyperv_list_snapshots).post(hyperv_create_snapshot))
        .route("/vms/:name/snapshots/:snapshot", get(hyperv_get_snapshot))
        .route("/vms/:name/snapshots/:snapshot/apply", axum::routing::post(hyperv_apply_snapshot))
        .route("/vms/:name/snapshots/:snapshot/delete", axum::routing::delete(hyperv_delete_snapshot))
        // VM GPU
        .route("/vms/:name/gpu", get(hyperv_vm_gpu_adapters))
        .route("/vms/:name/gpu/add", axum::routing::post(hyperv_add_gpu))
        .route("/vms/:name/gpu/remove", axum::routing::post(hyperv_remove_gpu))
        .route("/vms/:name/gpu/configure", axum::routing::post(hyperv_configure_gpu))
        // VM DDA
        .route("/vms/:name/dda", get(hyperv_vm_dda_devices))
        .route("/vms/:name/dda/assign", axum::routing::post(hyperv_assign_device))
        .route("/vms/:name/dda/remove", axum::routing::post(hyperv_remove_device))
        // Switches
        .route("/switches", get(hyperv_list_switches).post(hyperv_create_switch))
        .route("/switches/:name", get(hyperv_get_switch).delete(hyperv_delete_switch))
        // VHDs
        .route("/vhds", axum::routing::post(hyperv_create_vhd))
        .route("/vhds/info", get(hyperv_get_vhd_info))
        .route("/vhds/resize", axum::routing::post(hyperv_resize_vhd))
        .route("/vhds/compact", axum::routing::post(hyperv_compact_vhd))
        .route("/vhds/mount", axum::routing::post(hyperv_mount_vhd))
        .route("/vhds/dismount", axum::routing::post(hyperv_dismount_vhd))
        .route("/vhds/differencing", axum::routing::post(hyperv_create_diff_vhd))
        .route("/vhds/initialize", axum::routing::post(hyperv_initialize_vhd))
        // Windows Image
        .route("/iso/editions", get(hyperv_iso_editions))
        .route("/iso/create-vhdx", axum::routing::post(hyperv_create_vhdx_from_iso))
        // GPUs
        .route("/gpus", get(hyperv_list_gpus))
        .route("/gpus/partitionable", get(hyperv_list_partitionable_gpus))
        // DDA
        .route("/dda/support", get(hyperv_dda_support))
        .route("/dda/devices", get(hyperv_dda_devices))
        .route("/dda/device-path", get(hyperv_device_path))
        .route("/dda/dismount", axum::routing::post(hyperv_dismount_device))
        .route("/dda/mount", axum::routing::post(hyperv_mount_device))
}

// =============================================================================
// DTO Types
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
// Cluster Handlers (Windows only)
// =============================================================================

#[cfg(windows)]
mod cluster_handlers {
    use super::*;
    use clus::{Cluster, Csv, GroupState, NodeState, ResourceState};

    pub async fn cluster_info(Query(params): Query<ClusterNameQuery>) -> ApiResult<String> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let name = cluster
            .name()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(name)))
    }

    pub async fn cluster_connect(Path(name): Path<String>) -> ApiResult<String> {
        let cluster = Cluster::open(Some(&name))
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let cluster_name = cluster
            .name()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(cluster_name)))
    }

    pub async fn cluster_list_nodes(Query(params): Query<ClusterNameQuery>) -> ApiResult<Vec<NodeDto>> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let nodes = cluster
            .nodes()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        let dtos: Vec<NodeDto> = nodes
            .iter()
            .map(|n| {
                let state = n.state().map(|s| format!("{:?}", s)).unwrap_or_default();
                NodeDto {
                    name: n.name().to_string(),
                    state,
                }
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn cluster_get_node(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<NodeDto> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let node = cluster
            .open_node(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        let state = node.state().map(|s| format!("{:?}", s)).unwrap_or_default();
        Ok(Json(ApiResponse::success(NodeDto {
            name: node.name().to_string(),
            state,
        })))
    }

    pub async fn cluster_pause_node(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let node = cluster
            .open_node(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        node.pause()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn cluster_resume_node(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let node = cluster
            .open_node(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        node.resume()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn cluster_list_groups(Query(params): Query<ClusterNameQuery>) -> ApiResult<Vec<GroupDto>> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let groups = cluster
            .groups()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        let dtos: Vec<GroupDto> = groups
            .iter()
            .map(|g| {
                let (state, owner) = g.state().ok().unwrap_or((GroupState::Unknown(0), None));
                GroupDto {
                    name: g.name().to_string(),
                    state: format!("{:?}", state),
                    owner_node: owner,
                }
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn cluster_get_group(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<GroupDto> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let group = cluster
            .open_group(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        let (state, owner) = group.state().ok().unwrap_or((GroupState::Unknown(0), None));
        Ok(Json(ApiResponse::success(GroupDto {
            name: group.name().to_string(),
            state: format!("{:?}", state),
            owner_node: owner,
        })))
    }

    pub async fn cluster_group_online(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let group = cluster
            .open_group(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        group
            .online()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn cluster_group_offline(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let group = cluster
            .open_group(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        group
            .offline()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn cluster_move_group(
        Path((name, target_node)): Path<(String, String)>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let group = cluster
            .open_group(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        let node = cluster
            .open_node(&target_node)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        group
            .move_to(&node)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn cluster_list_resources(Query(params): Query<ClusterNameQuery>) -> ApiResult<Vec<ResourceDto>> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let resources = cluster
            .resources()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        let dtos: Vec<ResourceDto> = resources
            .iter()
            .map(|r| {
                let (state, owner) = r.state().ok().unwrap_or((ResourceState::Unknown(0), None));
                ResourceDto {
                    name: r.name().to_string(),
                    state: format!("{:?}", state),
                    owner_node: owner,
                }
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn cluster_get_resource(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<ResourceDto> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let resource = cluster
            .open_resource(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        let (state, owner) = resource
            .state()
            .ok()
            .unwrap_or((ResourceState::Unknown(0), None));
        Ok(Json(ApiResponse::success(ResourceDto {
            name: resource.name().to_string(),
            state: format!("{:?}", state),
            owner_node: owner,
        })))
    }

    pub async fn cluster_resource_online(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let resource = cluster
            .open_resource(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        resource
            .online()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn cluster_resource_offline(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let resource = cluster
            .open_resource(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        resource
            .offline()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn cluster_list_csv(Query(params): Query<ClusterNameQuery>) -> ApiResult<Vec<CsvDto>> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let resources = cluster
            .resources()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        let mut dtos = Vec::new();
        for r in &resources {
            let is_csv = Csv::is_csv_resource(r).unwrap_or(false);
            if is_csv {
                let (state, owner) = r.state().ok().unwrap_or((ResourceState::Unknown(0), None));
                dtos.push(CsvDto {
                    name: r.name().to_string(),
                    state: format!("{:?}", state),
                    owner_node: owner,
                    is_csv: true,
                });
            }
        }
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn cluster_csv_check_path(Query(params): Query<CsvPathQuery>) -> ApiResult<bool> {
        let is_csv = Csv::is_path_on_csv(&params.path);
        Ok(Json(ApiResponse::success(is_csv)))
    }

    pub async fn cluster_csv_maintenance(
        Path(name): Path<String>,
        Query(params): Query<ClusterNameQuery>,
        Json(req): Json<MaintenanceModeRequest>,
    ) -> ApiResult<&'static str> {
        let cluster = Cluster::open(params.name.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let resource = cluster
            .open_resource(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        Csv::set_maintenance_mode(&resource, req.enable)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }
}

// =============================================================================
// Hyper-V Handlers (Windows only)
// =============================================================================

#[cfg(windows)]
mod hyperv_handlers {
    use super::*;
    use hv::{HyperV, SnapshotType, SwitchType, VhdType, VmGeneration};

    pub async fn hyperv_host_info() -> ApiResult<HostInfoDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let info = hv
            .host_info()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(HostInfoDto {
            computer_name: info.computer_name,
            logical_processor_count: info.logical_processor_count,
            memory_capacity_bytes: info.memory_capacity_bytes,
            vm_path: info.vm_path,
            vhd_path: info.vhd_path,
        })))
    }

    pub async fn hyperv_list_adapters() -> ApiResult<Vec<NetworkAdapterDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let adapters = hv
            .list_network_adapters()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = adapters
            .into_iter()
            .map(|a| NetworkAdapterDto {
                name: a.name,
                description: a.description,
                mac_address: a.mac_address,
                link_speed: a.link_speed,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_list_vms() -> ApiResult<Vec<VmDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vms = hv
            .list_vms()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos: Vec<VmDto> = vms
            .iter()
            .map(|vm| VmDto {
                id: vm.id().to_string(),
                name: vm.name().to_string(),
                state: vm.state().map(|s| format!("{:?}", s)).unwrap_or_default(),
                cpu_count: vm.cpu_count().ok(),
                memory_mb: vm.memory_mb().ok(),
                uptime_seconds: vm.uptime_seconds().ok(),
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_get_vm(Path(name): Path<String>) -> ApiResult<VmDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        Ok(Json(ApiResponse::success(VmDto {
            id: vm.id().to_string(),
            name: vm.name().to_string(),
            state: vm.state().map(|s| format!("{:?}", s)).unwrap_or_default(),
            cpu_count: vm.cpu_count().ok(),
            memory_mb: vm.memory_mb().ok(),
            uptime_seconds: vm.uptime_seconds().ok(),
        })))
    }

    pub async fn hyperv_create_vm(Json(req): Json<CreateVmRequest>) -> ApiResult<VmDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let generation = match req.generation.unwrap_or(2) {
            1 => VmGeneration::Gen1,
            _ => VmGeneration::Gen2,
        };
        let vm = hv
            .create_vm(
                &req.name,
                req.memory_mb,
                req.cpu_count.unwrap_or(2),
                generation,
                req.vhd_path.as_deref(),
            )
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(VmDto {
            id: vm.id().to_string(),
            name: vm.name().to_string(),
            state: vm.state().map(|s| format!("{:?}", s)).unwrap_or_default(),
            cpu_count: vm.cpu_count().ok(),
            memory_mb: vm.memory_mb().ok(),
            uptime_seconds: None,
        })))
    }

    pub async fn hyperv_delete_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.delete_vm(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_start_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let mut vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vm.start()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_stop_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let mut vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vm.stop()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_force_stop_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let mut vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vm.force_stop()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_pause_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let mut vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vm.pause()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_resume_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let mut vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vm.resume()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_save_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let mut vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vm.save()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_reset_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let mut vm = hv
            .get_vm(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vm.reset()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_export_vm(
        Path(name): Path<String>,
        Json(req): Json<ExportVmRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.export_vm(&name, &req.path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_vm_disks(Path(name): Path<String>) -> ApiResult<Vec<DiskDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let disks = hv
            .get_hard_disk_drives(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = disks
            .into_iter()
            .map(|d| DiskDto {
                controller_type: d.controller_type,
                controller_number: d.controller_number,
                controller_location: d.controller_location,
                path: d.path,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_attach_disk(
        Path(name): Path<String>,
        Json(req): Json<AttachDiskRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.add_hard_disk_drive(&name, &req.vhd_path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_detach_disk(
        Path(name): Path<String>,
        Json(req): Json<DetachDiskRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.remove_hard_disk_drive(&name, req.controller_number, req.controller_location)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_vm_dvd(Path(name): Path<String>) -> ApiResult<Vec<DiskDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dvds = hv
            .get_dvd_drives(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = dvds
            .into_iter()
            .map(|d| DiskDto {
                controller_type: d.controller_type,
                controller_number: d.controller_number,
                controller_location: d.controller_location,
                path: d.path,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_mount_iso(
        Path(name): Path<String>,
        Json(req): Json<MountIsoRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.mount_iso(&name, &req.iso_path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_eject_iso(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.eject_iso(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_set_boot_order(
        Path(name): Path<String>,
        Json(req): Json<BootOrderRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let devices: Vec<&str> = req.devices.iter().map(|s| s.as_str()).collect();
        hv.set_boot_order(&name, &devices)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_list_snapshots(Path(name): Path<String>) -> ApiResult<Vec<SnapshotDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let snapshots = hv
            .list_snapshots(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = snapshots
            .into_iter()
            .map(|s| SnapshotDto {
                name: s.name().to_string(),
                id: s.id().to_string(),
                vm_name: s.vm_name().to_string(),
                creation_time: s.creation_time().map(|t| t.to_string()),
                parent_name: s.parent_name().map(|n| n.to_string()),
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_get_snapshot(
        Path((name, snapshot)): Path<(String, String)>,
    ) -> ApiResult<SnapshotDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let s = hv
            .get_snapshot(&name, &snapshot)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        Ok(Json(ApiResponse::success(SnapshotDto {
            name: s.name().to_string(),
            id: s.id().to_string(),
            vm_name: s.vm_name().to_string(),
            creation_time: s.creation_time().map(|t| t.to_string()),
            parent_name: s.parent_name().map(|n| n.to_string()),
        })))
    }

    pub async fn hyperv_create_snapshot(
        Path(name): Path<String>,
        Json(req): Json<CreateSnapshotRequest>,
    ) -> ApiResult<SnapshotDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let snapshot_type = match req.snapshot_type.as_deref() {
            Some("Production") => SnapshotType::Production,
            Some("ProductionOnly") => SnapshotType::ProductionOnly,
            _ => SnapshotType::Standard,
        };
        let s = hv
            .create_snapshot(&name, &req.name, snapshot_type)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(SnapshotDto {
            name: s.name().to_string(),
            id: s.id().to_string(),
            vm_name: s.vm_name().to_string(),
            creation_time: s.creation_time().map(|t| t.to_string()),
            parent_name: s.parent_name().map(|n| n.to_string()),
        })))
    }

    pub async fn hyperv_apply_snapshot(
        Path((name, snapshot)): Path<(String, String)>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let s = hv
            .get_snapshot(&name, &snapshot)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        s.apply()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_delete_snapshot(
        Path((name, snapshot)): Path<(String, String)>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let s = hv
            .get_snapshot(&name, &snapshot)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        s.delete()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_list_switches() -> ApiResult<Vec<SwitchDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let switches = hv
            .list_switches()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = switches
            .into_iter()
            .map(|s| SwitchDto {
                name: s.name().to_string(),
                id: s.id().to_string(),
                switch_type: format!("{:?}", s.switch_type()),
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_get_switch(Path(name): Path<String>) -> ApiResult<SwitchDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let s = hv
            .get_switch(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        Ok(Json(ApiResponse::success(SwitchDto {
            name: s.name().to_string(),
            id: s.id().to_string(),
            switch_type: format!("{:?}", s.switch_type()),
        })))
    }

    pub async fn hyperv_create_switch(Json(req): Json<CreateSwitchRequest>) -> ApiResult<SwitchDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

        let s = match req.switch_type.to_lowercase().as_str() {
            "external" => {
                let adapter = req
                    .network_adapter
                    .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "network_adapter required for external switch"))?;
                hv.create_external_switch(&req.name, &adapter, req.allow_management_os.unwrap_or(true))
                    .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?
            }
            "internal" => hv
                .create_switch(&req.name, SwitchType::Internal)
                .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?,
            "private" => hv
                .create_switch(&req.name, SwitchType::Private)
                .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?,
            _ => return Err(api_error(StatusCode::BAD_REQUEST, "Invalid switch_type")),
        };

        Ok(Json(ApiResponse::success(SwitchDto {
            name: s.name().to_string(),
            id: s.id().to_string(),
            switch_type: format!("{:?}", s.switch_type()),
        })))
    }

    pub async fn hyperv_delete_switch(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let s = hv
            .get_switch(&name)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        s.delete()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_get_vhd_info(Query(req): Query<VhdPathRequest>) -> ApiResult<VhdDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vhd = hv
            .get_vhd(&req.path)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        Ok(Json(ApiResponse::success(VhdDto {
            path: vhd.path().to_string(),
            format: format!("{:?}", vhd.format()),
            vhd_type: format!("{:?}", vhd.vhd_type()),
            max_size_bytes: vhd.max_size_bytes().unwrap_or(0),
            file_size_bytes: vhd.file_size_bytes().unwrap_or(0),
            parent_path: vhd.parent_path(),
            is_attached: vhd.is_attached().unwrap_or(false),
        })))
    }

    pub async fn hyperv_create_vhd(Json(req): Json<CreateVhdRequest>) -> ApiResult<VhdDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vhd_type = match req.vhd_type.as_deref() {
            Some("Fixed") => VhdType::Fixed,
            Some("Differencing") => VhdType::Differencing,
            _ => VhdType::Dynamic,
        };
        let vhd = hv
            .create_vhd(&req.path, req.size_bytes, vhd_type, req.block_size_bytes)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(VhdDto {
            path: vhd.path().to_string(),
            format: format!("{:?}", vhd.format()),
            vhd_type: format!("{:?}", vhd.vhd_type()),
            max_size_bytes: vhd.max_size_bytes().unwrap_or(0),
            file_size_bytes: vhd.file_size_bytes().unwrap_or(0),
            parent_path: vhd.parent_path(),
            is_attached: vhd.is_attached().unwrap_or(false),
        })))
    }

    pub async fn hyperv_resize_vhd(Json(req): Json<ResizeVhdRequest>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vhd = hv
            .get_vhd(&req.path)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vhd.resize(req.size_bytes)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_compact_vhd(Json(req): Json<VhdPathRequest>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vhd = hv
            .get_vhd(&req.path)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vhd.compact()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_mount_vhd(Json(req): Json<VhdPathRequest>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vhd = hv
            .get_vhd(&req.path)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vhd.mount(false)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_dismount_vhd(Json(req): Json<VhdPathRequest>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vhd = hv
            .get_vhd(&req.path)
            .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
        vhd.dismount()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_create_diff_vhd(Json(req): Json<DiffVhdRequest>) -> ApiResult<VhdDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let vhd = hv
            .create_differencing_vhd(&req.path, &req.parent_path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(VhdDto {
            path: vhd.path().to_string(),
            format: format!("{:?}", vhd.format()),
            vhd_type: format!("{:?}", vhd.vhd_type()),
            max_size_bytes: vhd.max_size_bytes().unwrap_or(0),
            file_size_bytes: vhd.file_size_bytes().unwrap_or(0),
            parent_path: vhd.parent_path(),
            is_attached: vhd.is_attached().unwrap_or(false),
        })))
    }

    pub async fn hyperv_initialize_vhd(Json(req): Json<InitVhdRequest>) -> ApiResult<String> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let partition_style = match req.partition_style.as_deref() {
            Some("Mbr") | Some("MBR") => hv::PartitionStyle::Mbr,
            _ => hv::PartitionStyle::Gpt,
        };
        let file_system = match req.file_system.as_deref() {
            Some("ReFS") | Some("refs") => hv::FileSystem::ReFs,
            Some("FAT32") | Some("fat32") => hv::FileSystem::Fat32,
            Some("ExFAT") | Some("exfat") => hv::FileSystem::ExFat,
            _ => hv::FileSystem::Ntfs,
        };
        let drive_letter = hv
            .initialize_vhd(&req.path, partition_style, file_system, req.label.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(drive_letter)))
    }

    pub async fn hyperv_iso_editions(Query(req): Query<IsoPathQuery>) -> ApiResult<Vec<WindowsEditionDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let editions = hv
            .get_windows_editions(&req.path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = editions
            .into_iter()
            .map(|e| WindowsEditionDto {
                index: e.index,
                name: e.name,
                description: e.description,
                size_bytes: e.size_bytes,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_create_vhdx_from_iso(
        Json(req): Json<CreateVhdxFromIsoRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.create_vhdx_from_iso(&req.iso_path, &req.vhdx_path, req.size_gb, req.edition_index)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_list_gpus() -> ApiResult<Vec<GpuDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let gpus = hv
            .list_gpus()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = gpus
            .into_iter()
            .map(|g| GpuDto {
                device_instance_id: g.device_instance_id,
                name: g.name,
                description: g.description,
                manufacturer: g.manufacturer,
                supports_partitioning: g.supports_partitioning,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_list_partitionable_gpus() -> ApiResult<Vec<GpuDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let gpus = hv
            .list_partitionable_gpus()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = gpus
            .into_iter()
            .map(|g| GpuDto {
                device_instance_id: g.device_instance_id,
                name: g.name,
                description: g.description,
                manufacturer: g.manufacturer,
                supports_partitioning: g.supports_partitioning,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_vm_gpu_adapters(Path(name): Path<String>) -> ApiResult<Vec<GpuAdapterDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let adapters = hv
            .get_vm_gpu_adapters(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = adapters
            .into_iter()
            .map(|a| GpuAdapterDto {
                vm_name: a.vm_name,
                instance_path: a.instance_path,
                min_partition_vram: a.min_partition_vram,
                max_partition_vram: a.max_partition_vram,
                optimal_partition_vram: a.optimal_partition_vram,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_add_gpu(
        Path(name): Path<String>,
        Json(req): Json<AddGpuRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.add_gpu_to_vm(&name, req.instance_path.as_deref())
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_remove_gpu(Path(name): Path<String>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.remove_gpu_from_vm(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_configure_gpu(
        Path(name): Path<String>,
        Json(req): Json<ConfigureGpuRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.configure_vm_for_gpu(&name, req.low_mmio_gb, req.high_mmio_gb)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_dda_support() -> ApiResult<DdaSupportDto> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let info = hv
            .check_dda_support()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(DdaSupportDto {
            is_supported: info.is_supported,
            is_server: info.is_server,
            has_iommu: info.has_iommu,
            cmdlet_available: info.cmdlet_available,
            reason: info.reason,
        })))
    }

    pub async fn hyperv_dda_devices() -> ApiResult<Vec<AssignableDeviceDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let devices = hv
            .get_assignable_devices()
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = devices
            .into_iter()
            .map(|d| AssignableDeviceDto {
                instance_id: d.instance_id,
                name: d.name,
                location_path: d.location_path,
                is_assigned: d.is_assigned,
                assigned_vm: d.assigned_vm,
                is_dismounted: d.is_dismounted,
                status: d.status,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_device_path(Query(req): Query<DevicePathRequest>) -> ApiResult<String> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let path = hv
            .get_device_location_path(&req.instance_id)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success(path)))
    }

    pub async fn hyperv_dismount_device(Json(req): Json<DeviceLocationRequest>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.dismount_device(&req.location_path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_mount_device(Json(req): Json<DeviceLocationRequest>) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.mount_device(&req.location_path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_vm_dda_devices(Path(name): Path<String>) -> ApiResult<Vec<AssignableDeviceDto>> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let devices = hv
            .get_vm_assigned_devices(&name)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        let dtos = devices
            .into_iter()
            .map(|d| AssignableDeviceDto {
                instance_id: d.instance_id,
                name: d.name,
                location_path: d.location_path,
                is_assigned: d.is_assigned,
                assigned_vm: d.assigned_vm,
                is_dismounted: d.is_dismounted,
                status: d.status,
            })
            .collect();
        Ok(Json(ApiResponse::success(dtos)))
    }

    pub async fn hyperv_assign_device(
        Path(name): Path<String>,
        Json(req): Json<DeviceLocationRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.assign_device_to_vm(&name, &req.location_path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }

    pub async fn hyperv_remove_device(
        Path(name): Path<String>,
        Json(req): Json<DeviceLocationRequest>,
    ) -> ApiResult<&'static str> {
        let hv = HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        hv.remove_assigned_device(&name, &req.location_path)
            .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
        Ok(Json(ApiResponse::success("ok")))
    }
}

// =============================================================================
// Non-Windows Stubs
// =============================================================================

#[cfg(not(windows))]
mod cluster_handlers {
    use super::*;

    fn not_supported() -> (StatusCode, Json<ApiResponse<()>>) {
        api_error(StatusCode::NOT_IMPLEMENTED, "Cluster API only available on Windows")
    }

    pub async fn cluster_info(_: Query<ClusterNameQuery>) -> ApiResult<String> { Err(not_supported()) }
    pub async fn cluster_connect(_: Path<String>) -> ApiResult<String> { Err(not_supported()) }
    pub async fn cluster_list_nodes(_: Query<ClusterNameQuery>) -> ApiResult<Vec<NodeDto>> { Err(not_supported()) }
    pub async fn cluster_get_node(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<NodeDto> { Err(not_supported()) }
    pub async fn cluster_pause_node(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn cluster_resume_node(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn cluster_list_groups(_: Query<ClusterNameQuery>) -> ApiResult<Vec<GroupDto>> { Err(not_supported()) }
    pub async fn cluster_get_group(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<GroupDto> { Err(not_supported()) }
    pub async fn cluster_group_online(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn cluster_group_offline(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn cluster_move_group(_: Path<(String, String)>, _: Query<ClusterNameQuery>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn cluster_list_resources(_: Query<ClusterNameQuery>) -> ApiResult<Vec<ResourceDto>> { Err(not_supported()) }
    pub async fn cluster_get_resource(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<ResourceDto> { Err(not_supported()) }
    pub async fn cluster_resource_online(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn cluster_resource_offline(_: Path<String>, _: Query<ClusterNameQuery>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn cluster_list_csv(_: Query<ClusterNameQuery>) -> ApiResult<Vec<CsvDto>> { Err(not_supported()) }
    pub async fn cluster_csv_check_path(_: Query<CsvPathQuery>) -> ApiResult<bool> { Err(not_supported()) }
    pub async fn cluster_csv_maintenance(_: Path<String>, _: Query<ClusterNameQuery>, _: Json<MaintenanceModeRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
}

#[cfg(not(windows))]
mod hyperv_handlers {
    use super::*;

    fn not_supported() -> (StatusCode, Json<ApiResponse<()>>) {
        api_error(StatusCode::NOT_IMPLEMENTED, "Hyper-V API only available on Windows")
    }

    pub async fn hyperv_host_info() -> ApiResult<HostInfoDto> { Err(not_supported()) }
    pub async fn hyperv_list_adapters() -> ApiResult<Vec<NetworkAdapterDto>> { Err(not_supported()) }
    pub async fn hyperv_list_vms() -> ApiResult<Vec<VmDto>> { Err(not_supported()) }
    pub async fn hyperv_get_vm(_: Path<String>) -> ApiResult<VmDto> { Err(not_supported()) }
    pub async fn hyperv_create_vm(_: Json<CreateVmRequest>) -> ApiResult<VmDto> { Err(not_supported()) }
    pub async fn hyperv_delete_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_start_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_stop_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_force_stop_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_pause_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_resume_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_save_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_reset_vm(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_export_vm(_: Path<String>, _: Json<ExportVmRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_vm_disks(_: Path<String>) -> ApiResult<Vec<DiskDto>> { Err(not_supported()) }
    pub async fn hyperv_attach_disk(_: Path<String>, _: Json<AttachDiskRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_detach_disk(_: Path<String>, _: Json<DetachDiskRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_vm_dvd(_: Path<String>) -> ApiResult<Vec<DiskDto>> { Err(not_supported()) }
    pub async fn hyperv_mount_iso(_: Path<String>, _: Json<MountIsoRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_eject_iso(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_set_boot_order(_: Path<String>, _: Json<BootOrderRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_list_snapshots(_: Path<String>) -> ApiResult<Vec<SnapshotDto>> { Err(not_supported()) }
    pub async fn hyperv_get_snapshot(_: Path<(String, String)>) -> ApiResult<SnapshotDto> { Err(not_supported()) }
    pub async fn hyperv_create_snapshot(_: Path<String>, _: Json<CreateSnapshotRequest>) -> ApiResult<SnapshotDto> { Err(not_supported()) }
    pub async fn hyperv_apply_snapshot(_: Path<(String, String)>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_delete_snapshot(_: Path<(String, String)>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_list_switches() -> ApiResult<Vec<SwitchDto>> { Err(not_supported()) }
    pub async fn hyperv_get_switch(_: Path<String>) -> ApiResult<SwitchDto> { Err(not_supported()) }
    pub async fn hyperv_create_switch(_: Json<CreateSwitchRequest>) -> ApiResult<SwitchDto> { Err(not_supported()) }
    pub async fn hyperv_delete_switch(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_get_vhd_info(_: Query<VhdPathRequest>) -> ApiResult<VhdDto> { Err(not_supported()) }
    pub async fn hyperv_create_vhd(_: Json<CreateVhdRequest>) -> ApiResult<VhdDto> { Err(not_supported()) }
    pub async fn hyperv_resize_vhd(_: Json<ResizeVhdRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_compact_vhd(_: Json<VhdPathRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_mount_vhd(_: Json<VhdPathRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_dismount_vhd(_: Json<VhdPathRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_create_diff_vhd(_: Json<DiffVhdRequest>) -> ApiResult<VhdDto> { Err(not_supported()) }
    pub async fn hyperv_initialize_vhd(_: Json<InitVhdRequest>) -> ApiResult<String> { Err(not_supported()) }
    pub async fn hyperv_iso_editions(_: Query<IsoPathQuery>) -> ApiResult<Vec<WindowsEditionDto>> { Err(not_supported()) }
    pub async fn hyperv_create_vhdx_from_iso(_: Json<CreateVhdxFromIsoRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_list_gpus() -> ApiResult<Vec<GpuDto>> { Err(not_supported()) }
    pub async fn hyperv_list_partitionable_gpus() -> ApiResult<Vec<GpuDto>> { Err(not_supported()) }
    pub async fn hyperv_vm_gpu_adapters(_: Path<String>) -> ApiResult<Vec<GpuAdapterDto>> { Err(not_supported()) }
    pub async fn hyperv_add_gpu(_: Path<String>, _: Json<AddGpuRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_remove_gpu(_: Path<String>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_configure_gpu(_: Path<String>, _: Json<ConfigureGpuRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_dda_support() -> ApiResult<DdaSupportDto> { Err(not_supported()) }
    pub async fn hyperv_dda_devices() -> ApiResult<Vec<AssignableDeviceDto>> { Err(not_supported()) }
    pub async fn hyperv_device_path(_: Query<DevicePathRequest>) -> ApiResult<String> { Err(not_supported()) }
    pub async fn hyperv_dismount_device(_: Json<DeviceLocationRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_mount_device(_: Json<DeviceLocationRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_vm_dda_devices(_: Path<String>) -> ApiResult<Vec<AssignableDeviceDto>> { Err(not_supported()) }
    pub async fn hyperv_assign_device(_: Path<String>, _: Json<DeviceLocationRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
    pub async fn hyperv_remove_device(_: Path<String>, _: Json<DeviceLocationRequest>) -> ApiResult<&'static str> { Err(not_supported()) }
}

// Re-export handlers
use cluster_handlers::*;
use hyperv_handlers::*;

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState);
    let app = create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
