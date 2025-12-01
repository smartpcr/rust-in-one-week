//! Virtual Machine management using HCS (Host Compute Service)
//!
//! Uses windows-rs HCS bindings directly for VM operations.

use crate::error::{HvError, Result};
use crate::hcs::{self, ComputeSystemInfo, HcsSystem, VmConfiguration, DEFAULT_TIMEOUT};
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
    /// Parse state from HCS state string
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

/// Properties returned from HCS
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

/// Represents a Hyper-V virtual machine managed via HCS
pub struct Vm {
    id: String,
    name: String,
    system: Option<HcsSystem>,
}

impl Vm {
    /// Create a new VM handle from enumeration info
    pub(crate) fn from_info(info: &ComputeSystemInfo) -> Self {
        Vm {
            id: info.id.clone(),
            name: info.name.clone().unwrap_or_else(|| info.id.clone()),
            system: None,
        }
    }

    /// Create a new VM handle with an open HCS system
    pub(crate) fn from_system(id: String, name: String, system: HcsSystem) -> Self {
        Vm {
            id,
            name,
            system: Some(system),
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

    /// Open the HCS system handle if not already open
    fn ensure_open(&mut self) -> Result<&HcsSystem> {
        if self.system.is_none() {
            let system = hcs::open_compute_system(&self.id)?;
            self.system = Some(system);
        }
        Ok(self.system.as_ref().unwrap())
    }

    /// Gets the current state of the VM
    pub fn state(&mut self) -> Result<VmState> {
        let system = self.ensure_open()?;

        let properties = system.get_properties(None)?;

        if let Some(props_json) = properties {
            let props: VmProperties = serde_json::from_str(&props_json)?;
            if let Some(state_str) = props.state {
                return Ok(VmState::from_hcs_state(&state_str));
            }
        }

        Ok(VmState::Unknown)
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

        let system = self.ensure_open()?;
        system.start(None)?;

        Ok(())
    }

    /// Stops the VM gracefully (sends shutdown signal)
    pub fn stop(&mut self) -> Result<()> {
        let state = self.state()?;
        if state.is_off() {
            return Ok(());
        }

        let system = self.ensure_open()?;

        // Try graceful shutdown first
        let options = r#"{"Type": "GracefulShutdown"}"#;
        if system.shutdown(Some(options)).is_err() {
            // Fall back to hard shutdown
            system.shutdown(None)?;
        }

        Ok(())
    }

    /// Forces the VM to power off immediately
    pub fn force_stop(&mut self) -> Result<()> {
        let system = self.ensure_open()?;
        system.terminate()?;
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

        let system = self.ensure_open()?;
        system.pause(None)?;

        Ok(())
    }

    /// Resumes a paused VM
    pub fn resume(&mut self) -> Result<()> {
        let state = self.state()?;
        if state.is_running() {
            return Ok(());
        }
        if state != VmState::Paused {
            return Err(HvError::InvalidState(format!(
                "VM must be Paused to resume (current: {:?})",
                state
            )));
        }

        let system = self.ensure_open()?;
        system.resume()?;

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

        let system = self.ensure_open()?;
        system.save(None)?;

        Ok(())
    }

    /// Gets the number of virtual CPUs
    pub fn cpu_count(&mut self) -> Result<u32> {
        let system = self.ensure_open()?;

        let query = r#"{"PropertyTypes": ["Processor"]}"#;
        let properties = system.get_properties(Some(query))?;

        if let Some(props_json) = properties {
            let props: VmProperties = serde_json::from_str(&props_json)?;
            if let Some(proc) = props.processor {
                return Ok(proc.count.unwrap_or(1));
            }
        }

        Ok(1)
    }

    /// Gets the memory size in MB
    pub fn memory_mb(&mut self) -> Result<u64> {
        let system = self.ensure_open()?;

        let query = r#"{"PropertyTypes": ["Memory"]}"#;
        let properties = system.get_properties(Some(query))?;

        if let Some(props_json) = properties {
            let props: VmProperties = serde_json::from_str(&props_json)?;
            if let Some(mem) = props.memory {
                if let Some(size) = mem.virtual_machine_memory {
                    return Ok(size / (1024 * 1024)); // Convert to MB
                }
            }
        }

        Ok(0)
    }

    /// Closes the HCS system handle
    pub fn close(&mut self) {
        self.system = None;
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        self.close();
    }
}

impl std::fmt::Debug for Vm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vm")
            .field("name", &self.name)
            .field("id", &self.id)
            .field("has_handle", &self.system.is_some())
            .finish()
    }
}
