//! Virtual Machine management using WMI
//!
//! Uses WMI Msvm_* classes for VM operations.

use crate::error::{HvError, Result};
use crate::wmi::msvm::MsvmVm;
use crate::wmi::{hyperv::EnabledState, operations as wmi_ops, WmiConnection};
use serde::{Deserialize, Serialize};

/// VM state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Unknown,
    Running,
    Off,
    Stopping,
    Saved,
    Paused,
    Starting,
    Saving,
    Pausing,
    Resuming,
}

impl VmState {
    /// Parse state from WMI EnabledState
    pub fn from_enabled_state(state: EnabledState) -> Self {
        match state {
            EnabledState::Enabled => VmState::Running,
            EnabledState::Disabled => VmState::Off,
            EnabledState::Stopping => VmState::Stopping,
            EnabledState::Suspended => VmState::Saved,
            EnabledState::Paused => VmState::Paused,
            EnabledState::Starting | EnabledState::Starting2 => VmState::Starting,
            EnabledState::Saving => VmState::Saving,
            EnabledState::Pausing => VmState::Pausing,
            EnabledState::Resuming => VmState::Resuming,
            EnabledState::ShuttingDown => VmState::Stopping,
            _ => VmState::Unknown,
        }
    }

    /// Parse state from HCS state string (for backward compatibility)
    pub fn from_hcs_state(state: &str) -> Self {
        match state.to_lowercase().as_str() {
            "running" => VmState::Running,
            "stopped" | "off" => VmState::Off,
            "stopping" => VmState::Stopping,
            "saved" | "savedstate" => VmState::Saved,
            "paused" | "suspended" => VmState::Paused,
            "starting" => VmState::Starting,
            "saving" => VmState::Saving,
            "pausing" | "suspending" => VmState::Pausing,
            "resuming" => VmState::Resuming,
            _ => VmState::Unknown,
        }
    }

    /// Returns true if the VM is running
    pub fn is_running(&self) -> bool {
        matches!(self, VmState::Running)
    }

    /// Returns true if the VM is off
    pub fn is_off(&self) -> bool {
        matches!(self, VmState::Off)
    }

    /// Returns true if the VM is in a transitional state
    pub fn is_transitioning(&self) -> bool {
        matches!(
            self,
            VmState::Starting
                | VmState::Stopping
                | VmState::Saving
                | VmState::Pausing
                | VmState::Resuming
        )
    }
}

/// VM generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmGeneration {
    Gen1,
    Gen2,
}

/// Properties returned from HCS (kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VmProperties {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub memory: Option<VmMemoryProperties>,
    #[serde(default)]
    pub processor: Option<VmProcessorProperties>,
    #[serde(default)]
    pub runtime_state_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VmMemoryProperties {
    #[serde(default)]
    pub virtual_machine_memory: Option<u64>,
    #[serde(default)]
    pub available_memory: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VmProcessorProperties {
    #[serde(default)]
    pub count: Option<u32>,
}

/// Represents a Hyper-V virtual machine managed via WMI
pub struct Vm {
    id: String,
    name: String,
    current_state: VmState,
    memory_mb: Option<u64>,
    processor_count: Option<u32>,
    generation: Option<u32>,
}

impl Vm {
    /// Create a new VM handle from WMI info
    pub(crate) fn from_wmi(vm: &MsvmVm) -> Self {
        Vm {
            id: vm.id.clone(),
            name: vm.name.clone(),
            current_state: VmState::from_enabled_state(vm.enabled_state),
            memory_mb: vm.memory_mb,
            processor_count: vm.processor_count,
            generation: vm.generation,
        }
    }

    /// Returns the VM name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the VM ID (GUID)
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Gets the current state of the VM
    pub fn state(&mut self) -> Result<VmState> {
        // Refresh state from WMI
        let conn = WmiConnection::connect_hyperv()?;
        let wmi_vm = wmi_ops::get_vm_by_name(&conn, &self.name)?;
        self.current_state = VmState::from_enabled_state(wmi_vm.enabled_state);
        Ok(self.current_state)
    }

    /// Gets the cached state without refreshing
    pub fn cached_state(&self) -> VmState {
        self.current_state
    }

    /// Starts the VM
    pub fn start(&mut self) -> Result<()> {
        let state = self.state()?;
        if state.is_running() {
            return Ok(());
        }
        if !state.is_off() && state != VmState::Saved && state != VmState::Paused {
            return Err(HvError::InvalidState(format!(
                "VM must be Off, Saved, or Paused to start (current: {:?})",
                state
            )));
        }

        let conn = WmiConnection::connect_hyperv()?;
        wmi_ops::start_vm(&conn, &self.name)?;
        self.current_state = VmState::Starting;

        Ok(())
    }

    /// Stops the VM gracefully (sends shutdown signal)
    pub fn stop(&mut self) -> Result<()> {
        let state = self.state()?;
        if state.is_off() {
            return Ok(());
        }

        let conn = WmiConnection::connect_hyperv()?;
        wmi_ops::shutdown_vm(&conn, &self.name)?;
        self.current_state = VmState::Stopping;

        Ok(())
    }

    /// Forces the VM to power off immediately
    pub fn force_stop(&mut self) -> Result<()> {
        let conn = WmiConnection::connect_hyperv()?;
        wmi_ops::stop_vm(&conn, &self.name)?;
        self.current_state = VmState::Off;
        Ok(())
    }

    /// Pauses the VM
    pub fn pause(&mut self) -> Result<()> {
        let state = self.state()?;
        if state == VmState::Paused {
            return Ok(());
        }
        if !state.is_running() {
            return Err(HvError::InvalidState(format!(
                "VM must be Running to pause (current: {:?})",
                state
            )));
        }

        let conn = WmiConnection::connect_hyperv()?;
        wmi_ops::pause_vm(&conn, &self.name)?;
        self.current_state = VmState::Pausing;

        Ok(())
    }

    /// Resumes a paused VM
    pub fn resume(&mut self) -> Result<()> {
        let state = self.state()?;
        if state.is_running() {
            return Ok(());
        }
        if state != VmState::Paused && state != VmState::Saved {
            return Err(HvError::InvalidState(format!(
                "VM must be Paused or Saved to resume (current: {:?})",
                state
            )));
        }

        let conn = WmiConnection::connect_hyperv()?;
        wmi_ops::resume_vm(&conn, &self.name)?;
        self.current_state = VmState::Resuming;

        Ok(())
    }

    /// Saves the VM state (hibernate)
    pub fn save(&mut self) -> Result<()> {
        let state = self.state()?;
        if state == VmState::Saved {
            return Ok(());
        }
        if !state.is_running() {
            return Err(HvError::InvalidState(format!(
                "VM must be Running to save (current: {:?})",
                state
            )));
        }

        let conn = WmiConnection::connect_hyperv()?;
        wmi_ops::save_vm(&conn, &self.name)?;
        self.current_state = VmState::Saving;

        Ok(())
    }

    /// Gets the number of virtual CPUs
    pub fn cpu_count(&mut self) -> Result<u32> {
        if let Some(count) = self.processor_count {
            return Ok(count);
        }

        // Refresh from WMI
        let conn = WmiConnection::connect_hyperv()?;
        let proc = wmi_ops::get_vm_processor_settings(&conn, &self.id)?;
        self.processor_count = Some(proc.virtual_quantity);
        Ok(proc.virtual_quantity)
    }

    /// Gets the memory size in MB
    pub fn memory_mb(&mut self) -> Result<u64> {
        if let Some(mem) = self.memory_mb {
            return Ok(mem);
        }

        // Refresh from WMI
        let conn = WmiConnection::connect_hyperv()?;
        let mem = wmi_ops::get_vm_memory_settings(&conn, &self.id)?;
        self.memory_mb = Some(mem.virtual_quantity_mb);
        Ok(mem.virtual_quantity_mb)
    }

    /// Gets the VM generation
    pub fn generation(&self) -> Option<VmGeneration> {
        self.generation.map(|g| {
            if g == 2 {
                VmGeneration::Gen2
            } else {
                VmGeneration::Gen1
            }
        })
    }
}

impl std::fmt::Debug for Vm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vm")
            .field("name", &self.name)
            .field("id", &self.id)
            .field("state", &self.current_state)
            .field("memory_mb", &self.memory_mb)
            .field("cpu_count", &self.processor_count)
            .field("generation", &self.generation)
            .finish()
    }
}
