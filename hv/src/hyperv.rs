//! Main Hyper-V management interface using WMI
//!
//! Provides a high-level API for managing Hyper-V VMs, switches, VHDs, snapshots, and GPUs.

use crate::disk::{self, DvdDrive, FileSystem, HardDiskDrive, PartitionStyle, WindowsEdition};
use crate::error::{HvError, Result};
use crate::gpu::{self, AssignableDevice, DdaSupportInfo, GpuInfo, GpuPartitionAdapter};
use crate::snapshot::{self, Snapshot, SnapshotType};
use crate::switch::{self, SwitchType, VirtualSwitch};
use crate::vhd::{self, Vhd, VhdType};
use crate::vm::{Vm, VmGeneration};
use crate::wmi::operations::{self as wmi_ops, VhdFormat, VhdType as WmiVhdType};
use crate::wmi::WmiConnection;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Main interface for Hyper-V management
pub struct HyperV {
    conn: WmiConnection,
}

impl HyperV {
    /// Creates a new HyperV management interface
    pub fn new() -> Result<Self> {
        let conn = WmiConnection::connect_hyperv()?;
        Ok(HyperV { conn })
    }

    // =========================================================================
    // VM Operations
    // =========================================================================

    /// Lists all virtual machines using WMI
    pub fn list_vms(&self) -> Result<Vec<Vm>> {
        let wmi_vms = wmi_ops::list_vms(&self.conn)?;

        Ok(wmi_vms
            .into_iter()
            .map(|vm| Vm::from_wmi(&vm))
            .collect())
    }

    /// Gets a VM by name
    pub fn get_vm(&self, name: &str) -> Result<Vm> {
        let wmi_vm = wmi_ops::get_vm_by_name(&self.conn, name)?;
        Ok(Vm::from_wmi(&wmi_vm))
    }

    /// Gets a VM by ID
    pub fn get_vm_by_id(&self, id: &str) -> Result<Vm> {
        let wmi_vm = wmi_ops::get_vm_by_id(&self.conn, id)?;
        Ok(Vm::from_wmi(&wmi_vm))
    }

    /// Creates a new VM with a new VHD
    ///
    /// This is the standard way to create a VM - it creates both the VM and a new virtual hard disk.
    ///
    /// # Arguments
    /// * `name` - VM name
    /// * `memory_mb` - Memory in MB
    /// * `cpu_count` - Number of virtual processors
    /// * `generation` - VM generation (Gen1 or Gen2)
    /// * `vhd_path` - Path where the new VHD will be created
    /// * `vhd_size_bytes` - Size of the VHD in bytes
    /// * `switch_name` - Optional virtual switch to connect to
    pub fn create_vm(
        &self,
        name: &str,
        memory_mb: u64,
        cpu_count: u32,
        generation: VmGeneration,
        vhd_path: &str,
        vhd_size_bytes: u64,
        switch_name: Option<&str>,
    ) -> Result<Vm> {
        let gen = match generation {
            VmGeneration::Gen1 => 1,
            VmGeneration::Gen2 => 2,
        };

        let wmi_vm = wmi_ops::create_vm(
            &self.conn,
            name,
            memory_mb,
            cpu_count,
            gen,
            vhd_path,
            vhd_size_bytes,
            switch_name,
        )?;
        Ok(Vm::from_wmi(&wmi_vm))
    }

    /// Creates a new VM with an existing VHD
    ///
    /// Use this when you already have a VHD (e.g., from a template or previous VM).
    ///
    /// # Arguments
    /// * `name` - VM name
    /// * `memory_mb` - Memory in MB
    /// * `cpu_count` - Number of virtual processors
    /// * `generation` - VM generation (Gen1 or Gen2)
    /// * `vhd_path` - Path to the existing VHD to attach
    /// * `switch_name` - Optional virtual switch to connect to
    pub fn create_vm_with_vhd(
        &self,
        name: &str,
        memory_mb: u64,
        cpu_count: u32,
        generation: VmGeneration,
        vhd_path: &str,
        switch_name: Option<&str>,
    ) -> Result<Vm> {
        let gen = match generation {
            VmGeneration::Gen1 => 1,
            VmGeneration::Gen2 => 2,
        };

        let wmi_vm = wmi_ops::create_vm_with_vhd(
            &self.conn,
            name,
            memory_mb,
            cpu_count,
            gen,
            vhd_path,
            switch_name,
        )?;
        Ok(Vm::from_wmi(&wmi_vm))
    }

    /// Creates a new VHD file using WMI
    ///
    /// # Arguments
    /// * `path` - Path where the VHD file will be created
    /// * `size_bytes` - Size of the VHD in bytes
    /// * `dynamic` - If true, creates a dynamic VHD; if false, creates a fixed VHD
    pub fn create_vhd_wmi(&self, path: &str, size_bytes: u64, dynamic: bool) -> Result<()> {
        let vhd_type = if dynamic {
            WmiVhdType::Dynamic
        } else {
            WmiVhdType::Fixed
        };

        let vhd_format = if path.to_lowercase().ends_with(".vhdx") {
            VhdFormat::Vhdx
        } else {
            VhdFormat::Vhd
        };

        wmi_ops::create_vhd(&self.conn, path, size_bytes, vhd_type, vhd_format)
    }

    /// Deletes a VM using WMI
    pub fn delete_vm(&self, name: &str) -> Result<()> {
        wmi_ops::delete_vm(&self.conn, name)
    }

    /// Imports a VM from an exported configuration
    pub fn import_vm(&self, path: &str, copy: bool) -> Result<Vm> {
        let copy_arg = if copy { "-Copy" } else { "" };

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "$vm = Import-VM -Path '{}' {} -GenerateNewId; $vm.Name",
                    path, copy_arg
                ),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        let vm_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        self.get_vm(&vm_name)
    }

    /// Exports a VM to a path
    pub fn export_vm(&self, name: &str, path: &str) -> Result<()> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Export-VM -Name '{}' -Path '{}'", name, path),
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        Ok(())
    }

    // =========================================================================
    // Virtual Switch Operations
    // =========================================================================

    /// Lists all virtual switches
    pub fn list_switches(&self) -> Result<Vec<VirtualSwitch>> {
        switch::enumerate_switches()
    }

    /// Gets a virtual switch by name
    pub fn get_switch(&self, name: &str) -> Result<VirtualSwitch> {
        switch::get_switch(name)
    }

    /// Creates a new virtual switch
    pub fn create_switch(&self, name: &str, switch_type: SwitchType) -> Result<VirtualSwitch> {
        switch::create_switch(name, switch_type)
    }

    /// Creates an external virtual switch connected to a physical NIC
    pub fn create_external_switch(
        &self,
        name: &str,
        network_adapter_name: &str,
        allow_management_os: bool,
    ) -> Result<VirtualSwitch> {
        switch::create_external_switch(name, network_adapter_name, allow_management_os)
    }

    // =========================================================================
    // VHD Operations
    // =========================================================================

    /// Gets a VHD by path
    pub fn get_vhd(&self, path: &str) -> Result<Vhd> {
        // Verify the VHD exists by trying to open it
        let vhd = Vhd::new(path.to_string());
        // Try to get info to verify it exists
        let _ = vhd.format();
        Ok(vhd)
    }

    /// Creates a new VHD
    pub fn create_vhd(
        &self,
        path: &str,
        size_bytes: u64,
        vhd_type: VhdType,
        block_size_bytes: Option<u32>,
    ) -> Result<Vhd> {
        vhd::create_vhd(path, size_bytes, vhd_type, block_size_bytes)
    }

    /// Creates a differencing VHD
    pub fn create_differencing_vhd(&self, path: &str, parent_path: &str) -> Result<Vhd> {
        vhd::create_differencing_vhd(path, parent_path)
    }

    // =========================================================================
    // Snapshot Operations
    // =========================================================================

    /// Lists all snapshots for a VM
    pub fn list_snapshots(&self, vm_name: &str) -> Result<Vec<Snapshot>> {
        snapshot::list_snapshots(vm_name)
    }

    /// Gets a specific snapshot
    pub fn get_snapshot(&self, vm_name: &str, snapshot_name: &str) -> Result<Snapshot> {
        snapshot::get_snapshot(vm_name, snapshot_name)
    }

    /// Creates a new snapshot for a VM
    pub fn create_snapshot(
        &self,
        vm_name: &str,
        snapshot_name: &str,
        snapshot_type: SnapshotType,
    ) -> Result<Snapshot> {
        snapshot::create_snapshot(vm_name, snapshot_name, snapshot_type)
    }

    // =========================================================================
    // GPU Operations
    // =========================================================================

    /// Lists all GPUs on the host
    pub fn list_gpus(&self) -> Result<Vec<GpuInfo>> {
        gpu::enumerate_gpus()
    }

    /// Lists GPUs that support GPU-P (partitioning)
    pub fn list_partitionable_gpus(&self) -> Result<Vec<GpuInfo>> {
        gpu::get_partitionable_gpus()
    }

    /// Adds a GPU partition adapter to a VM
    ///
    /// This enables GPU-P (GPU Partitioning) for the VM, allowing it to share
    /// a GPU with the host and other VMs.
    ///
    /// # Arguments
    /// * `vm_name` - Name of the VM
    /// * `instance_path` - Optional GPU device instance path. If None, uses the default GPU.
    pub fn add_gpu_to_vm(&self, vm_name: &str, instance_path: Option<&str>) -> Result<()> {
        gpu::add_gpu_partition_adapter(vm_name, instance_path)
    }

    /// Removes GPU partition adapter from a VM
    pub fn remove_gpu_from_vm(&self, vm_name: &str) -> Result<()> {
        gpu::remove_gpu_partition_adapter(vm_name)
    }

    /// Gets GPU partition adapters for a VM
    pub fn get_vm_gpu_adapters(&self, vm_name: &str) -> Result<Vec<GpuPartitionAdapter>> {
        gpu::get_gpu_partition_adapters(vm_name)
    }

    /// Configures GPU partition adapter properties for a VM
    ///
    /// # Arguments
    /// * `vm_name` - Name of the VM
    /// * `min_vram` - Minimum VRAM in bytes
    /// * `max_vram` - Maximum VRAM in bytes
    /// * `optimal_vram` - Optimal VRAM in bytes
    #[allow(clippy::too_many_arguments)]
    pub fn configure_vm_gpu_adapter(
        &self,
        vm_name: &str,
        min_vram: Option<u64>,
        max_vram: Option<u64>,
        optimal_vram: Option<u64>,
        min_encode: Option<u64>,
        max_encode: Option<u64>,
        optimal_encode: Option<u64>,
        min_decode: Option<u64>,
        max_decode: Option<u64>,
        optimal_decode: Option<u64>,
        min_compute: Option<u64>,
        max_compute: Option<u64>,
        optimal_compute: Option<u64>,
    ) -> Result<()> {
        gpu::set_gpu_partition_adapter(
            vm_name,
            min_vram,
            max_vram,
            optimal_vram,
            min_encode,
            max_encode,
            optimal_encode,
            min_decode,
            max_decode,
            optimal_decode,
            min_compute,
            max_compute,
            optimal_compute,
        )
    }

    /// Configures VM settings required for GPU-P
    ///
    /// Sets GuestControlledCacheTypes and memory mapped IO space.
    ///
    /// # Arguments
    /// * `vm_name` - Name of the VM
    /// * `low_mmio_gb` - Low memory mapped IO space in GB (typically 1)
    /// * `high_mmio_gb` - High memory mapped IO space in GB (typically 32 or more)
    pub fn configure_vm_for_gpu(
        &self,
        vm_name: &str,
        low_mmio_gb: u32,
        high_mmio_gb: u32,
    ) -> Result<()> {
        gpu::configure_vm_for_gpu(vm_name, low_mmio_gb, high_mmio_gb)
    }

    /// Copies GPU drivers from host to VM's VHD
    ///
    /// Required for GPU-PV to work. The VM must be stopped and VHD accessible.
    ///
    /// # Arguments
    /// * `vhd_path` - Path to the VM's VHD file
    /// * `host_driver_store` - Optional custom driver store path
    pub fn copy_gpu_drivers_to_vm(
        &self,
        vhd_path: &str,
        host_driver_store: Option<&str>,
    ) -> Result<()> {
        gpu::copy_gpu_drivers_to_vm(vhd_path, host_driver_store)
    }

    // =========================================================================
    // DDA (Discrete Device Assignment) Operations
    // =========================================================================

    /// Check if DDA is supported on this host
    ///
    /// DDA requires Windows Server 2016+ with IOMMU support.
    pub fn check_dda_support(&self) -> Result<DdaSupportInfo> {
        gpu::check_dda_support()
    }

    /// Get all devices that can be assigned via DDA
    ///
    /// Returns devices that are either dismounted from host or assigned to VMs.
    pub fn get_assignable_devices(&self) -> Result<Vec<AssignableDevice>> {
        gpu::get_assignable_devices()
    }

    /// Get the PCI location path for a device
    ///
    /// The location path is required for DDA operations.
    pub fn get_device_location_path(&self, instance_id: &str) -> Result<String> {
        gpu::get_device_location_path(instance_id)
    }

    /// Dismount a device from the host for DDA assignment
    ///
    /// This disables the device on the host so it can be assigned to a VM.
    /// The device must be dismounted before calling `assign_device_to_vm`.
    pub fn dismount_device(&self, location_path: &str) -> Result<()> {
        gpu::dismount_device_from_host(location_path)
    }

    /// Mount a device back to the host
    ///
    /// Re-enables a dismounted device on the host.
    pub fn mount_device(&self, location_path: &str) -> Result<()> {
        gpu::mount_device_to_host(location_path)
    }

    /// Assign a device to a VM via DDA (exclusive passthrough)
    ///
    /// The device must be dismounted first. The VM must be stopped.
    /// Unlike GPU-P, DDA provides exclusive device access to the VM.
    pub fn assign_device_to_vm(&self, vm_name: &str, location_path: &str) -> Result<()> {
        gpu::add_assignable_device_to_vm(vm_name, location_path)
    }

    /// Remove an assigned device from a VM
    ///
    /// The VM must be stopped. After removal, mount the device back to host
    /// or assign it to another VM.
    pub fn remove_assigned_device(&self, vm_name: &str, location_path: &str) -> Result<()> {
        gpu::remove_assignable_device_from_vm(vm_name, location_path)
    }

    /// Get devices assigned to a VM via DDA
    pub fn get_vm_assigned_devices(&self, vm_name: &str) -> Result<Vec<AssignableDevice>> {
        gpu::get_vm_assignable_devices(vm_name)
    }

    /// Move a DDA device from one VM to another
    ///
    /// Both VMs must be stopped.
    pub fn move_assigned_device(
        &self,
        source_vm: &str,
        target_vm: &str,
        location_path: &str,
    ) -> Result<()> {
        gpu::move_assignable_device(source_vm, target_vm, location_path)
    }

    /// Configure VM for DDA device assignment
    ///
    /// Sets automatic stop action to TurnOff (required for DDA).
    pub fn configure_vm_for_dda(&self, vm_name: &str) -> Result<()> {
        gpu::configure_vm_for_dda(vm_name, None)
    }

    /// Set MMIO space for a VM (required for DDA GPUs)
    ///
    /// # Arguments
    /// * `vm_name` - Name of the VM
    /// * `low_mmio_mb` - Low MMIO space in MB (typically 128-256)
    /// * `high_mmio_gb` - High MMIO space in GB (typically 32+ for GPUs)
    pub fn set_vm_mmio_space(
        &self,
        vm_name: &str,
        low_mmio_mb: u64,
        high_mmio_gb: u64,
    ) -> Result<()> {
        gpu::set_vm_mmio_space(vm_name, low_mmio_mb, high_mmio_gb)
    }

    // =========================================================================
    // DVD/ISO Operations
    // =========================================================================

    /// Get DVD drives for a VM
    pub fn get_dvd_drives(&self, vm_name: &str) -> Result<Vec<DvdDrive>> {
        disk::get_dvd_drives(vm_name)
    }

    /// Add a DVD drive to a VM
    pub fn add_dvd_drive(&self, vm_name: &str) -> Result<()> {
        disk::add_dvd_drive(vm_name, None, None)
    }

    /// Mount an ISO to a VM's DVD drive
    pub fn mount_iso(&self, vm_name: &str, iso_path: &str) -> Result<()> {
        disk::mount_iso(vm_name, iso_path, None, None)
    }

    /// Eject ISO from a VM's DVD drive
    pub fn eject_iso(&self, vm_name: &str) -> Result<()> {
        disk::eject_iso(vm_name, None, None)
    }

    /// Set boot order for a Gen2 VM
    ///
    /// # Arguments
    /// * `vm_name` - Name of the VM
    /// * `boot_devices` - Array of boot devices: "DVD", "VHD", "Network"
    pub fn set_boot_order(&self, vm_name: &str, boot_devices: &[&str]) -> Result<()> {
        disk::set_boot_order(vm_name, boot_devices)
    }

    // =========================================================================
    // Hard Disk Operations
    // =========================================================================

    /// Get hard disk drives for a VM
    pub fn get_hard_disk_drives(&self, vm_name: &str) -> Result<Vec<HardDiskDrive>> {
        disk::get_hard_disk_drives(vm_name)
    }

    /// Add a hard disk drive to a VM
    pub fn add_hard_disk_drive(&self, vm_name: &str, vhd_path: &str) -> Result<()> {
        disk::add_hard_disk_drive(vm_name, vhd_path, None, None, None)
    }

    /// Remove a hard disk drive from a VM
    pub fn remove_hard_disk_drive(
        &self,
        vm_name: &str,
        controller_number: u32,
        controller_location: u32,
    ) -> Result<()> {
        disk::remove_hard_disk_drive(vm_name, controller_number, controller_location)
    }

    // =========================================================================
    // Disk Initialization
    // =========================================================================

    /// Initialize a VHD with a single partition
    ///
    /// Mounts the VHD, initializes it, creates a partition, and formats it.
    /// Returns the drive letter of the mounted partition.
    pub fn initialize_vhd(
        &self,
        vhd_path: &str,
        partition_style: PartitionStyle,
        file_system: FileSystem,
        label: Option<&str>,
    ) -> Result<String> {
        disk::initialize_vhd(vhd_path, partition_style, file_system, label)
    }

    /// Initialize a VHD with Windows boot partitions
    ///
    /// Creates EFI System Partition, MSR, and Windows partition.
    /// Returns the drive letter of the Windows partition.
    pub fn initialize_windows_vhd(&self, vhd_path: &str, label: Option<&str>) -> Result<String> {
        disk::initialize_windows_vhd(vhd_path, label)
    }

    /// Dismount a VHD that was mounted for initialization
    pub fn dismount_vhd(&self, vhd_path: &str) -> Result<()> {
        disk::dismount_vhd(vhd_path)
    }

    // =========================================================================
    // Windows Image Operations
    // =========================================================================

    /// Get available Windows editions from an ISO
    pub fn get_windows_editions(&self, iso_path: &str) -> Result<Vec<WindowsEdition>> {
        disk::get_windows_editions(iso_path)
    }

    /// Create a bootable Windows VHDX from an ISO
    ///
    /// # Arguments
    /// * `iso_path` - Path to Windows ISO file
    /// * `vhdx_path` - Path for the new VHDX file
    /// * `size_gb` - Size of the VHDX in GB
    /// * `edition_index` - Index of Windows edition (use `get_windows_editions` to list)
    pub fn create_vhdx_from_iso(
        &self,
        iso_path: &str,
        vhdx_path: &str,
        size_gb: u64,
        edition_index: u32,
    ) -> Result<()> {
        disk::create_vhdx_from_iso(iso_path, vhdx_path, size_gb, edition_index)
    }

    /// Quick create a VM with Windows from ISO
    ///
    /// Creates a bootable VHDX from ISO and a Gen2 VM in one step.
    pub fn quick_create_windows_vm(
        &self,
        vm_name: &str,
        iso_path: &str,
        vhdx_path: &str,
        size_gb: u64,
        memory_mb: u64,
        cpu_count: u32,
        edition_index: u32,
    ) -> Result<()> {
        disk::quick_create_windows_vm(
            vm_name,
            iso_path,
            vhdx_path,
            size_gb,
            memory_mb,
            cpu_count,
            edition_index,
        )
    }

    // =========================================================================
    // Utility Methods
    // =========================================================================

    /// Gets the Hyper-V host information
    pub fn host_info(&self) -> Result<HostInfo> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-VMHost | Select-Object ComputerName, LogicalProcessorCount, \
                 MemoryCapacity, VirtualMachinePath, VirtualHardDiskPath | ConvertTo-Json -Compress",
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HvError::OperationFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let info: HostInfoJson = serde_json::from_str(&stdout)
            .map_err(|e| HvError::JsonError(format!("Failed to parse host info: {}", e)))?;

        Ok(HostInfo {
            computer_name: info.computer_name.unwrap_or_default(),
            logical_processor_count: info.logical_processor_count.unwrap_or(0),
            memory_capacity_bytes: info.memory_capacity.unwrap_or(0),
            vm_path: info.virtual_machine_path.unwrap_or_default(),
            vhd_path: info.virtual_hard_disk_path.unwrap_or_default(),
        })
    }

    /// Lists available physical network adapters
    pub fn list_network_adapters(&self) -> Result<Vec<NetworkAdapterInfo>> {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | \
                 Select-Object Name, InterfaceDescription, MacAddress, LinkSpeed | \
                 ConvertTo-Json -Compress",
            ])
            .output()
            .map_err(|e| HvError::OperationFailed(e.to_string()))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();

        if trimmed.is_empty() || trimmed == "null" {
            return Ok(Vec::new());
        }

        // Handle both single object and array
        let adapters: Vec<NetworkAdapterJson> = if trimmed.starts_with('[') {
            serde_json::from_str(trimmed)
                .map_err(|e| HvError::JsonError(format!("Failed to parse adapters: {}", e)))?
        } else {
            let single: NetworkAdapterJson = serde_json::from_str(trimmed)
                .map_err(|e| HvError::JsonError(format!("Failed to parse adapter: {}", e)))?;
            vec![single]
        };

        Ok(adapters
            .into_iter()
            .map(|a| NetworkAdapterInfo {
                name: a.name.unwrap_or_default(),
                description: a.interface_description.unwrap_or_default(),
                mac_address: a.mac_address.unwrap_or_default(),
                link_speed: a.link_speed.unwrap_or_default(),
            })
            .collect())
    }
}

/// Hyper-V host information
#[derive(Debug, Clone)]
pub struct HostInfo {
    pub computer_name: String,
    pub logical_processor_count: u32,
    pub memory_capacity_bytes: u64,
    pub vm_path: String,
    pub vhd_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct HostInfoJson {
    computer_name: Option<String>,
    logical_processor_count: Option<u32>,
    memory_capacity: Option<u64>,
    virtual_machine_path: Option<String>,
    virtual_hard_disk_path: Option<String>,
}

/// Network adapter information
#[derive(Debug, Clone)]
pub struct NetworkAdapterInfo {
    pub name: String,
    pub description: String,
    pub mac_address: String,
    pub link_speed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NetworkAdapterJson {
    name: Option<String>,
    interface_description: Option<String>,
    mac_address: Option<String>,
    link_speed: Option<String>,
}
