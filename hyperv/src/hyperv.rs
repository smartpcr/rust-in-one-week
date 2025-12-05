use crate::checkpoint::{Checkpoint, CheckpointSettings};
use crate::error::{Error, Result};
use crate::network::{NetworkAdapter, NetworkAdapterSettings, VirtualSwitch};
use crate::storage::{ControllerType, DiskAttachment, IsoAttachment, VhdManager};
use crate::vm::{Generation, VirtualMachine, VmSettings, VmState};
use crate::wmi::{WbemClassObjectExt, WmiConnection};
use std::sync::Arc;
use windows::Win32::System::Wmi::IWbemClassObject;

/// Main entry point for Hyper-V management operations.
pub struct HyperV {
    connection: Arc<WmiConnection>,
}

impl HyperV {
    /// Connect to the Hyper-V WMI namespace.
    pub fn connect() -> Result<Self> {
        let connection = Arc::new(WmiConnection::connect()?);
        Ok(Self { connection })
    }

    // ========== VM Operations ==========

    /// List all virtual machines.
    pub fn list_vms(&self) -> Result<Vec<VirtualMachine>> {
        let query = "SELECT * FROM Msvm_ComputerSystem WHERE Caption = 'Virtual Machine'";
        let objects = self.connection.query(query)?;

        objects
            .iter()
            .map(|obj| VirtualMachine::from_wmi(obj, Arc::clone(&self.connection)))
            .collect()
    }

    /// Get a VM by name.
    pub fn get_vm(&self, name: &str) -> Result<VirtualMachine> {
        let query = format!(
            "SELECT * FROM Msvm_ComputerSystem WHERE Caption = 'Virtual Machine' AND ElementName = '{}'",
            name.replace('\'', "''")
        );
        let obj = self
            .connection
            .query_first(&query)?
            .ok_or_else(|| Error::VmNotFound(name.to_string()))?;

        VirtualMachine::from_wmi(&obj, Arc::clone(&self.connection))
    }

    /// Get a VM by ID (GUID).
    pub fn get_vm_by_id(&self, id: &str) -> Result<VirtualMachine> {
        let query = format!(
            "SELECT * FROM Msvm_ComputerSystem WHERE Caption = 'Virtual Machine' AND Name = '{}'",
            id.replace('\'', "''")
        );
        let obj = self
            .connection
            .query_first(&query)?
            .ok_or_else(|| Error::VmNotFound(id.to_string()))?;

        VirtualMachine::from_wmi(&obj, Arc::clone(&self.connection))
    }

    /// Create a new virtual machine.
    pub fn create_vm(&self, settings: &VmSettings) -> Result<VirtualMachine> {
        settings.validate()?;

        // Get the management service
        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        // Create VirtualSystemSettingData
        // Only set ElementName and VirtualSystemSubType during DefineSystem.
        // Other properties (Notes, paths, automatic actions) must be set via
        // ModifySystemSettings after VM creation.
        let vs_settings = self
            .connection
            .spawn_instance("Msvm_VirtualSystemSettingData")?;
        vs_settings.put_string("ElementName", &settings.name)?;
        vs_settings.put_string("VirtualSystemSubType", settings.generation.to_subtype())?;

        let vs_settings_text = vs_settings.get_text()?;

        // Call DefineSystem
        // Note: Only pass SystemSettings. ResourceSettings is optional and should NOT
        // be passed as an empty array - omit it entirely to match Hyper-V's expectations.
        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "DefineSystem")?;
        in_params.put_string("SystemSettings", &vs_settings_text)?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "DefineSystem", Some(&in_params))?;
        self.handle_job_result(&out_params, "DefineSystem")?;

        // Get the created VM
        let result_system = out_params
            .get_string_prop("ResultingSystem")?
            .ok_or_else(|| Error::OperationFailed {
                operation: "DefineSystem",
                return_value: 0,
                message: "No ResultingSystem returned".to_string(),
            })?;

        // Get the VM object
        let vm_obj = self.connection.get_object(&result_system)?;
        let mut vm = VirtualMachine::from_wmi(&vm_obj, Arc::clone(&self.connection))?;

        // Configure memory
        self.configure_memory(&vm, settings)?;

        // Configure processor
        self.configure_processor(&vm, settings)?;

        // Configure secure boot for Gen2
        if settings.generation == Generation::Gen2 && settings.secure_boot {
            self.configure_secure_boot(&vm, settings)?;
        }

        // Add default SCSI controller for Gen2
        if settings.generation == Generation::Gen2 {
            self.add_scsi_controller(&vm)?;
        }

        // Configure optional settings via ModifySystemSettings
        // These properties cannot be set during DefineSystem
        let _ = self.configure_optional_settings(&vm, settings);

        vm.refresh()?;
        Ok(vm)
    }

    /// Delete a virtual machine.
    pub fn delete_vm(&self, vm: &VirtualMachine) -> Result<()> {
        if vm.state() != VmState::Off {
            return Err(Error::InvalidState {
                vm_name: vm.name().to_string(),
                current: vm.state().to_error(),
                operation: "delete",
            });
        }

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        // Get the VM's settings path
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.CreationClassName='Msvm_ComputerSystem',Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState ResultClass=Msvm_VirtualSystemSettingData",
            vm.id()
        );
        let _settings = self
            .connection
            .query_first(&query)?
            .ok_or_else(|| Error::VmNotFound(vm.name().to_string()))?;

        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "DestroySystem")?;
        in_params.put_string(
            "AffectedSystem",
            &format!(
                "Msvm_ComputerSystem.CreationClassName='Msvm_ComputerSystem',Name='{}'",
                vm.id()
            ),
        )?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "DestroySystem", Some(&in_params))?;
        self.handle_job_result(&out_params, "DestroySystem")
    }

    // ========== Storage Operations ==========

    /// Get VHD manager for disk operations.
    pub fn vhd(&self) -> VhdManager {
        VhdManager::new(Arc::clone(&self.connection))
    }

    /// Attach a VHD to a VM.
    pub fn attach_vhd(&self, vm: &VirtualMachine, attachment: &DiskAttachment) -> Result<()> {
        attachment.validate()?;

        // Get VM settings
        let settings = self.get_vm_settings(vm)?;
        let settings_path = settings.get_path()?;

        // Find the appropriate controller
        let controller = self.find_or_create_controller(
            vm,
            attachment.controller_type,
            attachment.controller_number,
        )?;
        let controller_path = controller.get_path()?;

        // Create disk drive resource
        let drive = self.create_disk_drive(&controller_path, attachment.controller_location)?;

        // Add the drive to VM
        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        let drive_text = drive.get_text()?;
        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params.put_string("AffectedConfiguration", &settings_path)?;
        in_params.put_string_array("ResourceSettings", &[&drive_text])?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "AddResourceSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "AddResourceSettings")?;

        // Get the created drive path from result
        // For simplicity, re-query for the drive
        let new_drive =
            self.find_disk_drive(vm, &controller_path, attachment.controller_location)?;
        let new_drive_path = new_drive.get_path()?;

        // Create VHD attachment
        let vhd_resource = self
            .connection
            .spawn_instance("Msvm_StorageAllocationSettingData")?;
        vhd_resource.put_string("Parent", &new_drive_path)?;
        vhd_resource.put_string_array("HostResource", &[&attachment.vhd_path])?;

        let vhd_text = vhd_resource.get_text()?;
        let in_params2 = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params2.put_string("AffectedConfiguration", &settings_path)?;
        in_params2.put_string_array("ResourceSettings", &[&vhd_text])?;

        let out_params2 =
            self.connection
                .exec_method(&mgmt_path, "AddResourceSettings", Some(&in_params2))?;
        self.handle_job_result(&out_params2, "AddResourceSettings")
    }

    /// Mount an ISO to a VM.
    pub fn mount_iso(&self, vm: &VirtualMachine, attachment: &IsoAttachment) -> Result<()> {
        attachment.validate()?;

        let settings = self.get_vm_settings(vm)?;
        let settings_path = settings.get_path()?;

        let controller = self.find_or_create_controller(
            vm,
            attachment.controller_type,
            attachment.controller_number,
        )?;
        let controller_path = controller.get_path()?;

        // Create DVD drive
        let dvd = self.create_dvd_drive(&controller_path, attachment.controller_location)?;

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        let dvd_text = dvd.get_text()?;
        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params.put_string("AffectedConfiguration", &settings_path)?;
        in_params.put_string_array("ResourceSettings", &[&dvd_text])?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "AddResourceSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "AddResourceSettings")?;

        // Get created DVD drive and attach ISO
        let new_dvd = self.find_dvd_drive(vm, &controller_path, attachment.controller_location)?;
        let new_dvd_path = new_dvd.get_path()?;

        let iso_resource = self
            .connection
            .spawn_instance("Msvm_StorageAllocationSettingData")?;
        iso_resource.put_string("Parent", &new_dvd_path)?;
        iso_resource.put_string_array("HostResource", &[&attachment.iso_path])?;

        let iso_text = iso_resource.get_text()?;
        let in_params2 = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params2.put_string("AffectedConfiguration", &settings_path)?;
        in_params2.put_string_array("ResourceSettings", &[&iso_text])?;

        let out_params2 =
            self.connection
                .exec_method(&mgmt_path, "AddResourceSettings", Some(&in_params2))?;
        self.handle_job_result(&out_params2, "AddResourceSettings")
    }

    // ========== Network Operations ==========

    /// List all virtual switches.
    pub fn list_switches(&self) -> Result<Vec<VirtualSwitch>> {
        let query = "SELECT * FROM Msvm_VirtualEthernetSwitch";
        let objects = self.connection.query(query)?;

        objects.iter().map(VirtualSwitch::from_wmi).collect()
    }

    /// Get a virtual switch by name.
    pub fn get_switch(&self, name: &str) -> Result<VirtualSwitch> {
        let query = format!(
            "SELECT * FROM Msvm_VirtualEthernetSwitch WHERE ElementName = '{}'",
            name.replace('\'', "''")
        );
        let obj = self
            .connection
            .query_first(&query)?
            .ok_or_else(|| Error::SwitchNotFound(name.to_string()))?;

        VirtualSwitch::from_wmi(&obj)
    }

    /// Add a network adapter to a VM.
    pub fn add_network_adapter(
        &self,
        vm: &VirtualMachine,
        settings: &NetworkAdapterSettings,
    ) -> Result<NetworkAdapter> {
        settings.validate()?;

        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        // Get default network adapter from resource pool (matching C# implementation)
        // This is the correct way - get pre-populated template with all required defaults
        let adapter = self
            .connection
            .get_default_resource("Microsoft:Hyper-V:Synthetic Ethernet Port")?;

        // Set ElementName
        let adapter_name = settings.name.as_deref().unwrap_or("Network Adapter");
        adapter.put_string("ElementName", adapter_name)?;

        // Configure MAC Address
        if let Some(ref mac) = settings.mac_address {
            adapter.put_bool("StaticMacAddress", true)?;
            adapter.put_string("Address", mac)?;
        }
        // If no static MAC provided, leave default (dynamic)

        let adapter_text = adapter.get_text()?;

        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params.put_string("AffectedConfiguration", &settings_path)?;
        in_params.put_string_array("ResourceSettings", &[&adapter_text])?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "AddResourceSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "AddResourceSettings")?;

        // Get created adapter
        let adapters = self.list_network_adapters(vm)?;
        adapters
            .into_iter()
            .last()
            .ok_or_else(|| Error::OperationFailed {
                operation: "AddNetworkAdapter",
                return_value: 0,
                message: "Failed to find created adapter".to_string(),
            })
    }

    /// List network adapters attached to a VM.
    pub fn list_network_adapters(&self, vm: &VirtualMachine) -> Result<Vec<NetworkAdapter>> {
        // Get VM settings first
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        // Query adapters associated with VM settings
        let query = format!(
            "ASSOCIATORS OF {{{}}} WHERE AssocClass=Msvm_VirtualSystemSettingDataComponent ResultClass=Msvm_SyntheticEthernetPortSettingData",
            settings_path
        );
        let objects = self.connection.query(&query)?;

        objects.iter().map(NetworkAdapter::from_wmi).collect()
    }

    /// Connect a network adapter to a switch.
    pub fn connect_adapter_to_switch(
        &self,
        vm: &VirtualMachine,
        adapter: &NetworkAdapter,
        switch: &VirtualSwitch,
    ) -> Result<()> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        // Find the switch port
        let switch_query = format!(
            "SELECT * FROM Msvm_VirtualEthernetSwitchSettingData WHERE VirtualSystemIdentifier = '{}'",
            switch.id()
        );
        let switch_settings = self
            .connection
            .query_first(&switch_query)?
            .ok_or_else(|| Error::SwitchNotFound(switch.name().to_string()))?;
        let switch_path = switch_settings.get_path()?;

        // Create connection
        let port = self
            .connection
            .spawn_instance("Msvm_EthernetPortAllocationSettingData")?;
        port.put_string("Parent", adapter.path())?;
        port.put_string_array("HostResource", &[&switch_path])?;

        let port_text = port.get_text()?;

        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params.put_string("AffectedConfiguration", &settings_path)?;
        in_params.put_string_array("ResourceSettings", &[&port_text])?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "AddResourceSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "AddResourceSettings")
    }

    // ========== Checkpoint Operations ==========

    /// List checkpoints for a VM.
    pub fn list_checkpoints(&self, vm: &VirtualMachine) -> Result<Vec<Checkpoint>> {
        // Query snapshots using VirtualSystemIdentifier (matching hv module)
        let escaped_id = vm.id().replace('\\', "\\\\").replace('\'', "\\'");
        let query = format!(
            "SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemIdentifier = '{}' \
             AND VirtualSystemType LIKE '%Snapshot%'",
            escaped_id
        );
        let objects = self.connection.query(&query)?;

        objects.iter().map(Checkpoint::from_wmi).collect()
    }

    /// Create a checkpoint.
    pub fn create_checkpoint(
        &self,
        vm: &VirtualMachine,
        settings: &CheckpointSettings,
    ) -> Result<Checkpoint> {
        settings.validate()?;

        // Record existing checkpoint IDs before creation (for reliable identification later)
        let existing_checkpoint_ids: std::collections::HashSet<String> = self
            .list_checkpoints(vm)?
            .into_iter()
            .map(|cp| cp.id)
            .collect();

        // Get Msvm_VirtualSystemSnapshotService - this is the correct service for CreateSnapshot
        // (Msvm_VirtualSystemManagementService does NOT have CreateSnapshot method)
        let snapshot_service = self.get_snapshot_service()?;
        let service_path = snapshot_service.get_path()?;

        // Create snapshot settings - only set ElementName
        let snapshot_settings = self
            .connection
            .spawn_instance("Msvm_VirtualSystemSnapshotSettingData")?;
        snapshot_settings.put_string("ElementName", &settings.name)?;

        let settings_text = snapshot_settings.get_text()?;

        // Get method params from Msvm_VirtualSystemSnapshotService class
        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemSnapshotService", "CreateSnapshot")?;
        // Use the actual WMI path from the VM object
        in_params.put_string("AffectedSystem", vm.path())?;
        in_params.put_string("SnapshotSettings", &settings_text)?;
        // SnapshotType: 2 = Full/Standard snapshot
        in_params.put_u32("SnapshotType", 2)?;

        let out_params =
            self.connection
                .exec_method(&service_path, "CreateSnapshot", Some(&in_params))?;

        // handle_job_result waits for job completion and returns ResultingSnapshot path
        let snapshot_path = self.handle_snapshot_job(&out_params)?;

        if let Some(path) = snapshot_path {
            let checkpoint_obj = self.connection.get_object(&path)?;
            return Checkpoint::from_wmi(&checkpoint_obj);
        }

        // Fallback: find the new checkpoint by comparing with existing IDs
        std::thread::sleep(std::time::Duration::from_millis(200));

        let checkpoints_after = self.list_checkpoints(vm)?;

        // Find checkpoint that wasn't in the original list
        let new_checkpoint = checkpoints_after
            .into_iter()
            .find(|cp| !existing_checkpoint_ids.contains(&cp.id));

        new_checkpoint.ok_or_else(|| Error::OperationFailed {
            operation: "CreateSnapshot",
            return_value: 0,
            message: "No new checkpoint found after creation".to_string(),
        })
    }

    /// Handle snapshot job result and return the ResultingSnapshot path
    fn handle_snapshot_job(&self, out_params: &IWbemClassObject) -> Result<Option<String>> {
        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);

        match return_value {
            0 => {
                // Completed synchronously - get ResultingSnapshot directly
                Ok(out_params.get_string_prop("ResultingSnapshot")?)
            }
            4096 => {
                // Job started - wait for completion and get result from job
                if let Some(job_path) = out_params.get_string_prop("Job")? {
                    self.wait_for_snapshot_job(&job_path)
                } else {
                    Ok(None)
                }
            }
            code => Err(Error::OperationFailed {
                operation: "CreateSnapshot",
                return_value: code,
                message: "CreateSnapshot failed".to_string(),
            }),
        }
    }

    /// Wait for snapshot job and return ResultingSnapshot path
    fn wait_for_snapshot_job(&self, job_path: &str) -> Result<Option<String>> {
        loop {
            let job = self.connection.get_object(job_path)?;
            let job_state = job.get_u16("JobState")?.unwrap_or(0);

            match job_state {
                7 => {
                    // Completed - try ASSOCIATORS OF to find the affected snapshot
                    let assoc_query = format!(
                        "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_VirtualSystemSettingData",
                        job_path.replace('\\', "\\\\")
                    );

                    if let Ok(results) = self.connection.query(&assoc_query) {
                        for obj in &results {
                            if let Ok(path) = obj.get_path() {
                                // Check if this is a snapshot (VirtualSystemType contains "Snapshot")
                                if let Ok(Some(vs_type)) = obj.get_string_prop("VirtualSystemType")
                                {
                                    if vs_type.contains("Snapshot") {
                                        return Ok(Some(path));
                                    }
                                }
                            }
                        }
                    }

                    return Ok(None);
                }
                8 | 9 | 10 | 11 => {
                    // Terminated, Killed, Exception, Service
                    let error_code = job.get_u32("ErrorCode")?.unwrap_or(0);
                    let error_desc = job.get_string_prop("ErrorDescription")?.unwrap_or_default();
                    return Err(Error::JobFailed {
                        operation: "CreateSnapshot",
                        error_code,
                        error_description: error_desc,
                    });
                }
                2 | 3 | 4 => {
                    // New, Starting, Running - keep waiting
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    /// Apply (restore) a checkpoint.
    pub fn apply_checkpoint(&self, vm: &mut VirtualMachine, checkpoint: &Checkpoint) -> Result<()> {
        if vm.state() != VmState::Off {
            return Err(Error::InvalidState {
                vm_name: vm.name().to_string(),
                current: vm.state().to_error(),
                operation: "apply checkpoint",
            });
        }

        // Use Msvm_VirtualSystemSnapshotService for snapshot operations
        let snapshot_service = self.get_snapshot_service()?;
        let service_path = snapshot_service.get_path()?;

        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemSnapshotService", "ApplySnapshot")?;
        in_params.put_string("Snapshot", checkpoint.path())?;

        let out_params =
            self.connection
                .exec_method(&service_path, "ApplySnapshot", Some(&in_params))?;
        self.handle_job_result(&out_params, "ApplySnapshot")?;

        vm.refresh()
    }

    /// Delete a checkpoint.
    pub fn delete_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        // Use Msvm_VirtualSystemSnapshotService for snapshot operations
        let snapshot_service = self.get_snapshot_service()?;
        let service_path = snapshot_service.get_path()?;

        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemSnapshotService", "DestroySnapshot")?;
        in_params.put_string("AffectedSnapshot", checkpoint.path())?;

        let out_params =
            self.connection
                .exec_method(&service_path, "DestroySnapshot", Some(&in_params))?;
        self.handle_job_result(&out_params, "DestroySnapshot")
    }

    // ========== Helper Methods ==========

    fn get_management_service(&self) -> Result<IWbemClassObject> {
        self.connection
            .query_first("SELECT * FROM Msvm_VirtualSystemManagementService")?
            .ok_or_else(|| Error::WmiQuery {
                query: "Msvm_VirtualSystemManagementService".to_string(),
                source: windows::core::Error::from_hresult(windows::core::HRESULT(-1)),
            })
    }

    fn get_snapshot_service(&self) -> Result<IWbemClassObject> {
        self.connection
            .query_first("SELECT * FROM Msvm_VirtualSystemSnapshotService")?
            .ok_or_else(|| Error::WmiQuery {
                query: "Msvm_VirtualSystemSnapshotService".to_string(),
                source: windows::core::Error::from_hresult(windows::core::HRESULT(-1)),
            })
    }

    fn get_vm_settings(&self, vm: &VirtualMachine) -> Result<IWbemClassObject> {
        // Use direct query matching hv module approach
        // VirtualSystemType = 'Microsoft:Hyper-V:System:Realized' gets the active settings
        let query = format!(
            "SELECT * FROM Msvm_VirtualSystemSettingData WHERE VirtualSystemIdentifier = '{}' \
             AND VirtualSystemType = 'Microsoft:Hyper-V:System:Realized'",
            vm.id().replace('\'', "''")
        );
        self.connection
            .query_first(&query)?
            .ok_or_else(|| Error::VmNotFound(vm.name().to_string()))
    }

    fn configure_memory(&self, vm: &VirtualMachine, settings: &VmSettings) -> Result<()> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        // Find memory settings
        let mem_query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_MemorySettingData",
            settings_path
        );
        let mem_settings = self
            .connection
            .query_first(&mem_query)?
            .ok_or_else(|| Error::VmNotFound(vm.name().to_string()))?;

        // Modify memory settings - use as_mb() to get the raw value
        let memory_mb = settings.memory.as_mb();
        mem_settings.put_u64("VirtualQuantity", memory_mb)?;
        mem_settings.put_u64("Reservation", memory_mb)?;
        mem_settings.put_u64("Limit", memory_mb)?;

        if settings.dynamic_memory {
            mem_settings.put_bool("DynamicMemoryEnabled", true)?;
            if let Some(ref min) = settings.dynamic_memory_min {
                mem_settings.put_u64("Reservation", min.as_mb())?;
            }
            if let Some(ref max) = settings.dynamic_memory_max {
                mem_settings.put_u64("Limit", max.as_mb())?;
            }
            if let Some(ref buffer) = settings.memory_buffer_percentage {
                mem_settings.put_u32("TargetMemoryBuffer", buffer.get())?;
            }
        }

        let mem_text = mem_settings.get_text()?;

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        let in_params = self.connection.get_method_params(
            "Msvm_VirtualSystemManagementService",
            "ModifyResourceSettings",
        )?;
        in_params.put_string_array("ResourceSettings", &[&mem_text])?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "ModifyResourceSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "ModifyResourceSettings")
    }

    fn configure_processor(&self, vm: &VirtualMachine, settings: &VmSettings) -> Result<()> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        // Find processor settings
        let proc_query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_ProcessorSettingData",
            settings_path
        );
        let proc_settings = self
            .connection
            .query_first(&proc_query)?
            .ok_or_else(|| Error::VmNotFound(vm.name().to_string()))?;

        // Use .get() to extract the raw processor count value
        proc_settings.put_u32("VirtualQuantity", settings.processor_count.get())?;

        if settings.nested_virtualization {
            proc_settings.put_bool("ExposeVirtualizationExtensions", true)?;
        }

        let proc_text = proc_settings.get_text()?;

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        let in_params = self.connection.get_method_params(
            "Msvm_VirtualSystemManagementService",
            "ModifyResourceSettings",
        )?;
        in_params.put_string_array("ResourceSettings", &[&proc_text])?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "ModifyResourceSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "ModifyResourceSettings")
    }

    fn configure_optional_settings(
        &self,
        vm: &VirtualMachine,
        settings: &VmSettings,
    ) -> Result<()> {
        use crate::vm::{AutomaticStartAction, AutomaticStopAction};

        // Check if any optional settings need to be applied
        let has_notes = settings.notes.is_some();
        let has_paths = settings.config_path.is_some()
            || settings.snapshot_path.is_some()
            || settings.smart_paging_path.is_some();
        let has_auto_actions = settings.automatic_start_action != AutomaticStartAction::Nothing
            || settings.automatic_stop_action != AutomaticStopAction::default()
            || !settings.automatic_start_delay.is_none();

        // Skip if nothing to configure
        if !has_notes && !has_paths && !has_auto_actions {
            return Ok(());
        }

        let vm_settings = self.get_vm_settings(vm)?;

        // Set notes
        if let Some(ref notes) = settings.notes {
            vm_settings.put_string("Notes", notes)?;
        }

        // Set paths
        if let Some(ref config_path) = settings.config_path {
            vm_settings.put_string("ConfigurationDataRoot", config_path)?;
        }
        if let Some(ref snapshot_path) = settings.snapshot_path {
            vm_settings.put_string("SnapshotDataRoot", snapshot_path)?;
        }
        if let Some(ref paging_path) = settings.smart_paging_path {
            vm_settings.put_string("SwapFileDataRoot", paging_path)?;
        }

        // Set automatic action properties
        if has_auto_actions {
            vm_settings.put_u16(
                "AutomaticStartupAction",
                settings.automatic_start_action.to_value(),
            )?;
            vm_settings.put_u16(
                "AutomaticShutdownAction",
                settings.automatic_stop_action.to_value(),
            )?;
            if !settings.automatic_start_delay.is_none() {
                vm_settings.put_string(
                    "AutomaticStartupActionDelay",
                    &settings.automatic_start_delay.to_cim_interval(),
                )?;
            }
        }

        let settings_text = vm_settings.get_text()?;

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        let in_params = self.connection.get_method_params(
            "Msvm_VirtualSystemManagementService",
            "ModifySystemSettings",
        )?;
        in_params.put_string("SystemSettings", &settings_text)?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "ModifySystemSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "ModifySystemSettings")
    }

    fn configure_secure_boot(&self, vm: &VirtualMachine, settings: &VmSettings) -> Result<()> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        // Find security settings
        let sec_query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_SecuritySettingData",
            settings_path
        );

        if let Some(sec_settings) = self.connection.query_first(&sec_query)? {
            sec_settings.put_bool("SecureBootEnabled", settings.secure_boot)?;

            if let Some(ref template) = settings.secure_boot_template {
                sec_settings.put_string("SecureBootTemplateId", template)?;
            }

            if settings.tpm_enabled {
                sec_settings.put_bool("TpmEnabled", true)?;
            }

            let sec_text = sec_settings.get_text()?;

            let mgmt_service = self.get_management_service()?;
            let mgmt_path = mgmt_service.get_path()?;

            let in_params = self.connection.get_method_params(
                "Msvm_VirtualSystemManagementService",
                "ModifySecuritySettings",
            )?;
            in_params.put_string("SecuritySettingData", &sec_text)?;

            let out_params = self.connection.exec_method(
                &mgmt_path,
                "ModifySecuritySettings",
                Some(&in_params),
            )?;
            self.handle_job_result(&out_params, "ModifySecuritySettings")?;
        }

        Ok(())
    }

    fn add_scsi_controller(&self, vm: &VirtualMachine) -> Result<()> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        let controller = self
            .connection
            .spawn_instance("Msvm_ResourceAllocationSettingData")?;
        controller.put_string("ResourceType", "6")?; // SCSI Controller
        controller.put_string(
            "ResourceSubType",
            "Microsoft:Hyper-V:Synthetic SCSI Controller",
        )?;

        let controller_text = controller.get_text()?;

        let mgmt_service = self.get_management_service()?;
        let mgmt_path = mgmt_service.get_path()?;

        let in_params = self
            .connection
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params.put_string("AffectedConfiguration", &settings_path)?;
        in_params.put_string_array("ResourceSettings", &[&controller_text])?;

        let out_params =
            self.connection
                .exec_method(&mgmt_path, "AddResourceSettings", Some(&in_params))?;
        self.handle_job_result(&out_params, "AddResourceSettings")
    }

    fn find_or_create_controller(
        &self,
        vm: &VirtualMachine,
        controller_type: ControllerType,
        controller_number: u32,
    ) -> Result<IWbemClassObject> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        let resource_subtype = controller_type.resource_subtype();
        let query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_ResourceAllocationSettingData",
            settings_path
        );

        let controllers = self.connection.query(&query)?;
        for controller in &controllers {
            if let Some(subtype) = controller.get_string_prop("ResourceSubType")? {
                if subtype.contains(resource_subtype) {
                    if let Some(addr) = controller.get_string_prop("Address")? {
                        if addr.parse::<u32>().unwrap_or(0) == controller_number {
                            return Ok(controller.clone());
                        }
                    }
                }
            }
        }

        // Controller not found, create it (for SCSI)
        if controller_type == ControllerType::Scsi {
            self.add_scsi_controller(vm)?;
            // Re-query
            let controllers = self.connection.query(&query)?;
            for controller in controllers {
                if let Some(subtype) = controller.get_string_prop("ResourceSubType")? {
                    if subtype.contains(resource_subtype) {
                        return Ok(controller);
                    }
                }
            }
        }

        Err(Error::OperationFailed {
            operation: "FindController",
            return_value: 0,
            message: format!(
                "Controller {:?} #{} not found",
                controller_type, controller_number
            ),
        })
    }

    fn create_disk_drive(&self, controller_path: &str, location: u32) -> Result<IWbemClassObject> {
        let drive = self
            .connection
            .spawn_instance("Msvm_ResourceAllocationSettingData")?;
        drive.put_string("ResourceType", "22")?; // Disk Drive
        drive.put_string("ResourceSubType", "Microsoft:Hyper-V:Synthetic Disk Drive")?;
        drive.put_string("Parent", controller_path)?;
        drive.put_u32("AddressOnParent", location)?;
        Ok(drive)
    }

    fn create_dvd_drive(&self, controller_path: &str, location: u32) -> Result<IWbemClassObject> {
        let drive = self
            .connection
            .spawn_instance("Msvm_ResourceAllocationSettingData")?;
        drive.put_string("ResourceType", "16")?; // DVD Drive
        drive.put_string("ResourceSubType", "Microsoft:Hyper-V:Synthetic DVD Drive")?;
        drive.put_string("Parent", controller_path)?;
        drive.put_u32("AddressOnParent", location)?;
        Ok(drive)
    }

    fn find_disk_drive(
        &self,
        vm: &VirtualMachine,
        controller_path: &str,
        location: u32,
    ) -> Result<IWbemClassObject> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        let query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_ResourceAllocationSettingData",
            settings_path
        );
        let resources = self.connection.query(&query)?;

        for resource in resources {
            if let Some(subtype) = resource.get_string_prop("ResourceSubType")? {
                if subtype.contains("Disk Drive") {
                    if let Some(parent) = resource.get_string_prop("Parent")? {
                        if parent == controller_path {
                            if let Some(addr) = resource.get_u32("AddressOnParent")? {
                                if addr == location {
                                    return Ok(resource);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(Error::OperationFailed {
            operation: "FindDiskDrive",
            return_value: 0,
            message: format!("Disk drive at location {} not found", location),
        })
    }

    fn find_dvd_drive(
        &self,
        vm: &VirtualMachine,
        controller_path: &str,
        location: u32,
    ) -> Result<IWbemClassObject> {
        let vm_settings = self.get_vm_settings(vm)?;
        let settings_path = vm_settings.get_path()?;

        let query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_ResourceAllocationSettingData",
            settings_path
        );
        let resources = self.connection.query(&query)?;

        for resource in resources {
            if let Some(subtype) = resource.get_string_prop("ResourceSubType")? {
                if subtype.contains("DVD Drive") {
                    if let Some(parent) = resource.get_string_prop("Parent")? {
                        if parent == controller_path {
                            if let Some(addr) = resource.get_u32("AddressOnParent")? {
                                if addr == location {
                                    return Ok(resource);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err(Error::OperationFailed {
            operation: "FindDvdDrive",
            return_value: 0,
            message: format!("DVD drive at location {} not found", location),
        })
    }

    fn handle_job_result(
        &self,
        out_params: &IWbemClassObject,
        operation: &'static str,
    ) -> Result<()> {
        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);

        match return_value {
            0 => Ok(()),
            4096 => {
                // Job started - wait for completion
                if let Some(job_path) = out_params.get_string_prop("Job")? {
                    self.wait_for_job(&job_path, operation)
                } else {
                    Ok(())
                }
            }
            code => Err(Error::OperationFailed {
                operation,
                return_value: code,
                message: format!("{} failed", operation),
            }),
        }
    }

    fn wait_for_job(&self, job_path: &str, operation: &'static str) -> Result<()> {
        loop {
            let job = self.connection.get_object(job_path)?;
            let job_state = job.get_u16("JobState")?.unwrap_or(0);

            match job_state {
                7 => return Ok(()), // Completed
                8 | 9 | 10 | 11 => {
                    // Terminated, Killed, Exception, Service
                    let error_code = job.get_u32("ErrorCode")?.unwrap_or(0);
                    let error_desc = job.get_string_prop("ErrorDescription")?.unwrap_or_default();
                    return Err(Error::JobFailed {
                        operation,
                        error_code,
                        error_description: error_desc,
                    });
                }
                _ => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
}
