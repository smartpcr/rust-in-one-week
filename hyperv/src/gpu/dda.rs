//! DDA (Discrete Device Assignment) operations for Hyper-V.
//!
//! DDA allows assigning a complete PCI device (typically a GPU) to a VM for
//! exclusive use, providing near-native performance.

use super::types::{DdaDevice, DdaDeviceSettings, DdaDeviceStatus};
use crate::error::{Error, FailureType, Result};

#[cfg(windows)]
use crate::wmi::{JobWaiter, WbemClassObjectExt, WmiConnection};
#[cfg(windows)]
use std::time::Duration;

/// DDA device manager for host-level device operations.
pub struct DdaManager<'a> {
    #[cfg(windows)]
    conn: &'a WmiConnection,
    #[cfg(not(windows))]
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> DdaManager<'a> {
    /// Create a new DDA manager.
    #[cfg(windows)]
    pub fn new(conn: &'a WmiConnection) -> Self {
        Self { conn }
    }

    /// Create a new DDA manager (non-Windows stub).
    #[cfg(not(windows))]
    pub fn new(_conn: &'a ()) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// List all DDA-capable devices on the host.
    #[cfg(windows)]
    pub fn list_devices(&self) -> Result<Vec<DdaDevice>> {
        let query = "SELECT * FROM Msvm_PciExpress";
        let results = self.conn.query(query)?;

        let mut devices = Vec::new();
        for obj in results {
            if let Ok(device) = self.parse_dda_device(&obj) {
                devices.push(device);
            }
        }
        Ok(devices)
    }

    /// List all DDA-capable devices on the host (non-Windows stub).
    #[cfg(not(windows))]
    pub fn list_devices(&self) -> Result<Vec<DdaDevice>> {
        Ok(Vec::new())
    }

    /// List available (unassigned) DDA devices.
    #[cfg(windows)]
    pub fn list_available_devices(&self) -> Result<Vec<DdaDevice>> {
        let all_devices = self.list_devices()?;
        Ok(all_devices.into_iter().filter(|d| d.is_available()).collect())
    }

    /// List available DDA devices (non-Windows stub).
    #[cfg(not(windows))]
    pub fn list_available_devices(&self) -> Result<Vec<DdaDevice>> {
        Ok(Vec::new())
    }

    /// Get a specific DDA device by location path.
    #[cfg(windows)]
    pub fn get_device(&self, location_path: &str) -> Result<DdaDevice> {
        let query = format!(
            "SELECT * FROM Msvm_PciExpress WHERE DeviceInstancePath LIKE '%{}%'",
            location_path.replace('\'', "''")
        );
        let result = self.conn.query_first(&query)?;

        match result {
            Some(obj) => self.parse_dda_device(&obj),
            None => Err(Error::OperationFailed {
                failure_type: FailureType::Permanent,
                operation: "GetDdaDevice",
                return_value: 0,
                message: format!("DDA device not found: {}", location_path),
            }),
        }
    }

    /// Get a specific DDA device by location path (non-Windows stub).
    #[cfg(not(windows))]
    pub fn get_device(&self, location_path: &str) -> Result<DdaDevice> {
        Err(Error::OperationFailed {
            failure_type: FailureType::Permanent,
            operation: "GetDdaDevice",
            return_value: 0,
            message: format!("DDA device not found: {}", location_path),
        })
    }

    /// Check if a device is assignable via DDA.
    #[cfg(windows)]
    pub fn is_device_assignable(&self, location_path: &str) -> Result<bool> {
        // Get the Assignable Device Service
        let service = self.get_assignable_device_service()?;
        let service_path = service.get_path()?;

        // Call IsDeviceAssignable method
        let in_params = self.conn.get_method_params(
            "Msvm_AssignableDeviceService",
            "IsDeviceAssignable",
        )?;
        in_params.put_string("DeviceInstancePath", location_path)?;

        let out_params = self
            .conn
            .exec_method(&service_path, "IsDeviceAssignable", Some(&in_params))?;

        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(1);
        if return_value != 0 {
            return Ok(false);
        }

        out_params.get_bool("IsAssignable").map(|opt| opt.unwrap_or(false))
    }

    /// Check if a device is assignable via DDA (non-Windows stub).
    #[cfg(not(windows))]
    pub fn is_device_assignable(&self, _location_path: &str) -> Result<bool> {
        Ok(false)
    }

    /// Dismount a device from the host for DDA assignment.
    ///
    /// This prepares the device for assignment to a VM by unbinding it from
    /// the host driver.
    #[cfg(windows)]
    pub fn dismount_device(&self, location_path: &str) -> Result<()> {
        let service = self.get_assignable_device_service()?;
        let service_path = service.get_path()?;

        let in_params = self.conn.get_method_params(
            "Msvm_AssignableDeviceService",
            "DismountAssignableDevice",
        )?;
        in_params.put_string("DeviceInstancePath", location_path)?;

        let out_params = self
            .conn
            .exec_method(&service_path, "DismountAssignableDevice", Some(&in_params))?;

        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);
        match return_value {
            0 => Ok(()),
            4096 => {
                let job_path = out_params.get_string_prop("Job")?.ok_or_else(|| {
                    Error::operation_failed("DismountAssignableDevice", 4096, "No job path returned")
                })?;
                let waiter = JobWaiter::with_timeout(self.conn, Duration::from_secs(60));
                waiter.wait_for_job(&job_path, "DismountDdaDevice")?;
                Ok(())
            }
            code => Err(Error::operation_failed(
                "DismountAssignableDevice",
                code,
                format!("Failed to dismount device: {}", location_path),
            )),
        }
    }

    /// Dismount a device from the host (non-Windows stub).
    #[cfg(not(windows))]
    pub fn dismount_device(&self, _location_path: &str) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "DDA".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }

    /// Mount a device back to the host after DDA assignment.
    #[cfg(windows)]
    pub fn mount_device(&self, location_path: &str) -> Result<()> {
        let service = self.get_assignable_device_service()?;
        let service_path = service.get_path()?;

        let in_params = self.conn.get_method_params(
            "Msvm_AssignableDeviceService",
            "MountAssignableDevice",
        )?;
        in_params.put_string("DeviceInstancePath", location_path)?;

        let out_params = self
            .conn
            .exec_method(&service_path, "MountAssignableDevice", Some(&in_params))?;

        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);
        match return_value {
            0 => Ok(()),
            4096 => {
                let job_path = out_params.get_string_prop("Job")?.ok_or_else(|| {
                    Error::operation_failed("MountAssignableDevice", 4096, "No job path returned")
                })?;
                let waiter = JobWaiter::with_timeout(self.conn, Duration::from_secs(60));
                waiter.wait_for_job(&job_path, "MountDdaDevice")?;
                Ok(())
            }
            code => Err(Error::operation_failed(
                "MountAssignableDevice",
                code,
                format!("Failed to mount device: {}", location_path),
            )),
        }
    }

    /// Mount a device back to the host (non-Windows stub).
    #[cfg(not(windows))]
    pub fn mount_device(&self, _location_path: &str) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "DDA".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }

    #[cfg(windows)]
    fn get_assignable_device_service(
        &self,
    ) -> Result<windows::Win32::System::Wmi::IWbemClassObject> {
        self.conn.get_singleton("Msvm_AssignableDeviceService")
    }

    #[cfg(windows)]
    fn parse_dda_device(
        &self,
        obj: &windows::Win32::System::Wmi::IWbemClassObject,
    ) -> Result<DdaDevice> {
        let location_path = obj.get_string_prop("DeviceInstancePath")?.unwrap_or_default();
        let instance_id = obj.get_string_prop("InstanceID")?.unwrap_or_default();
        let friendly_name = obj
            .get_string_prop("ElementName")?
            .unwrap_or_else(|| location_path.clone());

        // Check if device is assigned
        let host_resource = obj.get_string_array("HostResource")?;
        let assigned_vm = if let Some(resources) = host_resource {
            if !resources.is_empty() {
                // Extract VM name from the resource path
                resources.first().cloned()
            } else {
                None
            }
        } else {
            None
        };

        let status = if assigned_vm.is_some() {
            DdaDeviceStatus::Assigned
        } else {
            DdaDeviceStatus::Available
        };

        Ok(DdaDevice {
            location_path,
            instance_id,
            friendly_name,
            device_class: obj
                .get_string_prop("DeviceClass")?
                .unwrap_or_else(|| "Unknown".to_string()),
            vendor: obj.get_string_prop("Vendor")?,
            status,
            assigned_vm,
            is_virtual_function: obj.get_bool("IsVirtualFunction")?.unwrap_or(false),
            mmio_space_required_low: obj.get_u64("MmioSpaceRequired")?.unwrap_or(0),
            mmio_space_required_high: obj.get_u64("MmioSpaceRequiredHigh")?.unwrap_or(0),
        })
    }
}

/// DDA operations for a specific VM.
pub struct VmDda<'a> {
    #[cfg(windows)]
    conn: &'a WmiConnection,
    #[cfg(not(windows))]
    _phantom: std::marker::PhantomData<&'a ()>,
    vm_id: String,
    vm_name: String,
}

impl<'a> VmDda<'a> {
    /// Create DDA operations for a VM.
    #[cfg(windows)]
    pub fn new(conn: &'a WmiConnection, vm_id: impl Into<String>, vm_name: impl Into<String>) -> Self {
        Self {
            conn,
            vm_id: vm_id.into(),
            vm_name: vm_name.into(),
        }
    }

    /// Create DDA operations for a VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn new(
        _conn: &'a (),
        vm_id: impl Into<String>,
        vm_name: impl Into<String>,
    ) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            vm_id: vm_id.into(),
            vm_name: vm_name.into(),
        }
    }

    /// Get the VM ID.
    pub fn vm_id(&self) -> &str {
        &self.vm_id
    }

    /// Get the VM name.
    pub fn vm_name(&self) -> &str {
        &self.vm_name
    }

    /// List DDA devices assigned to this VM.
    #[cfg(windows)]
    pub fn list_devices(&self) -> Result<Vec<DdaDevice>> {
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SystemDevice \
             ResultClass=Msvm_PciExpress",
            self.vm_id.replace('\'', "''")
        );
        let results = self.conn.query(&query)?;

        let manager = DdaManager::new(self.conn);
        let mut devices = Vec::new();
        for obj in results {
            if let Ok(device) = manager.parse_dda_device(&obj) {
                devices.push(device);
            }
        }
        Ok(devices)
    }

    /// List DDA devices assigned to this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn list_devices(&self) -> Result<Vec<DdaDevice>> {
        Ok(Vec::new())
    }

    /// Add a DDA device to this VM.
    #[cfg(windows)]
    pub fn add_device(&self, settings: &DdaDeviceSettings) -> Result<()> {
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

        // Get default PCI Express setting data
        let pci_default = self
            .conn
            .get_default_resource("Microsoft:Hyper-V:Pci Express")?;

        // Configure DDA device settings
        pci_default.put_string("HostResource", &settings.location_path)?;

        if settings.mmio_space_low > 0 {
            pci_default.put_u64("MmioSpaceReserved", settings.mmio_space_low)?;
        }
        if settings.mmio_space_high > 0 {
            pci_default.put_u64("MmioSpaceReservedHigh", settings.mmio_space_high)?;
        }

        // Set parent to VSSD
        pci_default.put_string("Parent", &vssd_path)?;

        let pci_text = pci_default.get_text()?;

        // Call AddResourceSettings
        let in_params = self
            .conn
            .get_method_params("Msvm_VirtualSystemManagementService", "AddResourceSettings")?;
        in_params.put_string("AffectedConfiguration", &vssd_path)?;
        in_params.put_string_array("ResourceSettings", &[&pci_text])?;

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
                waiter.wait_for_job(&job_path, "AddDdaDevice")?;
                Ok(())
            }
            code => Err(Error::operation_failed(
                "AddResourceSettings",
                code,
                "Failed to add DDA device",
            )),
        }
    }

    /// Add a DDA device to this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn add_device(&self, _settings: &DdaDeviceSettings) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "DDA".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }

    /// Remove a DDA device from this VM.
    #[cfg(windows)]
    pub fn remove_device(&self, location_path: &str) -> Result<()> {
        // Get VSMS
        let vsms = self.conn.get_singleton("Msvm_VirtualSystemManagementService")?;
        let vsms_path = vsms.get_path()?;

        // Find the PCI Express setting for this device
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.Name='{}'}} \
             WHERE AssocClass=Msvm_SystemDevice \
             ResultClass=Msvm_PciExpressSettingData",
            self.vm_id.replace('\'', "''")
        );
        let results = self.conn.query(&query)?;

        // Find the matching device
        let mut device_path = None;
        for obj in results {
            if let Ok(Some(host_resources)) = obj.get_string_array("HostResource") {
                if host_resources.iter().any(|r| r.contains(location_path)) {
                    device_path = Some(obj.get_path()?);
                    break;
                }
            }
        }

        let device_path = device_path.ok_or_else(|| Error::OperationFailed {
            failure_type: FailureType::Permanent,
            operation: "RemoveDdaDevice",
            return_value: 0,
            message: format!("DDA device not found on VM: {}", location_path),
        })?;

        // Call RemoveResourceSettings
        let in_params = self.conn.get_method_params(
            "Msvm_VirtualSystemManagementService",
            "RemoveResourceSettings",
        )?;
        in_params.put_string_array("ResourceSettings", &[&device_path])?;

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
                waiter.wait_for_job(&job_path, "RemoveDdaDevice")?;
                Ok(())
            }
            code => Err(Error::operation_failed(
                "RemoveResourceSettings",
                code,
                "Failed to remove DDA device",
            )),
        }
    }

    /// Remove a DDA device from this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn remove_device(&self, _location_path: &str) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "DDA".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }

    /// Remove all DDA devices from this VM.
    #[cfg(windows)]
    pub fn remove_all_devices(&self) -> Result<u32> {
        let devices = self.list_devices()?;
        let count = devices.len() as u32;

        for device in devices {
            self.remove_device(&device.location_path)?;
        }

        Ok(count)
    }

    /// Remove all DDA devices from this VM (non-Windows stub).
    #[cfg(not(windows))]
    pub fn remove_all_devices(&self) -> Result<u32> {
        Ok(0)
    }

    /// Configure VM MMIO space for DDA.
    ///
    /// Some DDA devices (especially high-end GPUs) require additional MMIO space.
    #[cfg(windows)]
    pub fn configure_mmio(&self, low_mmio_gap_mb: u64, high_mmio_gap_mb: u64) -> Result<()> {
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

        // Set MMIO gap sizes
        vssd.put_u64("LowMmioGapSize", low_mmio_gap_mb)?;
        vssd.put_u64("HighMmioGapSize", high_mmio_gap_mb)?;

        let vssd_text = vssd.get_text()?;

        // Call ModifySystemSettings
        let in_params = self.conn.get_method_params(
            "Msvm_VirtualSystemManagementService",
            "ModifySystemSettings",
        )?;
        in_params.put_string("SystemSettings", &vssd_text)?;

        let out_params = self
            .conn
            .exec_method(&vsms_path, "ModifySystemSettings", Some(&in_params))?;

        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);
        match return_value {
            0 => Ok(()),
            4096 => {
                let job_path = out_params.get_string_prop("Job")?.ok_or_else(|| {
                    Error::operation_failed("ModifySystemSettings", 4096, "No job path returned")
                })?;
                let waiter = JobWaiter::with_timeout(self.conn, Duration::from_secs(60));
                waiter.wait_for_job(&job_path, "ConfigureMmio")?;
                Ok(())
            }
            code => Err(Error::operation_failed(
                "ModifySystemSettings",
                code,
                "Failed to configure MMIO space",
            )),
        }
    }

    /// Configure VM MMIO space for DDA (non-Windows stub).
    #[cfg(not(windows))]
    pub fn configure_mmio(&self, _low_mmio_gap_mb: u64, _high_mmio_gap_mb: u64) -> Result<()> {
        Err(Error::FeatureNotAvailable {
            feature: "DDA".to_string(),
            reason: "Not available on this platform".to_string(),
        })
    }
}

/// Helper function to add a DDA device to a VM by name.
#[cfg(windows)]
pub fn add_dda_device_to_vm(
    conn: &WmiConnection,
    vm_name: &str,
    location_path: &str,
) -> Result<()> {
    // Get VM by name
    let query = format!(
        "SELECT * FROM Msvm_ComputerSystem WHERE ElementName = '{}' AND Caption = 'Virtual Machine'",
        vm_name.replace('\'', "''")
    );
    let vm_obj = conn.query_first(&query)?.ok_or_else(|| Error::VmNotFound(vm_name.to_string()))?;
    let vm_id = vm_obj.get_string_prop("Name")?.unwrap_or_default();

    let vm_dda = VmDda::new(conn, &vm_id, vm_name);
    let settings = DdaDeviceSettings::new(location_path);
    vm_dda.add_device(&settings)
}

/// Helper function to add a DDA device to a VM by name (non-Windows stub).
#[cfg(not(windows))]
pub fn add_dda_device_to_vm(
    _conn: &(),
    _vm_name: &str,
    _location_path: &str,
) -> Result<()> {
    Err(Error::FeatureNotAvailable {
        feature: "DDA".to_string(),
        reason: "Not available on this platform".to_string(),
    })
}

/// Helper function to remove a DDA device from a VM by name.
#[cfg(windows)]
pub fn remove_dda_device_from_vm(
    conn: &WmiConnection,
    vm_name: &str,
    location_path: &str,
) -> Result<()> {
    // Get VM by name
    let query = format!(
        "SELECT * FROM Msvm_ComputerSystem WHERE ElementName = '{}' AND Caption = 'Virtual Machine'",
        vm_name.replace('\'', "''")
    );
    let vm_obj = conn.query_first(&query)?.ok_or_else(|| Error::VmNotFound(vm_name.to_string()))?;
    let vm_id = vm_obj.get_string_prop("Name")?.unwrap_or_default();

    let vm_dda = VmDda::new(conn, &vm_id, vm_name);
    vm_dda.remove_device(location_path)
}

/// Helper function to remove a DDA device from a VM by name (non-Windows stub).
#[cfg(not(windows))]
pub fn remove_dda_device_from_vm(
    _conn: &(),
    _vm_name: &str,
    _location_path: &str,
) -> Result<()> {
    Err(Error::FeatureNotAvailable {
        feature: "DDA".to_string(),
        reason: "Not available on this platform".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dda_device_settings() {
        let settings = DdaDeviceSettings::new("PCIROOT(0)#PCI(0100)")
            .with_mmio(256 * 1024 * 1024, 8 * 1024 * 1024 * 1024)
            .with_vf_slot(0);

        assert_eq!(settings.location_path, "PCIROOT(0)#PCI(0100)");
        assert_eq!(settings.mmio_space_low, 256 * 1024 * 1024);
        assert_eq!(settings.virtual_function_slot, Some(0));
    }
}
