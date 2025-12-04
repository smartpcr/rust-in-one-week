use crate::error::{Error, Result};
use crate::vm::{Generation, VmState, RequestedState, ShutdownType};
use crate::wmi::{WbemClassObjectExt, WmiConnection};
use windows::Win32::System::Wmi::IWbemClassObject;

/// Represents a Hyper-V virtual machine (Msvm_ComputerSystem).
#[derive(Debug)]
pub struct VirtualMachine {
    /// VM display name (ElementName).
    name: String,
    /// VM unique identifier (Name - GUID format).
    id: String,
    /// Current enabled state.
    state: VmState,
    /// VM generation.
    generation: Generation,
    /// WMI object path for method invocation.
    path: String,
    /// Reference to WMI connection.
    connection: std::sync::Arc<WmiConnection>,
}

impl VirtualMachine {
    /// Create from WMI object.
    pub(crate) fn from_wmi(
        obj: &IWbemClassObject,
        connection: std::sync::Arc<WmiConnection>,
    ) -> Result<Self> {
        let name = obj.get_string_prop_required("ElementName")?;
        let id = obj.get_string_prop_required("Name")?;
        let enabled_state = obj.get_u16("EnabledState")?.unwrap_or(0);
        let path = obj.get_path()?;

        // Get generation from associated settings
        let generation = Self::query_generation(&connection, &id)?;

        Ok(Self {
            name,
            id,
            state: VmState::from_enabled_state(enabled_state),
            generation,
            path,
            connection,
        })
    }

    /// Get VM display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get VM unique identifier (GUID).
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get current VM state.
    pub fn state(&self) -> VmState {
        self.state
    }

    /// Get VM generation.
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Refresh state from WMI.
    pub fn refresh(&mut self) -> Result<()> {
        let obj = self.connection.get_object(&self.path)?;
        self.state = VmState::from_enabled_state(obj.get_u16("EnabledState")?.unwrap_or(0));
        Ok(())
    }

    /// Start the VM.
    pub fn start(&mut self) -> Result<()> {
        if !self.state.can_start() {
            return Err(Error::InvalidState {
                vm_name: self.name.clone(),
                current: self.state.to_error(),
                operation: "start",
            });
        }
        self.request_state_change(RequestedState::Running)?;
        self.refresh()
    }

    /// Stop the VM.
    pub fn stop(&mut self, shutdown_type: ShutdownType) -> Result<()> {
        if !self.state.can_stop() {
            return Err(Error::InvalidState {
                vm_name: self.name.clone(),
                current: self.state.to_error(),
                operation: "stop",
            });
        }

        match shutdown_type {
            ShutdownType::Force => {
                self.request_state_change(RequestedState::Off)?;
            }
            ShutdownType::Graceful => {
                self.graceful_shutdown()?;
            }
            ShutdownType::GracefulWithForce => {
                if self.graceful_shutdown().is_err() {
                    self.request_state_change(RequestedState::Off)?;
                }
            }
        }
        self.refresh()
    }

    /// Pause the VM.
    pub fn pause(&mut self) -> Result<()> {
        if !self.state.can_pause() {
            return Err(Error::InvalidState {
                vm_name: self.name.clone(),
                current: self.state.to_error(),
                operation: "pause",
            });
        }
        self.request_state_change(RequestedState::Paused)?;
        self.refresh()
    }

    /// Resume a paused VM.
    pub fn resume(&mut self) -> Result<()> {
        if self.state != VmState::Paused {
            return Err(Error::InvalidState {
                vm_name: self.name.clone(),
                current: self.state.to_error(),
                operation: "resume",
            });
        }
        self.request_state_change(RequestedState::Running)?;
        self.refresh()
    }

    /// Save the VM state (suspend/hibernate).
    pub fn save(&mut self) -> Result<()> {
        if !self.state.can_save() {
            return Err(Error::InvalidState {
                vm_name: self.name.clone(),
                current: self.state.to_error(),
                operation: "save",
            });
        }
        self.request_state_change(RequestedState::Saved)?;
        self.refresh()
    }

    /// Reset the VM (hard restart).
    pub fn reset(&mut self) -> Result<()> {
        if self.state != VmState::Running {
            return Err(Error::InvalidState {
                vm_name: self.name.clone(),
                current: self.state.to_error(),
                operation: "reset",
            });
        }
        self.request_state_change(RequestedState::Reset)?;
        self.refresh()
    }

    /// Get memory size in MB.
    pub fn memory_mb(&self) -> Result<u64> {
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.CreationClassName='Msvm_ComputerSystem',Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState ResultClass=Msvm_VirtualSystemSettingData",
            self.id
        );
        let settings = self.connection.query_first(&query)?
            .ok_or_else(|| Error::VmNotFound(self.name.clone()))?;

        let settings_path = settings.get_path()?;
        let mem_query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_MemorySettingData",
            settings_path
        );
        let mem_settings = self.connection.query_first(&mem_query)?
            .ok_or_else(|| Error::VmNotFound(self.name.clone()))?;

        mem_settings.get_u64("VirtualQuantity")?.ok_or_else(|| Error::TypeConversion {
            property: "VirtualQuantity",
            expected: "u64",
        })
    }

    /// Get processor count.
    pub fn processor_count(&self) -> Result<u32> {
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.CreationClassName='Msvm_ComputerSystem',Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState ResultClass=Msvm_VirtualSystemSettingData",
            self.id
        );
        let settings = self.connection.query_first(&query)?
            .ok_or_else(|| Error::VmNotFound(self.name.clone()))?;

        let settings_path = settings.get_path()?;
        let proc_query = format!(
            "ASSOCIATORS OF {{{}}} WHERE ResultClass=Msvm_ProcessorSettingData",
            settings_path
        );
        let proc_settings = self.connection.query_first(&proc_query)?
            .ok_or_else(|| Error::VmNotFound(self.name.clone()))?;

        proc_settings.get_u32("VirtualQuantity")?.ok_or_else(|| Error::TypeConversion {
            property: "VirtualQuantity",
            expected: "u32",
        })
    }

    /// Request state change via WMI.
    fn request_state_change(&self, requested: RequestedState) -> Result<()> {
        let in_params = self.connection.get_method_params("Msvm_ComputerSystem", "RequestStateChange")?;
        in_params.put_u16("RequestedState", requested as u16)?;

        let out_params = self.connection.exec_method(&self.path, "RequestStateChange", Some(&in_params))?;
        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);

        match return_value {
            0 => Ok(()), // Completed
            4096 => {
                // Job started - wait for completion
                let job_path: std::string::String = out_params.get_string_prop("Job")
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                if !job_path.is_empty() {
                    self.wait_for_job(&job_path)
                } else {
                    Ok(())
                }
            }
            code => Err(Error::OperationFailed {
                operation: "RequestStateChange",
                return_value: code,
                message: format!("State change to {:?} failed", requested),
            }),
        }
    }

    /// Attempt graceful shutdown via guest integration services.
    fn graceful_shutdown(&self) -> Result<()> {
        // Find ShutdownComponent for this VM
        let query = format!(
            "SELECT * FROM Msvm_ShutdownComponent WHERE SystemName='{}'",
            self.id
        );
        let shutdown_component = self.connection.query_first(&query)?
            .ok_or_else(|| Error::OperationFailed {
                operation: "GracefulShutdown",
                return_value: 0,
                message: "Shutdown integration service not available".to_string(),
            })?;

        let component_path = shutdown_component.get_path()?;
        let in_params = self.connection.get_method_params("Msvm_ShutdownComponent", "InitiateShutdown")?;
        in_params.put_bool("Force", false)?;
        in_params.put_string("Reason", "User requested shutdown")?;

        let out_params = self.connection.exec_method(&component_path, "InitiateShutdown", Some(&in_params))?;
        let return_value = out_params.get_u32("ReturnValue")?.unwrap_or(0);

        if return_value == 0 {
            Ok(())
        } else {
            Err(Error::OperationFailed {
                operation: "InitiateShutdown",
                return_value,
                message: "Graceful shutdown failed".to_string(),
            })
        }
    }

    /// Wait for an async WMI job to complete.
    fn wait_for_job(&self, job_path: &str) -> Result<()> {
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
                        operation: "Job",
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

    /// Query VM generation from settings.
    fn query_generation(connection: &WmiConnection, vm_id: &str) -> Result<Generation> {
        let query = format!(
            "ASSOCIATORS OF {{Msvm_ComputerSystem.CreationClassName='Msvm_ComputerSystem',Name='{}'}} \
             WHERE AssocClass=Msvm_SettingsDefineState ResultClass=Msvm_VirtualSystemSettingData",
            vm_id
        );
        if let Some(settings) = connection.query_first(&query)? {
            let subtype: std::string::String = settings.get_string_prop("VirtualSystemSubType")
                .ok()
                .flatten()
                .unwrap_or_default();
            if !subtype.is_empty() {
                return Ok(Generation::from_subtype(&subtype));
            }
        }
        Ok(Generation::Gen1)
    }
}
