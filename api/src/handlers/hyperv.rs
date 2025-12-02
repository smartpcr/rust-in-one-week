//! Hyper-V API handlers

use axum::{extract::Path, extract::Query, http::StatusCode, Json};

use crate::dto::*;
use crate::response::{api_error, ApiResponse, ApiResult};

#[cfg(windows)]
use hv::{HyperV, SnapshotType, SwitchType, VhdType, VmGeneration};

// =============================================================================
// Windows Implementation
// =============================================================================

#[cfg(windows)]
pub async fn hyperv_host_info() -> ApiResult<HostInfoDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_list_adapters() -> ApiResult<Vec<NetworkAdapterDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_list_vms() -> ApiResult<Vec<VmDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let vms = hv
        .list_vms()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let dtos: Vec<VmDto> = vms
        .into_iter()
        .map(|mut vm| VmDto {
            id: vm.id().to_string(),
            name: vm.name().to_string(),
            state: vm.state().map(|s| format!("{:?}", s)).unwrap_or_default(),
            cpu_count: vm.cpu_count().ok(),
            memory_mb: vm.memory_mb().ok(),
            uptime_seconds: None,
        })
        .collect();
    Ok(Json(ApiResponse::success(dtos)))
}

#[cfg(windows)]
pub async fn hyperv_get_vm(Path(name): Path<String>) -> ApiResult<VmDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    Ok(Json(ApiResponse::success(VmDto {
        id: vm.id().to_string(),
        name: vm.name().to_string(),
        state: vm.state().map(|s| format!("{:?}", s)).unwrap_or_default(),
        cpu_count: vm.cpu_count().ok(),
        memory_mb: vm.memory_mb().ok(),
        uptime_seconds: None,
    })))
}

#[cfg(windows)]
pub async fn hyperv_create_vm(Json(req): Json<CreateVmRequest>) -> ApiResult<VmDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let generation = match req.generation.unwrap_or(2) {
        1 => VmGeneration::Gen1,
        _ => VmGeneration::Gen2,
    };
    let mut vm = hv
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

#[cfg(windows)]
pub async fn hyperv_delete_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.delete_vm(&name)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_start_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vm.start()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_stop_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vm.stop()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_force_stop_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vm.force_stop()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_pause_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vm.pause()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_resume_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vm.resume()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_save_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vm.save()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_reset_vm(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let mut vm = hv
        .get_vm(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vm.force_stop()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    vm.start()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_export_vm(
    Path(name): Path<String>,
    Json(req): Json<ExportVmRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.export_vm(&name, &req.path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_vm_disks(Path(name): Path<String>) -> ApiResult<Vec<DiskDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_attach_disk(
    Path(name): Path<String>,
    Json(req): Json<AttachDiskRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.add_hard_disk_drive(&name, &req.vhd_path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_detach_disk(
    Path(name): Path<String>,
    Json(req): Json<DetachDiskRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.remove_hard_disk_drive(&name, req.controller_number, req.controller_location)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_vm_dvd(Path(name): Path<String>) -> ApiResult<Vec<DiskDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_mount_iso(
    Path(name): Path<String>,
    Json(req): Json<MountIsoRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.mount_iso(&name, &req.iso_path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_eject_iso(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.eject_iso(&name)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_set_boot_order(
    Path(name): Path<String>,
    Json(req): Json<BootOrderRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let devices: Vec<&str> = req.devices.iter().map(|s| s.as_str()).collect();
    hv.set_boot_order(&name, &devices)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_list_snapshots(Path(name): Path<String>) -> ApiResult<Vec<SnapshotDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let snapshots = hv
        .list_snapshots(&name)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let dtos = snapshots
        .into_iter()
        .map(|s| SnapshotDto {
            name: s.name().to_string(),
            id: s.id().to_string(),
            vm_name: s.vm_name().to_string(),
            creation_time: s.creation_time().ok(),
            parent_name: s.parent_name().ok().flatten(),
        })
        .collect();
    Ok(Json(ApiResponse::success(dtos)))
}

#[cfg(windows)]
pub async fn hyperv_get_snapshot(
    Path((name, snapshot)): Path<(String, String)>,
) -> ApiResult<SnapshotDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let s = hv
        .get_snapshot(&name, &snapshot)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    Ok(Json(ApiResponse::success(SnapshotDto {
        name: s.name().to_string(),
        id: s.id().to_string(),
        vm_name: s.vm_name().to_string(),
        creation_time: s.creation_time().ok(),
        parent_name: s.parent_name().ok().flatten(),
    })))
}

#[cfg(windows)]
pub async fn hyperv_create_snapshot(
    Path(name): Path<String>,
    Json(req): Json<CreateSnapshotRequest>,
) -> ApiResult<SnapshotDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let snapshot_type = match req.snapshot_type.as_deref() {
        Some("Production") => SnapshotType::Production,
        Some("ProductionOnly") => SnapshotType::Production,
        _ => SnapshotType::Standard,
    };
    let s = hv
        .create_snapshot(&name, &req.name, snapshot_type)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success(SnapshotDto {
        name: s.name().to_string(),
        id: s.id().to_string(),
        vm_name: s.vm_name().to_string(),
        creation_time: s.creation_time().ok(),
        parent_name: s.parent_name().ok().flatten(),
    })))
}

#[cfg(windows)]
pub async fn hyperv_apply_snapshot(
    Path((name, snapshot)): Path<(String, String)>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let s = hv
        .get_snapshot(&name, &snapshot)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    s.apply()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_delete_snapshot(
    Path((name, snapshot)): Path<(String, String)>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let s = hv
        .get_snapshot(&name, &snapshot)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    s.delete()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_list_switches() -> ApiResult<Vec<SwitchDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_get_switch(Path(name): Path<String>) -> ApiResult<SwitchDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let s = hv
        .get_switch(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    Ok(Json(ApiResponse::success(SwitchDto {
        name: s.name().to_string(),
        id: s.id().to_string(),
        switch_type: format!("{:?}", s.switch_type()),
    })))
}

#[cfg(windows)]
pub async fn hyperv_create_switch(Json(req): Json<CreateSwitchRequest>) -> ApiResult<SwitchDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let s = match req.switch_type.to_lowercase().as_str() {
        "external" => {
            let adapter = req.network_adapter.ok_or_else(|| {
                api_error(
                    StatusCode::BAD_REQUEST,
                    "network_adapter required for external switch",
                )
            })?;
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

#[cfg(windows)]
pub async fn hyperv_delete_switch(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let s = hv
        .get_switch(&name)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    s.delete()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_get_vhd_info(Query(req): Query<VhdPathRequest>) -> ApiResult<VhdDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let vhd = hv
        .get_vhd(&req.path)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    Ok(Json(ApiResponse::success(VhdDto {
        path: vhd.path().to_string(),
        format: format!("{:?}", vhd.format()),
        vhd_type: format!("{:?}", vhd.vhd_type()),
        max_size_bytes: vhd.max_size_bytes().unwrap_or(0),
        file_size_bytes: vhd.file_size_bytes().unwrap_or(0),
        parent_path: None,
        is_attached: vhd.is_attached().unwrap_or(false),
    })))
}

#[cfg(windows)]
pub async fn hyperv_create_vhd(Json(req): Json<CreateVhdRequest>) -> ApiResult<VhdDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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
        parent_path: None,
        is_attached: vhd.is_attached().unwrap_or(false),
    })))
}

#[cfg(windows)]
pub async fn hyperv_resize_vhd(Json(req): Json<ResizeVhdRequest>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let vhd = hv
        .get_vhd(&req.path)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vhd.resize(req.size_bytes)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_compact_vhd(Json(req): Json<VhdPathRequest>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let vhd = hv
        .get_vhd(&req.path)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vhd.compact()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_mount_vhd(Json(req): Json<VhdPathRequest>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let vhd = hv
        .get_vhd(&req.path)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vhd.mount(false)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_dismount_vhd(Json(req): Json<VhdPathRequest>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let vhd = hv
        .get_vhd(&req.path)
        .map_err(|e| api_error(StatusCode::NOT_FOUND, &e.to_string()))?;
    vhd.dismount()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_create_diff_vhd(Json(req): Json<DiffVhdRequest>) -> ApiResult<VhdDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let vhd = hv
        .create_differencing_vhd(&req.path, &req.parent_path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success(VhdDto {
        path: vhd.path().to_string(),
        format: format!("{:?}", vhd.format()),
        vhd_type: format!("{:?}", vhd.vhd_type()),
        max_size_bytes: vhd.max_size_bytes().unwrap_or(0),
        file_size_bytes: vhd.file_size_bytes().unwrap_or(0),
        parent_path: Some(req.parent_path.clone()),
        is_attached: vhd.is_attached().unwrap_or(false),
    })))
}

#[cfg(windows)]
pub async fn hyperv_initialize_vhd(Json(req): Json<InitVhdRequest>) -> ApiResult<String> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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
        .initialize_vhd(
            &req.path,
            partition_style,
            file_system,
            req.label.as_deref(),
        )
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success(drive_letter)))
}

#[cfg(windows)]
pub async fn hyperv_iso_editions(
    Query(req): Query<IsoPathQuery>,
) -> ApiResult<Vec<WindowsEditionDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_create_vhdx_from_iso(
    Json(req): Json<CreateVhdxFromIsoRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.create_vhdx_from_iso(
        &req.iso_path,
        &req.vhdx_path,
        req.size_gb,
        req.edition_index,
    )
    .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_list_gpus() -> ApiResult<Vec<GpuDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_list_partitionable_gpus() -> ApiResult<Vec<GpuDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_vm_gpu_adapters(Path(name): Path<String>) -> ApiResult<Vec<GpuAdapterDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let adapters = hv
        .get_vm_gpu_adapters(&name)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let dtos = adapters
        .into_iter()
        .map(|a| GpuAdapterDto {
            vm_name: a.vm_name,
            instance_path: a.instance_path,
            min_partition_vram: a.min_partition_vram.unwrap_or(0),
            max_partition_vram: a.max_partition_vram.unwrap_or(0),
            optimal_partition_vram: a.optimal_partition_vram.unwrap_or(0),
        })
        .collect();
    Ok(Json(ApiResponse::success(dtos)))
}

#[cfg(windows)]
pub async fn hyperv_add_gpu(
    Path(name): Path<String>,
    Json(req): Json<AddGpuRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.add_gpu_to_vm(&name, req.instance_path.as_deref())
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_remove_gpu(Path(name): Path<String>) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.remove_gpu_from_vm(&name)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_configure_gpu(
    Path(name): Path<String>,
    Json(req): Json<ConfigureGpuRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.configure_vm_for_gpu(&name, req.low_mmio_gb, req.high_mmio_gb)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_dda_support() -> ApiResult<DdaSupportDto> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_dda_devices() -> ApiResult<Vec<AssignableDeviceDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_device_path(Query(req): Query<DevicePathRequest>) -> ApiResult<String> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    let path = hv
        .get_device_location_path(&req.instance_id)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success(path)))
}

#[cfg(windows)]
pub async fn hyperv_dismount_device(
    Json(req): Json<DeviceLocationRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.dismount_device(&req.location_path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_mount_device(
    Json(req): Json<DeviceLocationRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.mount_device(&req.location_path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_vm_dda_devices(
    Path(name): Path<String>,
) -> ApiResult<Vec<AssignableDeviceDto>> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
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

#[cfg(windows)]
pub async fn hyperv_assign_device(
    Path(name): Path<String>,
    Json(req): Json<DeviceLocationRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.assign_device_to_vm(&name, &req.location_path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

#[cfg(windows)]
pub async fn hyperv_remove_device(
    Path(name): Path<String>,
    Json(req): Json<DeviceLocationRequest>,
) -> ApiResult<&'static str> {
    let hv =
        HyperV::new().map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    hv.remove_assigned_device(&name, &req.location_path)
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    Ok(Json(ApiResponse::success("ok")))
}

// =============================================================================
// Non-Windows Stubs
// =============================================================================

#[cfg(not(windows))]
fn not_supported() -> (StatusCode, Json<ApiResponse<()>>) {
    api_error(
        StatusCode::NOT_IMPLEMENTED,
        "Hyper-V API only available on Windows",
    )
}

#[cfg(not(windows))]
pub async fn hyperv_host_info() -> ApiResult<HostInfoDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_list_adapters() -> ApiResult<Vec<NetworkAdapterDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_list_vms() -> ApiResult<Vec<VmDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_get_vm(_: Path<String>) -> ApiResult<VmDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_create_vm(_: Json<CreateVmRequest>) -> ApiResult<VmDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_delete_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_start_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_stop_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_force_stop_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_pause_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_resume_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_save_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_reset_vm(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_export_vm(
    _: Path<String>,
    _: Json<ExportVmRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_vm_disks(_: Path<String>) -> ApiResult<Vec<DiskDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_attach_disk(
    _: Path<String>,
    _: Json<AttachDiskRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_detach_disk(
    _: Path<String>,
    _: Json<DetachDiskRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_vm_dvd(_: Path<String>) -> ApiResult<Vec<DiskDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_mount_iso(
    _: Path<String>,
    _: Json<MountIsoRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_eject_iso(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_set_boot_order(
    _: Path<String>,
    _: Json<BootOrderRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_list_snapshots(_: Path<String>) -> ApiResult<Vec<SnapshotDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_get_snapshot(_: Path<(String, String)>) -> ApiResult<SnapshotDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_create_snapshot(
    _: Path<String>,
    _: Json<CreateSnapshotRequest>,
) -> ApiResult<SnapshotDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_apply_snapshot(_: Path<(String, String)>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_delete_snapshot(_: Path<(String, String)>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_list_switches() -> ApiResult<Vec<SwitchDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_get_switch(_: Path<String>) -> ApiResult<SwitchDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_create_switch(_: Json<CreateSwitchRequest>) -> ApiResult<SwitchDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_delete_switch(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_get_vhd_info(_: Query<VhdPathRequest>) -> ApiResult<VhdDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_create_vhd(_: Json<CreateVhdRequest>) -> ApiResult<VhdDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_resize_vhd(_: Json<ResizeVhdRequest>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_compact_vhd(_: Json<VhdPathRequest>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_mount_vhd(_: Json<VhdPathRequest>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_dismount_vhd(_: Json<VhdPathRequest>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_create_diff_vhd(_: Json<DiffVhdRequest>) -> ApiResult<VhdDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_initialize_vhd(_: Json<InitVhdRequest>) -> ApiResult<String> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_iso_editions(_: Query<IsoPathQuery>) -> ApiResult<Vec<WindowsEditionDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_create_vhdx_from_iso(
    _: Json<CreateVhdxFromIsoRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_list_gpus() -> ApiResult<Vec<GpuDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_list_partitionable_gpus() -> ApiResult<Vec<GpuDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_vm_gpu_adapters(_: Path<String>) -> ApiResult<Vec<GpuAdapterDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_add_gpu(_: Path<String>, _: Json<AddGpuRequest>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_remove_gpu(_: Path<String>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_configure_gpu(
    _: Path<String>,
    _: Json<ConfigureGpuRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_dda_support() -> ApiResult<DdaSupportDto> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_dda_devices() -> ApiResult<Vec<AssignableDeviceDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_device_path(_: Query<DevicePathRequest>) -> ApiResult<String> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_dismount_device(_: Json<DeviceLocationRequest>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_mount_device(_: Json<DeviceLocationRequest>) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_vm_dda_devices(_: Path<String>) -> ApiResult<Vec<AssignableDeviceDto>> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_assign_device(
    _: Path<String>,
    _: Json<DeviceLocationRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}

#[cfg(not(windows))]
pub async fn hyperv_remove_device(
    _: Path<String>,
    _: Json<DeviceLocationRequest>,
) -> ApiResult<&'static str> {
    Err(not_supported())
}
