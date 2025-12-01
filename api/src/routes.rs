//! Route definitions

use axum::{routing::get, Router};

use crate::handlers::*;
use crate::SharedState;

pub fn cluster_routes() -> Router<SharedState> {
    Router::new()
        // Cluster info
        .route("/", get(cluster_info))
        .route("/connect/{name}", get(cluster_connect))
        // Nodes
        .route("/nodes", get(cluster_list_nodes))
        .route("/nodes/{name}", get(cluster_get_node))
        .route("/nodes/{name}/pause", axum::routing::post(cluster_pause_node))
        .route("/nodes/{name}/resume", axum::routing::post(cluster_resume_node))
        // Groups
        .route("/groups", get(cluster_list_groups))
        .route("/groups/{name}", get(cluster_get_group))
        .route("/groups/{name}/online", axum::routing::post(cluster_group_online))
        .route("/groups/{name}/offline", axum::routing::post(cluster_group_offline))
        .route("/groups/{name}/move/{target_node}", axum::routing::post(cluster_move_group))
        // Resources
        .route("/resources", get(cluster_list_resources))
        .route("/resources/{name}", get(cluster_get_resource))
        .route("/resources/{name}/online", axum::routing::post(cluster_resource_online))
        .route("/resources/{name}/offline", axum::routing::post(cluster_resource_offline))
        // CSV
        .route("/csv", get(cluster_list_csv))
        .route("/csv/check-path", get(cluster_csv_check_path))
        .route("/csv/{name}/maintenance", axum::routing::post(cluster_csv_maintenance))
}

pub fn hyperv_routes() -> Router<SharedState> {
    Router::new()
        // Host info
        .route("/host", get(hyperv_host_info))
        .route("/adapters", get(hyperv_list_adapters))
        // VMs
        .route("/vms", get(hyperv_list_vms).post(hyperv_create_vm))
        .route("/vms/{name}", get(hyperv_get_vm).delete(hyperv_delete_vm))
        .route("/vms/{name}/start", axum::routing::post(hyperv_start_vm))
        .route("/vms/{name}/stop", axum::routing::post(hyperv_stop_vm))
        .route("/vms/{name}/force-stop", axum::routing::post(hyperv_force_stop_vm))
        .route("/vms/{name}/pause", axum::routing::post(hyperv_pause_vm))
        .route("/vms/{name}/resume", axum::routing::post(hyperv_resume_vm))
        .route("/vms/{name}/save", axum::routing::post(hyperv_save_vm))
        .route("/vms/{name}/reset", axum::routing::post(hyperv_reset_vm))
        .route("/vms/{name}/export", axum::routing::post(hyperv_export_vm))
        // VM Disks
        .route("/vms/{name}/disks", get(hyperv_vm_disks))
        .route("/vms/{name}/disks/attach", axum::routing::post(hyperv_attach_disk))
        .route("/vms/{name}/disks/detach", axum::routing::post(hyperv_detach_disk))
        // VM DVD/ISO
        .route("/vms/{name}/dvd", get(hyperv_vm_dvd))
        .route("/vms/{name}/dvd/mount", axum::routing::post(hyperv_mount_iso))
        .route("/vms/{name}/dvd/eject", axum::routing::post(hyperv_eject_iso))
        .route("/vms/{name}/boot-order", axum::routing::post(hyperv_set_boot_order))
        // VM Snapshots
        .route("/vms/{name}/snapshots", get(hyperv_list_snapshots).post(hyperv_create_snapshot))
        .route("/vms/{name}/snapshots/{snapshot}", get(hyperv_get_snapshot))
        .route("/vms/{name}/snapshots/{snapshot}/apply", axum::routing::post(hyperv_apply_snapshot))
        .route("/vms/{name}/snapshots/{snapshot}/delete", axum::routing::delete(hyperv_delete_snapshot))
        // VM GPU
        .route("/vms/{name}/gpu", get(hyperv_vm_gpu_adapters))
        .route("/vms/{name}/gpu/add", axum::routing::post(hyperv_add_gpu))
        .route("/vms/{name}/gpu/remove", axum::routing::post(hyperv_remove_gpu))
        .route("/vms/{name}/gpu/configure", axum::routing::post(hyperv_configure_gpu))
        // VM DDA
        .route("/vms/{name}/dda", get(hyperv_vm_dda_devices))
        .route("/vms/{name}/dda/assign", axum::routing::post(hyperv_assign_device))
        .route("/vms/{name}/dda/remove", axum::routing::post(hyperv_remove_device))
        // Switches
        .route("/switches", get(hyperv_list_switches).post(hyperv_create_switch))
        .route("/switches/{name}", get(hyperv_get_switch).delete(hyperv_delete_switch))
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
