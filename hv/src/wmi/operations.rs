//! High-level WMI operations for Hyper-V management
//!
//! Provides functions for VM lifecycle, switch management, and snapshots
//! using the Msvm_* WMI classes.

use super::msvm::*;
use super::{hyperv, WmiConnection, WmiObject};
use crate::error::{HvError, Result};

// ============================================================================
// VM Operations
// ============================================================================

/// List all VMs with detailed information
pub fn list_vms(conn: &WmiConnection) -> Result<Vec<MsvmVm>> {
    // Query only real VMs, not the host computer system
    let query = r#"SELECT * FROM Msvm_ComputerSystem WHERE Caption = 'Virtual Machine'"#;
    let results = conn.query(query)?;

    let mut vms = Vec::new();
    for result in results {
        let obj = result?;
        let mut vm = MsvmVm::from_wmi(&obj)?;

        // Get additional settings (memory, CPU, generation)
        if let Ok(settings) = get_vm_settings(conn, &vm.id) {
            vm.generation = settings.generation;
        }
        if let Ok(mem) = get_vm_memory_settings(conn, &vm.id) {
            vm.memory_mb = Some(mem.virtual_quantity_mb);
        }
        if let Ok(proc) = get_vm_processor_settings(conn, &vm.id) {
            vm.processor_count = Some(proc.virtual_quantity);
        }

        vms.push(vm);
    }

    Ok(vms)
}

/// Get a VM by name
pub fn get_vm_by_name(conn: &WmiConnection, name: &str) -> Result<MsvmVm> {
    let query = format!(
        r#"SELECT * FROM Msvm_ComputerSystem WHERE Caption = 'Virtual Machine' AND ElementName = '{}'"#,
        escape_wql(name)
    );
    let mut results = conn.query(&query)?;

    let obj = results
        .next()
        .ok_or_else(|| HvError::VmNotFound(name.to_string()))??;

    let mut vm = MsvmVm::from_wmi(&obj)?;

    // Get additional settings
    if let Ok(settings) = get_vm_settings(conn, &vm.id) {
        vm.generation = settings.generation;
    }
    if let Ok(mem) = get_vm_memory_settings(conn, &vm.id) {
        vm.memory_mb = Some(mem.virtual_quantity_mb);
    }
    if let Ok(proc) = get_vm_processor_settings(conn, &vm.id) {
        vm.processor_count = Some(proc.virtual_quantity);
    }

    Ok(vm)
}

/// Get a VM by GUID
pub fn get_vm_by_id(conn: &WmiConnection, id: &str) -> Result<MsvmVm> {
    let query = format!(
        r#"SELECT * FROM Msvm_ComputerSystem WHERE Name = '{}'"#,
        escape_wql(id)
    );
    let mut results = conn.query(&query)?;

    let obj = results
        .next()
        .ok_or_else(|| HvError::VmNotFound(id.to_string()))??;

    MsvmVm::from_wmi(&obj)
}

/// Get VM settings (Msvm_VirtualSystemSettingData)
pub fn get_vm_settings(conn: &WmiConnection, vm_id: &str) -> Result<MsvmVmSettings> {
    // Get the active settings (SettingType = 3 is Current)
    let query = format!(
        r#"SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemIdentifier = '{}' AND VirtualSystemType = 'Microsoft:Hyper-V:System:Realized'"#,
        escape_wql(vm_id)
    );
    let mut results = conn.query(&query)?;

    let obj = results
        .next()
        .ok_or_else(|| HvError::WmiError("VM settings not found".to_string()))??;

    MsvmVmSettings::from_wmi(&obj)
}

/// Get VM memory settings
pub fn get_vm_memory_settings(conn: &WmiConnection, vm_id: &str) -> Result<MsvmMemorySettings> {
    // First get the VM settings path
    let settings = get_vm_settings(conn, vm_id)?;

    // Query memory settings associated with these VM settings
    let query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_MemorySettingData"#,
        settings.path
    );
    let mut results = conn.query(&query)?;

    let obj = results
        .next()
        .ok_or_else(|| HvError::WmiError("Memory settings not found".to_string()))??;

    MsvmMemorySettings::from_wmi(&obj)
}

/// Get VM processor settings
pub fn get_vm_processor_settings(
    conn: &WmiConnection,
    vm_id: &str,
) -> Result<MsvmProcessorSettings> {
    let settings = get_vm_settings(conn, vm_id)?;

    let query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ProcessorSettingData"#,
        settings.path
    );
    let mut results = conn.query(&query)?;

    let obj = results
        .next()
        .ok_or_else(|| HvError::WmiError("Processor settings not found".to_string()))??;

    MsvmProcessorSettings::from_wmi(&obj)
}

/// Start a VM
pub fn start_vm(conn: &WmiConnection, vm_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    change_vm_state(conn, &vm.path, hyperv::RequestedState::Enabled)
}

/// Stop a VM (hard power off)
pub fn stop_vm(conn: &WmiConnection, vm_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    change_vm_state(conn, &vm.path, hyperv::RequestedState::Disabled)
}

/// Shutdown a VM gracefully
pub fn shutdown_vm(conn: &WmiConnection, vm_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;

    // Use the shutdown integration service
    let query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_SystemDevice ResultClass = Msvm_ShutdownComponent"#,
        vm.path
    );
    let mut results = conn.query(&query)?;

    if let Some(shutdown_result) = results.next() {
        let shutdown = shutdown_result?;
        let shutdown_path = shutdown.path()?;

        // Call InitiateShutdown method
        let result = conn.exec_method(&shutdown_path, "InitiateShutdown", None)?;
        if let Some(result_obj) = result {
            hyperv::check_job_result(conn, &result_obj)?;
        }
        Ok(())
    } else {
        // Fall back to hard stop if shutdown service not available
        change_vm_state(conn, &vm.path, hyperv::RequestedState::Disabled)
    }
}

/// Save a VM state
pub fn save_vm(conn: &WmiConnection, vm_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    change_vm_state(conn, &vm.path, hyperv::RequestedState::Saved)
}

/// Pause a VM
pub fn pause_vm(conn: &WmiConnection, vm_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    change_vm_state(conn, &vm.path, hyperv::RequestedState::Paused)
}

/// Resume a paused or saved VM
pub fn resume_vm(conn: &WmiConnection, vm_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    change_vm_state(conn, &vm.path, hyperv::RequestedState::Enabled)
}

/// Change VM state
fn change_vm_state(
    conn: &WmiConnection,
    vm_path: &str,
    requested_state: hyperv::RequestedState,
) -> Result<()> {
    // Get method parameters
    let params = conn.get_method_params("Msvm_ComputerSystem", "RequestStateChange")?;
    params.put_u32("RequestedState", requested_state as u32)?;

    // Execute the method
    let result = conn.exec_method(vm_path, "RequestStateChange", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// VHD type for creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VhdType {
    /// Fixed size VHD - allocates all space upfront
    Fixed = 2,
    /// Dynamic VHD - grows as needed (default)
    Dynamic = 3,
    /// Differencing VHD - based on a parent VHD
    Differencing = 4,
}

/// VHD format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VhdFormat {
    /// Legacy VHD format
    Vhd = 2,
    /// Modern VHDX format (default, recommended)
    Vhdx = 3,
}

/// Create a new VHD file
pub fn create_vhd(
    conn: &WmiConnection,
    path: &str,
    size_bytes: u64,
    vhd_type: VhdType,
    vhd_format: VhdFormat,
) -> Result<()> {
    let ims = hyperv::get_ims(conn)?;
    let ims_path = ims.path()?;

    // Create VirtualHardDiskSettingData
    // Reference: https://learn.microsoft.com/en-us/windows/win32/hyperv_v2/msvm-virtualharddisksettingdata
    let vhd_settings_class = conn.get_class("Msvm_VirtualHardDiskSettingData")?;
    let vhd_settings = vhd_settings_class.spawn_instance()?;
    vhd_settings.put_string("Path", path)?;
    vhd_settings.put_u64("MaxInternalSize", size_bytes)?;
    // Type and Format are uint16 in WMI schema
    vhd_settings.put_u16("Type", vhd_type as u16)?;
    vhd_settings.put_u16("Format", vhd_format as u16)?;
    // BlockSize and LogicalSectorSize are uint32 - use 0 for defaults
    vhd_settings.put_u32("BlockSize", 0)?;
    vhd_settings.put_u32("LogicalSectorSize", 0)?;

    let vhd_settings_text = get_instance_text(conn, &vhd_settings)?;

    // Get method parameters for CreateVirtualHardDisk
    let params = conn.get_method_params("Msvm_ImageManagementService", "CreateVirtualHardDisk")?;
    params.put_string("VirtualDiskSettingData", &vhd_settings_text)?;

    // Create the VHD
    let result = conn.exec_method(&ims_path, "CreateVirtualHardDisk", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// Create a new VM with a new VHD
///
/// This is the standard way to create a VM - it creates both the VM and a new virtual hard disk.
///
/// # Arguments
/// * `conn` - WMI connection
/// * `name` - VM name
/// * `memory_mb` - Memory in MB
/// * `cpu_count` - Number of virtual CPUs
/// * `generation` - VM generation (1 or 2, use 2 for UEFI)
/// * `vhd_path` - Path where the new VHD will be created
/// * `vhd_size_bytes` - Size of the VHD in bytes
/// * `switch_name` - Optional virtual switch to connect to
pub fn create_vm(
    conn: &WmiConnection,
    name: &str,
    memory_mb: u64,
    cpu_count: u32,
    generation: u32,
    vhd_path: &str,
    vhd_size_bytes: u64,
    switch_name: Option<&str>,
) -> Result<MsvmVm> {
    // Create the base VM
    let vm_id = create_vm_internal(conn, name, memory_mb, cpu_count, generation)?;

    // Create and attach the new VHD
    let vhd_format = if vhd_path.to_lowercase().ends_with(".vhdx") {
        VhdFormat::Vhdx
    } else {
        VhdFormat::Vhd
    };
    create_vhd(conn, vhd_path, vhd_size_bytes, VhdType::Dynamic, vhd_format)?;
    add_vhd_to_vm(conn, &vm_id, vhd_path)?;

    // Connect to virtual switch if specified
    if let Some(switch) = switch_name {
        connect_vm_to_switch(conn, name, switch)?;
    }

    get_vm_by_id(conn, &vm_id)
}

/// Create a new VM with an existing VHD
///
/// Use this when you already have a VHD (e.g., from a template or previous VM).
///
/// # Arguments
/// * `conn` - WMI connection
/// * `name` - VM name
/// * `memory_mb` - Memory in MB
/// * `cpu_count` - Number of virtual CPUs
/// * `generation` - VM generation (1 or 2, use 2 for UEFI)
/// * `vhd_path` - Path to the existing VHD to attach
/// * `switch_name` - Optional virtual switch to connect to
pub fn create_vm_with_vhd(
    conn: &WmiConnection,
    name: &str,
    memory_mb: u64,
    cpu_count: u32,
    generation: u32,
    vhd_path: &str,
    switch_name: Option<&str>,
) -> Result<MsvmVm> {
    // Create the base VM
    let vm_id = create_vm_internal(conn, name, memory_mb, cpu_count, generation)?;

    // Attach the existing VHD
    add_vhd_to_vm(conn, &vm_id, vhd_path)?;

    // Connect to virtual switch if specified
    if let Some(switch) = switch_name {
        connect_vm_to_switch(conn, name, switch)?;
    }

    get_vm_by_id(conn, &vm_id)
}

/// Internal helper to create the base VM (without VHD)
fn create_vm_internal(
    conn: &WmiConnection,
    name: &str,
    memory_mb: u64,
    cpu_count: u32,
    generation: u32,
) -> Result<String> {
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    // Create VM settings data
    let settings_class = conn.get_class("Msvm_VirtualSystemSettingData")?;
    let settings = settings_class.spawn_instance()?;
    settings.put_string("ElementName", name)?;
    settings.put_string(
        "VirtualSystemSubType",
        if generation == 2 {
            "Microsoft:Hyper-V:SubType:2"
        } else {
            "Microsoft:Hyper-V:SubType:1"
        },
    )?;

    // Serialize settings to XML
    let settings_text = get_instance_text(conn, &settings)?;

    // Get method parameters for DefineSystem
    let define_params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "DefineSystem")?;
    define_params.put_string("SystemSettings", &settings_text)?;

    // Create the VM
    let result = conn.exec_method(&vsms_path, "DefineSystem", Some(&define_params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;

        // Get the created VM
        let vm_path = result_obj.get_string("ResultingSystem")?;
        if let Some(path) = vm_path {
            // Extract VM ID from path and fetch full VM info
            let vm_obj = conn.get_object(&path)?;
            let vm_id = vm_obj.get_string_required("Name")?;

            // Configure memory
            configure_vm_memory(conn, &vm_id, memory_mb)?;

            // Configure CPU
            configure_vm_processor(conn, &vm_id, cpu_count)?;

            return Ok(vm_id);
        }
    }

    Err(HvError::OperationFailed(
        "Failed to create VM".to_string(),
    ))
}

/// Delete a VM
pub fn delete_vm(conn: &WmiConnection, vm_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "DestroySystem")?;
    params.put_string("AffectedSystem", &vm.path)?;

    let result = conn.exec_method(&vsms_path, "DestroySystem", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// Configure VM memory
pub fn configure_vm_memory(conn: &WmiConnection, vm_id: &str, memory_mb: u64) -> Result<()> {
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    // Get current memory settings
    let settings = get_vm_settings(conn, vm_id)?;
    let query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_MemorySettingData"#,
        settings.path
    );
    let mut results = conn.query(&query)?;

    let mem_obj = results
        .next()
        .ok_or_else(|| HvError::WmiError("Memory settings not found".to_string()))??;

    // Update memory
    mem_obj.put_u64("VirtualQuantity", memory_mb)?;
    mem_obj.put_u64("Reservation", memory_mb)?;
    mem_obj.put_u64("Limit", memory_mb)?;

    let mem_text = get_instance_text(conn, &mem_obj)?;

    let params = conn.get_method_params(
        "Msvm_VirtualSystemManagementService",
        "ModifyResourceSettings",
    )?;
    params.put_string_array("ResourceSettings", &[&mem_text])?;

    let result = conn.exec_method(&vsms_path, "ModifyResourceSettings", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// Configure VM processor count
pub fn configure_vm_processor(conn: &WmiConnection, vm_id: &str, cpu_count: u32) -> Result<()> {
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    let settings = get_vm_settings(conn, vm_id)?;
    let query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ProcessorSettingData"#,
        settings.path
    );
    let mut results = conn.query(&query)?;

    let proc_obj = results
        .next()
        .ok_or_else(|| HvError::WmiError("Processor settings not found".to_string()))??;

    proc_obj.put_u32("VirtualQuantity", cpu_count)?;

    let proc_text = get_instance_text(conn, &proc_obj)?;

    let params = conn.get_method_params(
        "Msvm_VirtualSystemManagementService",
        "ModifyResourceSettings",
    )?;
    params.put_string_array("ResourceSettings", &[&proc_text])?;

    let result = conn.exec_method(&vsms_path, "ModifyResourceSettings", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// Add a VHD to a VM
pub fn add_vhd_to_vm(conn: &WmiConnection, vm_id: &str, vhd_path: &str) -> Result<()> {
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;
    let settings = get_vm_settings(conn, vm_id)?;

    // First, get or create a SCSI controller
    let scsi_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ResourceAllocationSettingData"#,
        settings.path
    );
    let scsi_results = conn.query(&scsi_query)?;

    let mut scsi_controller_path: Option<String> = None;
    for result in scsi_results {
        let obj = result?;
        if let Some(resource_type) = obj.get_u32("ResourceType")? {
            // ResourceType 6 = SCSI Controller
            if resource_type == 6 {
                scsi_controller_path = Some(obj.path()?);
                break;
            }
        }
    }

    // If no SCSI controller, add one
    let scsi_path = if let Some(path) = scsi_controller_path {
        path
    } else {
        add_scsi_controller(conn, &settings.path)?
    };

    // Get default disk drive settings from Hyper-V's resource pool
    let disk = get_default_resource(conn, ResourceSubtype::SyntheticDiskDrive)?;
    // Set parent (SCSI controller) and port address
    disk.put_string("Parent", &scsi_path)?;
    disk.put_string("AddressOnParent", "0")?;

    let disk_text = get_instance_text(conn, &disk)?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
    params.put_string("AffectedConfiguration", &settings.path)?;
    params.put_string_array("ResourceSettings", &[&disk_text])?;

    let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    // Query for the newly added disk drive
    let disk_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ResourceAllocationSettingData"#,
        settings.path
    );
    let disk_results = conn.query(&disk_query)?;

    let mut disk_path: Option<String> = None;
    for result in disk_results {
        let obj = result?;
        if let Some(resource_type) = obj.get_u32("ResourceType")? {
            // ResourceType 17 = Disk drive, and check it's attached to our SCSI controller
            if resource_type == 17 {
                if let Some(parent) = obj.get_string("Parent")? {
                    if parent == scsi_path {
                        disk_path = Some(obj.path()?);
                        break;
                    }
                }
            }
        }
    }

    // Now attach the VHD to the disk
    if let Some(disk_path) = disk_path {
        // Get default VHD settings from Hyper-V's resource pool
        let vhd_obj = get_default_resource(conn, ResourceSubtype::VirtualHardDisk)?;
        vhd_obj.put_string("Parent", &disk_path)?;
        // Set HostResource as array with VHD path
        vhd_obj.put_string_array("HostResource", &[vhd_path])?;

        let vhd_text = get_instance_text(conn, &vhd_obj)?;

        let params = conn
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        params.put_string("AffectedConfiguration", &settings.path)?;
        params.put_string_array("ResourceSettings", &[&vhd_text])?;

        let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;
        if let Some(result_obj) = result {
            hyperv::check_job_result(conn, &result_obj)?;
        }
    }

    Ok(())
}

/// Add a SCSI controller to a VM
fn add_scsi_controller(conn: &WmiConnection, settings_path: &str) -> Result<String> {
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    // Get default SCSI controller settings from Hyper-V's resource pool
    // This is the correct way - we get a pre-populated template with all required defaults
    let scsi = get_default_resource(conn, ResourceSubtype::ScsiController).map_err(|e| {
        HvError::OperationFailed(format!("Failed to get default SCSI controller: {}", e))
    })?;

    let scsi_text = get_instance_text(conn, &scsi).map_err(|e| {
        HvError::OperationFailed(format!("Failed to get SCSI controller text: {}", e))
    })?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
    params.put_string("AffectedConfiguration", settings_path)?;
    params.put_string_array("ResourceSettings", &[&scsi_text])?;

    let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj).map_err(|e| {
            HvError::OperationFailed(format!("SCSI controller job failed: {}", e))
        })?;
    }

    // Query for the newly added SCSI controller
    // ResultingResourceSettings is a string array which is hard to parse,
    // so we query for the SCSI controller directly
    let scsi_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ResourceAllocationSettingData"#,
        settings_path
    );
    let scsi_results = conn.query(&scsi_query)?;

    for result in scsi_results {
        let obj = result?;
        if let Some(resource_type) = obj.get_u32("ResourceType")? {
            if resource_type == 6 {
                // SCSI Controller
                return Ok(obj.path()?);
            }
        }
    }

    Err(HvError::OperationFailed(
        "SCSI controller was added but could not be found".to_string(),
    ))
}

/// Mount an ISO to a VM
pub fn mount_iso(conn: &WmiConnection, vm_name: &str, iso_path: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;
    let settings = get_vm_settings(conn, &vm.id)?;

    // Find or create DVD drive
    let dvd_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ResourceAllocationSettingData"#,
        settings.path
    );
    let dvd_results = conn.query(&dvd_query)?;

    let mut dvd_path: Option<String> = None;
    for result in dvd_results {
        let obj = result?;
        if let Some(resource_type) = obj.get_u32("ResourceType")? {
            // ResourceType 16 = DVD drive
            if resource_type == 16 {
                dvd_path = Some(obj.path()?);
                break;
            }
        }
    }

    // Create DVD drive if not found
    let dvd_path = if let Some(path) = dvd_path {
        path
    } else {
        add_dvd_drive(conn, &settings.path)?
    };

    // Get default ISO settings from Hyper-V's resource pool
    let iso_obj = get_default_resource(conn, ResourceSubtype::VirtualDvdDisk)?;
    iso_obj.put_string("Parent", &dvd_path)?;
    iso_obj.put_string_array("HostResource", &[iso_path])?;

    let iso_text = get_instance_text(conn, &iso_obj)?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
    params.put_string("AffectedConfiguration", &settings.path)?;
    params.put_string_array("ResourceSettings", &[&iso_text])?;

    let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// Add a DVD drive to a VM
fn add_dvd_drive(conn: &WmiConnection, settings_path: &str) -> Result<String> {
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    // Get SCSI controller first
    let scsi_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ResourceAllocationSettingData"#,
        settings_path
    );
    let scsi_results = conn.query(&scsi_query)?;

    let mut scsi_path: Option<String> = None;
    for result in scsi_results {
        let obj = result?;
        if let Some(resource_type) = obj.get_u32("ResourceType")? {
            if resource_type == 6 {
                // SCSI Controller
                scsi_path = Some(obj.path()?);
                break;
            }
        }
    }

    let scsi_path = scsi_path.ok_or_else(|| {
        HvError::OperationFailed("No SCSI controller found - cannot add DVD".to_string())
    })?;

    // Get default DVD drive settings from Hyper-V's resource pool
    let dvd = get_default_resource(conn, ResourceSubtype::DvdDrive)?;
    dvd.put_string("Parent", &scsi_path)?;
    dvd.put_string("AddressOnParent", "1")?; // Location 1 on SCSI

    let dvd_text = get_instance_text(conn, &dvd)?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
    params.put_string("AffectedConfiguration", settings_path)?;
    params.put_string_array("ResourceSettings", &[&dvd_text])?;

    let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    // Query for the newly added DVD drive
    let dvd_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_VirtualSystemSettingDataComponent ResultClass = Msvm_ResourceAllocationSettingData"#,
        settings_path
    );
    let dvd_results = conn.query(&dvd_query)?;

    for result in dvd_results {
        let obj = result?;
        if let Some(resource_type) = obj.get_u32("ResourceType")? {
            // ResourceType 16 = DVD drive
            if resource_type == 16 {
                return Ok(obj.path()?);
            }
        }
    }

    Err(HvError::OperationFailed(
        "DVD drive was added but could not be found".to_string(),
    ))
}

// ============================================================================
// Switch Operations
// ============================================================================

/// List all virtual switches
pub fn list_switches(conn: &WmiConnection) -> Result<Vec<MsvmSwitch>> {
    let query = "SELECT * FROM Msvm_VirtualEthernetSwitch";
    let results = conn.query(query)?;

    let mut switches = Vec::new();
    for result in results {
        let obj = result?;
        switches.push(MsvmSwitch::from_wmi(&obj)?);
    }

    Ok(switches)
}

/// Get a switch by name
pub fn get_switch_by_name(conn: &WmiConnection, name: &str) -> Result<MsvmSwitch> {
    let query = format!(
        r#"SELECT * FROM Msvm_VirtualEthernetSwitch WHERE ElementName = '{}'"#,
        escape_wql(name)
    );
    let mut results = conn.query(&query)?;

    let obj = results
        .next()
        .ok_or_else(|| HvError::SwitchNotFound(name.to_string()))??;

    MsvmSwitch::from_wmi(&obj)
}

/// Create a private or internal virtual switch
pub fn create_switch(conn: &WmiConnection, name: &str, _switch_type: &str) -> Result<MsvmSwitch> {
    let vesms = hyperv::get_vesms(conn)?;
    let vesms_path = vesms.path()?;

    // Create switch settings
    let settings_class = conn.get_class("Msvm_VirtualEthernetSwitchSettingData")?;
    let settings = settings_class.spawn_instance()?;
    settings.put_string("ElementName", name)?;

    let settings_text = get_instance_text(conn, &settings)?;

    let params = conn.get_method_params(
        "Msvm_VirtualEthernetSwitchManagementService",
        "DefineSystem",
    )?;
    params.put_string("SystemSettings", &settings_text)?;

    let result = conn.exec_method(&vesms_path, "DefineSystem", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    get_switch_by_name(conn, name)
}

/// Delete a virtual switch
pub fn delete_switch(conn: &WmiConnection, name: &str) -> Result<()> {
    let switch = get_switch_by_name(conn, name)?;
    let vesms = hyperv::get_vesms(conn)?;
    let vesms_path = vesms.path()?;

    let params = conn.get_method_params(
        "Msvm_VirtualEthernetSwitchManagementService",
        "DestroySystem",
    )?;
    params.put_string("AffectedSystem", &switch.path)?;

    let result = conn.exec_method(&vesms_path, "DestroySystem", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// Connect a VM to a switch
pub fn connect_vm_to_switch(conn: &WmiConnection, vm_name: &str, switch_name: &str) -> Result<()> {
    let vm = get_vm_by_name(conn, vm_name)?;
    let switch = get_switch_by_name(conn, switch_name)?;
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;
    let settings = get_vm_settings(conn, &vm.id)?;

    // Create synthetic network adapter
    let nic_class = conn.get_class("Msvm_SyntheticEthernetPortSettingData")?;
    let nic = nic_class.spawn_instance()?;
    nic.put_string("ElementName", "Network Adapter")?;
    nic.put_bool("StaticMacAddress", false)?;

    let nic_text = get_instance_text(conn, &nic)?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
    params.put_string("AffectedConfiguration", &settings.path)?;
    params.put_string_array("ResourceSettings", &[&nic_text])?;

    let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;

    let nic_path = if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
        result_obj.get_string("ResultingResourceSettings")?
    } else {
        None
    };

    // Connect the NIC to the switch
    if let Some(nic_path) = nic_path {
        let clean_path = nic_path.trim_matches(|c| c == '[' || c == ']' || c == '"');

        // Create connection to switch
        let conn_class = conn.get_class("Msvm_EthernetPortAllocationSettingData")?;
        let conn_obj = conn_class.spawn_instance()?;
        conn_obj.put_string("Parent", clean_path)?;
        conn_obj.put_string("HostResource", &switch.path)?;

        let conn_text = get_instance_text(conn, &conn_obj)?;

        let params = conn
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        params.put_string("AffectedConfiguration", &settings.path)?;
        params.put_string_array("ResourceSettings", &[&conn_text])?;

        let result = conn.exec_method(&vsms_path, "AddResourceSettings", Some(&params))?;
        if let Some(result_obj) = result {
            hyperv::check_job_result(conn, &result_obj)?;
        }
    }

    Ok(())
}

// ============================================================================
// Snapshot Operations
// ============================================================================

/// List snapshots for a VM
pub fn list_snapshots(conn: &WmiConnection, vm_name: &str) -> Result<Vec<MsvmSnapshot>> {
    let vm = get_vm_by_name(conn, vm_name)?;

    // Query snapshots (VirtualSystemType contains "Snapshot")
    let query = format!(
        r#"SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemIdentifier = '{}' AND VirtualSystemType LIKE '%Snapshot%'"#,
        escape_wql(&vm.id)
    );
    let results = conn.query(&query)?;

    let mut snapshots = Vec::new();
    for result in results {
        let obj = result?;
        snapshots.push(MsvmSnapshot::from_wmi(&obj)?);
    }

    Ok(snapshots)
}

/// Create a snapshot
pub fn create_snapshot(
    conn: &WmiConnection,
    vm_name: &str,
    snapshot_name: &str,
) -> Result<MsvmSnapshot> {
    let vm = get_vm_by_name(conn, vm_name)?;
    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    // Create snapshot settings
    let settings_class = conn.get_class("Msvm_VirtualSystemSnapshotSettingData")?;
    let settings = settings_class.spawn_instance()?;
    settings.put_string("ElementName", snapshot_name)?;

    let settings_text = get_instance_text(conn, &settings)?;

    let params = conn.get_method_params(
        "Msvm_VirtualSystemManagementService",
        "CreateSnapshot",
    )?;
    params.put_string("AffectedSystem", &vm.path)?;
    params.put_string("SnapshotSettings", &settings_text)?;
    params.put_u32("SnapshotType", 2)?; // Full snapshot

    let result = conn.exec_method(&vsms_path, "CreateSnapshot", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    // Get the created snapshot
    let snapshots = list_snapshots(conn, vm_name)?;
    snapshots
        .into_iter()
        .find(|s| s.name == snapshot_name)
        .ok_or_else(|| HvError::SnapshotNotFound(snapshot_name.to_string()))
}

/// Apply (restore) a snapshot
pub fn apply_snapshot(conn: &WmiConnection, vm_name: &str, snapshot_name: &str) -> Result<()> {
    let snapshots = list_snapshots(conn, vm_name)?;
    let snapshot = snapshots
        .into_iter()
        .find(|s| s.name == snapshot_name)
        .ok_or_else(|| HvError::SnapshotNotFound(snapshot_name.to_string()))?;

    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "ApplySnapshot")?;
    params.put_string("Snapshot", &snapshot.path)?;

    let result = conn.exec_method(&vsms_path, "ApplySnapshot", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

/// Delete a snapshot
pub fn delete_snapshot(conn: &WmiConnection, vm_name: &str, snapshot_name: &str) -> Result<()> {
    let snapshots = list_snapshots(conn, vm_name)?;
    let snapshot = snapshots
        .into_iter()
        .find(|s| s.name == snapshot_name)
        .ok_or_else(|| HvError::SnapshotNotFound(snapshot_name.to_string()))?;

    let vsms = hyperv::get_vsms(conn)?;
    let vsms_path = vsms.path()?;

    let params =
        conn.get_method_params("Msvm_VirtualSystemManagementService", "DestroySnapshot")?;
    params.put_string("AffectedSnapshot", &snapshot.path)?;

    let result = conn.exec_method(&vsms_path, "DestroySnapshot", Some(&params))?;

    if let Some(result_obj) = result {
        hyperv::check_job_result(conn, &result_obj)?;
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Resource subtype identifiers for Hyper-V
#[derive(Debug, Clone, Copy)]
pub enum ResourceSubtype {
    /// SCSI Controller
    ScsiController,
    /// Synthetic Disk Drive (attachment point on SCSI controller)
    SyntheticDiskDrive,
    /// Virtual Hard Disk (the actual VHD/VHDX)
    VirtualHardDisk,
    /// DVD Drive
    DvdDrive,
    /// Virtual CD/DVD Disk
    VirtualDvdDisk,
}

impl ResourceSubtype {
    /// Get the WMI resource subtype string
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceSubtype::ScsiController => "Microsoft:Hyper-V:Synthetic SCSI Controller",
            ResourceSubtype::SyntheticDiskDrive => "Microsoft:Hyper-V:Synthetic Disk Drive",
            ResourceSubtype::VirtualHardDisk => "Microsoft:Hyper-V:Virtual Hard Disk",
            ResourceSubtype::DvdDrive => "Microsoft:Hyper-V:Synthetic DVD Drive",
            ResourceSubtype::VirtualDvdDisk => "Microsoft:Hyper-V:Virtual CD/DVD Disk",
        }
    }

    /// Get the resource type number
    pub fn resource_type(&self) -> u16 {
        match self {
            ResourceSubtype::ScsiController => 6,
            ResourceSubtype::SyntheticDiskDrive => 17,
            ResourceSubtype::VirtualHardDisk => 31,
            ResourceSubtype::DvdDrive => 16,
            ResourceSubtype::VirtualDvdDisk => 21,
        }
    }
}

/// Get default resource settings from Hyper-V's allocation capabilities
///
/// This queries Msvm_ResourcePool -> Msvm_AllocationCapabilities -> Msvm_SettingsDefineCapabilities
/// to get the default/template settings for a given resource type.
///
/// This is the correct way to create resources - Hyper-V expects default instances
/// with all required properties pre-populated, not blank instances.
fn get_default_resource(conn: &WmiConnection, subtype: ResourceSubtype) -> Result<WmiObject> {
    // Query the resource pool for this subtype
    let pool_query = format!(
        r#"SELECT * FROM Msvm_ResourcePool WHERE ResourceSubType = '{}' AND Primordial = TRUE"#,
        subtype.as_str()
    );
    let mut pool_results = conn.query(&pool_query)?;

    let pool = pool_results.next().ok_or_else(|| {
        HvError::WmiError(format!(
            "Resource pool not found for subtype: {}",
            subtype.as_str()
        ))
    })??;
    let pool_path = pool.path()?;

    // Get the allocation capabilities for this pool
    let caps_query = format!(
        r#"ASSOCIATORS OF {{{}}} WHERE AssocClass = Msvm_ElementCapabilities ResultClass = Msvm_AllocationCapabilities"#,
        pool_path
    );
    let mut caps_results = conn.query(&caps_query)?;

    let caps = caps_results.next().ok_or_else(|| {
        HvError::WmiError("Allocation capabilities not found".to_string())
    })??;
    let caps_path = caps.path()?;

    // Get the SettingsDefineCapabilities associations to find the default settings
    // We need to query REFERENCES OF to get the association objects which contain ValueRole
    let assoc_query = format!(
        r#"REFERENCES OF {{{}}} WHERE ResultClass = Msvm_SettingsDefineCapabilities"#,
        caps_path
    );
    let assoc_results = conn.query(&assoc_query)?;

    // Look for the association with ValueRole = 0 (Default)
    for assoc_result in assoc_results {
        let assoc = assoc_result?;
        // ValueRole: 0=Default, 1=Supported, 2=Minimum, 3=Maximum, 4=Increment
        if let Some(role) = assoc.get_u32("ValueRole")? {
            if role == 0 {
                // Get the PartComponent which is the path to the default setting
                if let Some(part_component) = assoc.get_string("PartComponent")? {
                    // Get the actual default setting object
                    let default_setting = conn.get_object(&part_component)?;
                    return Ok(default_setting);
                }
            }
        }
    }

    Err(HvError::WmiError(format!(
        "Default settings not found for resource: {}",
        subtype.as_str()
    )))
}

/// Escape a string for WQL query
fn escape_wql(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Get the text representation of a WMI object for method calls
///
/// Uses WMI DTD 2.0 format which is required for embedded instances in Hyper-V WMI methods.
fn get_instance_text(_conn: &WmiConnection, obj: &WmiObject) -> Result<String> {
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::System::Wmi::{
        IWbemObjectTextSrc, WbemObjectTextSrc, WMI_OBJ_TEXT_WMI_DTD_2_0,
    };

    unsafe {
        // Create the text source object
        let text_src: IWbemObjectTextSrc =
            CoCreateInstance(&WbemObjectTextSrc, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| HvError::WmiError(format!("Failed to create text source: {:?}", e)))?;

        // Get text in WMI DTD 2.0 format (required for embedded instances)
        let text = text_src
            .GetText(0, obj.inner(), WMI_OBJ_TEXT_WMI_DTD_2_0.0 as u32, None)
            .map_err(|e| HvError::WmiError(format!("Failed to get object text: {:?}", e)))?;

        Ok(text.to_string())
    }
}
