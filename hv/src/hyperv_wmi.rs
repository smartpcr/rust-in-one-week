//! WMI-based Hyper-V management interface
//!
//! Provides a high-level API for managing Hyper-V VMs, switches, and snapshots
//! using WMI (Msvm_* classes) instead of HCS or PowerShell.

use crate::error::Result;
use crate::wmi::msvm::*;
use crate::wmi::operations::{self, VhdFormat, VhdType};
use crate::wmi::WmiConnection;

/// WMI-based Hyper-V management interface
///
/// Provides comprehensive VM management through the Msvm_* WMI classes,
/// offering more detailed information and control than the HCS-based interface.
pub struct HyperVWmi {
    conn: WmiConnection,
}

impl HyperVWmi {
    /// Create a new WMI-based Hyper-V interface
    pub fn new() -> Result<Self> {
        let conn = WmiConnection::connect_hyperv()?;
        Ok(HyperVWmi { conn })
    }

    /// Get the underlying WMI connection for advanced operations
    pub fn connection(&self) -> &WmiConnection {
        &self.conn
    }

    // =========================================================================
    // VM Operations
    // =========================================================================

    /// List all VMs with detailed information
    ///
    /// Returns VMs with name, state, memory, CPU count, and generation.
    pub fn list_vms(&self) -> Result<Vec<MsvmVm>> {
        operations::list_vms(&self.conn)
    }

    /// Get a VM by name
    pub fn get_vm(&self, name: &str) -> Result<MsvmVm> {
        operations::get_vm_by_name(&self.conn, name)
    }

    /// Get a VM by GUID
    pub fn get_vm_by_id(&self, id: &str) -> Result<MsvmVm> {
        operations::get_vm_by_id(&self.conn, id)
    }

    /// Create a new VM with a new VHD
    ///
    /// This is the standard way to create a VM - it creates both the VM and a new virtual hard disk.
    ///
    /// # Arguments
    /// * `name` - VM name
    /// * `memory_mb` - Memory in MB
    /// * `cpu_count` - Number of virtual processors
    /// * `generation` - VM generation (1 or 2)
    /// * `vhd_path` - Path where the new VHD will be created
    /// * `vhd_size_bytes` - Size of the VHD in bytes
    /// * `switch_name` - Optional virtual switch to connect to
    pub fn create_vm(
        &self,
        name: &str,
        memory_mb: u64,
        cpu_count: u32,
        generation: u32,
        vhd_path: &str,
        vhd_size_bytes: u64,
        switch_name: Option<&str>,
    ) -> Result<MsvmVm> {
        operations::create_vm(
            &self.conn,
            name,
            memory_mb,
            cpu_count,
            generation,
            vhd_path,
            vhd_size_bytes,
            switch_name,
        )
    }

    /// Create a new VM with an existing VHD
    ///
    /// Use this when you already have a VHD (e.g., from a template or previous VM).
    ///
    /// # Arguments
    /// * `name` - VM name
    /// * `memory_mb` - Memory in MB
    /// * `cpu_count` - Number of virtual processors
    /// * `generation` - VM generation (1 or 2)
    /// * `vhd_path` - Path to the existing VHD to attach
    /// * `switch_name` - Optional virtual switch to connect to
    pub fn create_vm_with_vhd(
        &self,
        name: &str,
        memory_mb: u64,
        cpu_count: u32,
        generation: u32,
        vhd_path: &str,
        switch_name: Option<&str>,
    ) -> Result<MsvmVm> {
        operations::create_vm_with_vhd(
            &self.conn,
            name,
            memory_mb,
            cpu_count,
            generation,
            vhd_path,
            switch_name,
        )
    }

    /// Create a new VHD file
    ///
    /// # Arguments
    /// * `path` - Path where the VHD file will be created
    /// * `size_bytes` - Size of the VHD in bytes
    /// * `vhd_type` - Type of VHD (Fixed, Dynamic, or Differencing)
    /// * `vhd_format` - Format (Vhd or Vhdx)
    pub fn create_vhd(
        &self,
        path: &str,
        size_bytes: u64,
        vhd_type: VhdType,
        vhd_format: VhdFormat,
    ) -> Result<()> {
        operations::create_vhd(&self.conn, path, size_bytes, vhd_type, vhd_format)
    }

    /// Delete a VM
    pub fn delete_vm(&self, name: &str) -> Result<()> {
        operations::delete_vm(&self.conn, name)
    }

    /// Start a VM
    pub fn start_vm(&self, name: &str) -> Result<()> {
        operations::start_vm(&self.conn, name)
    }

    /// Stop a VM (hard power off)
    pub fn stop_vm(&self, name: &str) -> Result<()> {
        operations::stop_vm(&self.conn, name)
    }

    /// Shutdown a VM gracefully
    ///
    /// Uses the Hyper-V integration services to perform a graceful shutdown.
    /// Falls back to hard stop if integration services are not available.
    pub fn shutdown_vm(&self, name: &str) -> Result<()> {
        operations::shutdown_vm(&self.conn, name)
    }

    /// Save VM state (hibernate)
    pub fn save_vm(&self, name: &str) -> Result<()> {
        operations::save_vm(&self.conn, name)
    }

    /// Pause a running VM
    pub fn pause_vm(&self, name: &str) -> Result<()> {
        operations::pause_vm(&self.conn, name)
    }

    /// Resume a paused or saved VM
    pub fn resume_vm(&self, name: &str) -> Result<()> {
        operations::resume_vm(&self.conn, name)
    }

    /// Get VM settings
    pub fn get_vm_settings(&self, vm_name: &str) -> Result<MsvmVmSettings> {
        let vm = self.get_vm(vm_name)?;
        operations::get_vm_settings(&self.conn, &vm.id)
    }

    /// Get VM memory settings
    pub fn get_vm_memory(&self, vm_name: &str) -> Result<MsvmMemorySettings> {
        let vm = self.get_vm(vm_name)?;
        operations::get_vm_memory_settings(&self.conn, &vm.id)
    }

    /// Get VM processor settings
    pub fn get_vm_processor(&self, vm_name: &str) -> Result<MsvmProcessorSettings> {
        let vm = self.get_vm(vm_name)?;
        operations::get_vm_processor_settings(&self.conn, &vm.id)
    }

    /// Configure VM memory
    pub fn set_vm_memory(&self, vm_name: &str, memory_mb: u64) -> Result<()> {
        let vm = self.get_vm(vm_name)?;
        operations::configure_vm_memory(&self.conn, &vm.id, memory_mb)
    }

    /// Configure VM processor count
    pub fn set_vm_processor(&self, vm_name: &str, cpu_count: u32) -> Result<()> {
        let vm = self.get_vm(vm_name)?;
        operations::configure_vm_processor(&self.conn, &vm.id, cpu_count)
    }

    /// Add a VHD to a VM
    pub fn add_vhd(&self, vm_name: &str, vhd_path: &str) -> Result<()> {
        let vm = self.get_vm(vm_name)?;
        operations::add_vhd_to_vm(&self.conn, &vm.id, vhd_path)
    }

    /// Mount an ISO to a VM
    pub fn mount_iso(&self, vm_name: &str, iso_path: &str) -> Result<()> {
        operations::mount_iso(&self.conn, vm_name, iso_path)
    }

    // =========================================================================
    // Switch Operations
    // =========================================================================

    /// List all virtual switches
    pub fn list_switches(&self) -> Result<Vec<MsvmSwitch>> {
        operations::list_switches(&self.conn)
    }

    /// Get a switch by name
    pub fn get_switch(&self, name: &str) -> Result<MsvmSwitch> {
        operations::get_switch_by_name(&self.conn, name)
    }

    /// Create a virtual switch
    ///
    /// # Arguments
    /// * `name` - Switch name
    /// * `switch_type` - "Private" or "Internal"
    pub fn create_switch(&self, name: &str, switch_type: &str) -> Result<MsvmSwitch> {
        operations::create_switch(&self.conn, name, switch_type)
    }

    /// Delete a virtual switch
    pub fn delete_switch(&self, name: &str) -> Result<()> {
        operations::delete_switch(&self.conn, name)
    }

    /// Connect a VM to a switch
    pub fn connect_vm_to_switch(&self, vm_name: &str, switch_name: &str) -> Result<()> {
        operations::connect_vm_to_switch(&self.conn, vm_name, switch_name)
    }

    // =========================================================================
    // Snapshot Operations
    // =========================================================================

    /// List snapshots for a VM
    pub fn list_snapshots(&self, vm_name: &str) -> Result<Vec<MsvmSnapshot>> {
        operations::list_snapshots(&self.conn, vm_name)
    }

    /// Create a snapshot
    pub fn create_snapshot(&self, vm_name: &str, snapshot_name: &str) -> Result<MsvmSnapshot> {
        operations::create_snapshot(&self.conn, vm_name, snapshot_name)
    }

    /// Apply (restore) a snapshot
    pub fn apply_snapshot(&self, vm_name: &str, snapshot_name: &str) -> Result<()> {
        operations::apply_snapshot(&self.conn, vm_name, snapshot_name)
    }

    /// Delete a snapshot
    pub fn delete_snapshot(&self, vm_name: &str, snapshot_name: &str) -> Result<()> {
        operations::delete_snapshot(&self.conn, vm_name, snapshot_name)
    }
}

impl Default for HyperVWmi {
    fn default() -> Self {
        Self::new().expect("Failed to create HyperVWmi instance")
    }
}
