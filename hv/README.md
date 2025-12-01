# hv

Rust library for Windows Hyper-V management via HCS (Host Compute Service) and native Windows APIs.

## Features

- **VM Management**: Create, delete, start, stop, pause, resume, save, reset VMs
- **Virtual Switches**: Create, delete, and manage virtual network switches
- **VHD/VHDX Management**: Create, resize, compact, convert, mount/dismount virtual hard disks
- **Snapshots**: Create, restore, delete, and export VM checkpoints
- **GPU-P Support**: GPU partitioning for sharing GPUs between host and VMs
- **DDA Support**: Discrete Device Assignment for exclusive GPU passthrough (Server only)
- **DVD/ISO Management**: Mount/eject ISO images, manage boot order
- **Windows Image**: Create bootable VHDX from Windows ISO
- **Disk Initialization**: Partition and format VHDs with GPT/MBR and NTFS/ReFS
- **Host Information**: Query Hyper-V host capabilities and configuration

## Modules

- `hyperv` - Main interface for Hyper-V management
- `vm` - Virtual machine operations via HCS
- `hcs` - Host Compute Service API wrappers
- `switch` - Virtual switch management
- `vhd` - Virtual hard disk management via VirtDisk API
- `snapshot` - VM snapshot/checkpoint operations
- `gpu` - GPU enumeration and GPU-P partition adapter management
- `disk` - DVD/ISO, disk initialization, and Windows image operations
- `error` - Error types

## Usage

```rust
use hv::{HyperV, VmState, VmGeneration, SnapshotType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HyperV management interface
    let hyperv = HyperV::new()?;

    // Get host info
    let host = hyperv.host_info()?;
    println!("Host: {}", host.computer_name);

    // List all VMs
    for vm in hyperv.list_vms()? {
        println!("{}: {:?}", vm.name(), vm.state()?);
    }

    // Create a new VM
    let vm = hyperv.create_vm("MyVM", 2048, VmGeneration::Gen2, None)?;

    // Start the VM
    vm.start()?;

    // Create a snapshot
    hyperv.create_snapshot("MyVM", "Before Update", SnapshotType::Standard)?;

    // Stop the VM
    vm.stop()?;

    Ok(())
}
```

## API Reference

### HyperV

| Method | Description |
|--------|-------------|
| `new()` | Create HyperV management interface |
| `host_info()` | Get host information |
| `list_vms()` | List all VMs |
| `get_vm(name)` | Get VM by name |
| `create_vm(...)` | Create a new VM |
| `delete_vm(name)` | Delete a VM |
| `export_vm(name, path)` | Export a VM |
| `import_vm(path, copy)` | Import a VM |
| `list_switches()` | List virtual switches |
| `get_switch(name)` | Get switch by name |
| `create_switch(...)` | Create a switch |
| `create_external_switch(...)` | Create external switch |
| `get_vhd(path)` | Get VHD by path |
| `create_vhd(...)` | Create a VHD |
| `create_differencing_vhd(...)` | Create differencing VHD |
| `list_snapshots(vm)` | List VM snapshots |
| `get_snapshot(vm, name)` | Get snapshot |
| `create_snapshot(...)` | Create snapshot |
| `list_gpus()` | List all host GPUs |
| `list_partitionable_gpus()` | List GPUs supporting GPU-P |
| `add_gpu_to_vm(vm, path)` | Add GPU partition adapter |
| `remove_gpu_from_vm(vm)` | Remove GPU partition adapter |
| `get_vm_gpu_adapters(vm)` | Get VM's GPU adapters |
| `configure_vm_gpu_adapter(...)` | Configure GPU adapter properties |
| `configure_vm_for_gpu(...)` | Set VM GPU settings |
| `copy_gpu_drivers_to_vm(...)` | Copy GPU drivers to VHD |
| `check_dda_support()` | Check if DDA is supported |
| `get_assignable_devices()` | List DDA-assignable devices |
| `get_device_location_path(id)` | Get PCI location path |
| `dismount_device(path)` | Dismount device from host |
| `mount_device(path)` | Mount device back to host |
| `assign_device_to_vm(vm, path)` | Assign device via DDA |
| `remove_assigned_device(vm, path)` | Remove DDA device |
| `get_vm_assigned_devices(vm)` | Get VM's DDA devices |
| `move_assigned_device(...)` | Move DDA device between VMs |
| `configure_vm_for_dda(vm)` | Configure VM for DDA |
| `set_vm_mmio_space(...)` | Set MMIO space for DDA |
| `get_dvd_drives(vm)` | Get VM's DVD drives |
| `add_dvd_drive(vm)` | Add DVD drive to VM |
| `mount_iso(vm, path)` | Mount ISO to VM |
| `eject_iso(vm)` | Eject ISO from VM |
| `set_boot_order(vm, devices)` | Set boot order (Gen2) |
| `get_hard_disk_drives(vm)` | Get VM's hard disks |
| `add_hard_disk_drive(vm, path)` | Attach VHD to VM |
| `remove_hard_disk_drive(...)` | Detach VHD from VM |
| `initialize_vhd(...)` | Initialize VHD with partition |
| `initialize_windows_vhd(...)` | Init VHD with boot partitions |
| `dismount_vhd(path)` | Dismount initialized VHD |
| `get_windows_editions(iso)` | List Windows editions in ISO |
| `create_vhdx_from_iso(...)` | Create bootable VHDX from ISO |
| `quick_create_windows_vm(...)` | Create VM with Windows from ISO |

### Vm

| Method | Description |
|--------|-------------|
| `name()` | Get VM name |
| `id()` | Get VM GUID |
| `state()` | Get current state |
| `start()` | Start the VM |
| `stop()` | Graceful shutdown |
| `force_stop()` | Force power off |
| `pause()` | Pause the VM |
| `resume()` | Resume paused VM |
| `save()` | Save VM state |
| `reset()` | Hard reset |
| `cpu_count()` | Get vCPU count |
| `memory_mb()` | Get memory in MB |
| `uptime_seconds()` | Get uptime |

### VirtualSwitch

| Method | Description |
|--------|-------------|
| `name()` | Get switch name |
| `id()` | Get switch GUID |
| `switch_type()` | Get type (External/Internal/Private) |
| `connected_vms()` | List connected VMs |
| `delete()` | Delete the switch |

### Vhd

| Method | Description |
|--------|-------------|
| `path()` | Get file path |
| `format()` | Get format (VHD/VHDX) |
| `vhd_type()` | Get type (Fixed/Dynamic/Differencing) |
| `max_size_bytes()` | Get maximum size |
| `file_size_bytes()` | Get current file size |
| `parent_path()` | Get parent for differencing disks |
| `is_attached()` | Check if attached to VM |
| `resize(size)` | Resize the VHD |
| `compact()` | Compact dynamic VHD |
| `convert(path, type)` | Convert to different type |
| `merge()` | Merge differencing disk |
| `mount(readonly)` | Mount to host |
| `dismount()` | Dismount from host |

### Snapshot

| Method | Description |
|--------|-------------|
| `name()` | Get snapshot name |
| `id()` | Get snapshot GUID |
| `vm_name()` | Get parent VM name |
| `creation_time()` | Get creation timestamp |
| `parent_name()` | Get parent snapshot |
| `apply()` | Restore this snapshot |
| `delete()` | Delete this snapshot |
| `delete_subtree()` | Delete with children |
| `rename(name)` | Rename snapshot |
| `export(path)` | Export to path |

### GpuInfo

| Field | Description |
|-------|-------------|
| `device_instance_id` | Device instance ID for GPU-P |
| `name` | GPU friendly name |
| `description` | Device description |
| `manufacturer` | GPU manufacturer |
| `hardware_ids` | Hardware ID list |
| `driver` | Driver information |
| `location` | PCI location info |
| `supports_partitioning` | GPU-P support status |

### GpuPartitionAdapter

| Field | Description |
|-------|-------------|
| `vm_name` | Associated VM name |
| `instance_path` | GPU instance path |
| `min_partition_vram` | Min VRAM bytes |
| `max_partition_vram` | Max VRAM bytes |
| `optimal_partition_vram` | Optimal VRAM bytes |
| `*_partition_encode` | Encode capacity (0-100) |
| `*_partition_decode` | Decode capacity (0-100) |
| `*_partition_compute` | Compute capacity (0-100) |

### AssignableDevice (DDA)

| Field | Description |
|-------|-------------|
| `instance_id` | Device instance ID |
| `name` | Device friendly name |
| `location_path` | PCI location path |
| `is_assigned` | Whether assigned to VM |
| `assigned_vm` | VM name if assigned |
| `is_dismounted` | Whether dismounted from host |
| `status` | Device status |

### DdaSupportInfo

| Field | Description |
|-------|-------------|
| `is_supported` | Full DDA support |
| `is_server` | Windows Server detected |
| `has_iommu` | IOMMU available |
| `cmdlet_available` | DDA cmdlets present |
| `reason` | Reason if unsupported |

## Requirements

- Windows 10/11 Pro or Enterprise, or Windows Server 2016+
- Hyper-V feature enabled
- Administrator privileges for most operations

## Examples

```bash
# List all VMs
cargo run -p hv --example list_vms

# VM lifecycle management
cargo run -p hv --example vm_lifecycle -- list
cargo run -p hv --example vm_lifecycle -- start MyVM
cargo run -p hv --example vm_lifecycle -- snapshot MyVM "Before Update"

# GPU information and capabilities
cargo run -p hv --example gpu_info

# GPU-P (Partitioning) - works on Windows 10/11 Pro
cargo run -p hv --example gpu_passthrough -- gpup-add MyVM
cargo run -p hv --example gpu_passthrough -- gpup-config MyVM
cargo run -p hv --example gpu_passthrough -- gpup-list MyVM
cargo run -p hv --example gpu_passthrough -- gpup-drivers "C:\VMs\MyVM.vhdx"

# DDA (Discrete Device Assignment) - Windows Server only
cargo run -p hv --example gpu_passthrough -- dda-check
cargo run -p hv --example gpu_passthrough -- dda-list
cargo run -p hv --example gpu_passthrough -- dda-path "PCI\VEN_10DE&DEV_2204..."
cargo run -p hv --example gpu_passthrough -- dda-dismount "PCIROOT(0)#PCI(0100)#PCI(0000)"
cargo run -p hv --example gpu_passthrough -- dda-assign MyVM "PCIROOT(0)#PCI(0100)#PCI(0000)"

# Create VM with Windows from ISO (automated installation)
cargo run -p hv --example create_vm -- iso-info "C:\ISOs\Win11.iso"
cargo run -p hv --example create_vm -- from-iso Win11VM "C:\ISOs\Win11.iso" "C:\VMs\Win11.vhdx" 64 4096 1

# Create VM for manual OS installation
cargo run -p hv --example create_vm -- with-iso UbuntuVM "C:\ISOs\ubuntu.iso" 50 2048

# Create VM from existing VHDX
cargo run -p hv --example create_vm -- from-vhdx DevVM "C:\VMs\template.vhdx" 8192 4

# Create and attach data disk
cargo run -p hv --example create_vm -- create-data-disk "C:\VMs\data.vhdx" 500 "Data"
cargo run -p hv --example create_vm -- attach-disk MyVM "C:\VMs\data.vhdx"

# Manage ISO/DVD
cargo run -p hv --example create_vm -- mount-iso MyVM "C:\ISOs\tools.iso"
cargo run -p hv --example create_vm -- eject-iso MyVM
cargo run -p hv --example create_vm -- set-boot MyVM DVD VHD

# Quick VM creation from template VHDX
cargo run -p hv --example quick_vm -- list-templates "C:\VMs\Templates"
cargo run -p hv --example quick_vm -- windows Win11Dev "C:\VMs\Templates\Win11.vhdx" "C:\VMs"
cargo run -p hv --example quick_vm -- linux UbuntuDev "C:\VMs\Templates\Ubuntu.vhdx" "C:\VMs"

# Fast VM cloning with differencing disks
cargo run -p hv --example quick_vm -- windows-clone Win11Test "C:\VMs\Templates\Win11.vhdx" "C:\VMs"
cargo run -p hv --example quick_vm -- linux-clone UbuntuTest "C:\VMs\Templates\Ubuntu.vhdx" "C:\VMs"

# Development VMs with nested virtualization and data disk
cargo run -p hv --example quick_vm -- dev-windows DevBox "C:\VMs\Templates\Win11.vhdx" "C:\VMs"
cargo run -p hv --example quick_vm -- dev-linux DockerHost "C:\VMs\Templates\Ubuntu.vhdx" "C:\VMs"

# Server VM with high resources
cargo run -p hv --example quick_vm -- server SQLServer "C:\VMs\Templates\WinServer.vhdx" "C:\VMs" 32768 8

# Batch create multiple VMs from template
cargo run -p hv --example quick_vm -- batch "C:\VMs\Templates\Win11.vhdx" "C:\VMs" TestVM 5
```

## TODO: Low-Level API Support

### VHD/VHDX Enhanced Operations (`Win32_Storage_Vhd`)

- [ ] **VHD Validation & Integrity**
  - [ ] `GetVirtualDiskInformation` with all query types:
    - [ ] `GET_VIRTUAL_DISK_INFO_SIZE` - Size, block size, sector size
    - [ ] `GET_VIRTUAL_DISK_INFO_IDENTIFIER` - Unique GUID
    - [ ] `GET_VIRTUAL_DISK_INFO_PARENT_LOCATION` - Parent paths (differencing)
    - [ ] `GET_VIRTUAL_DISK_INFO_PARENT_IDENTIFIER` - Parent GUID validation
    - [ ] `GET_VIRTUAL_DISK_INFO_PARENT_TIMESTAMP` - Timestamp validation
    - [ ] `GET_VIRTUAL_DISK_INFO_VIRTUAL_STORAGE_TYPE` - VHD vs VHDX
    - [ ] `GET_VIRTUAL_DISK_INFO_PROVIDER_SUBTYPE` - Fixed/Dynamic/Differencing
    - [ ] `GET_VIRTUAL_DISK_INFO_IS_4K_ALIGNED` - 4K alignment check
    - [ ] `GET_VIRTUAL_DISK_INFO_FRAGMENTATION` - Fragmentation percentage
    - [ ] `GET_VIRTUAL_DISK_INFO_IS_LOADED` - Mount status
    - [ ] `GET_VIRTUAL_DISK_INFO_CHANGE_TRACKING_STATE` - RCT status

- [ ] **VHD Set Operations** (for VM checkpoints)
  - [ ] `TakeSnapshotVhdSet` - Create VHD Set snapshot
  - [ ] `DeleteSnapshotVhdSet` - Remove VHD Set snapshot
  - [ ] `ApplySnapshotVhdSet` - Apply/restore VHD Set snapshot
  - [ ] `ModifyVhdSet` - Modify VHD Set properties

- [ ] **Change Tracking** (for backup/replication)
  - [ ] `QueryChangesVirtualDisk` - Get changed blocks (RCT/CBT)
  - [ ] Enable/disable resilient change tracking

- [ ] **Metadata Operations**
  - [ ] `GetVirtualDiskMetadata` - Read custom metadata
  - [ ] `SetVirtualDiskMetadata` - Write custom metadata
  - [ ] `EnumerateVirtualDiskMetadata` - List metadata keys
  - [ ] `DeleteVirtualDiskMetadata` - Remove metadata

- [ ] **Advanced Operations**
  - [ ] `GetStorageDependencyInformation` - Dependency chain for differencing disks
  - [ ] `AddVirtualDiskParent` - Add parent to differencing disk
  - [ ] `ForkVirtualDisk` / `CompleteForkVirtualDisk` - Fork operations
  - [ ] `MirrorVirtualDisk` / `BreakMirrorVirtualDisk` - Mirror operations
  - [ ] `RawSCSIVirtualDisk` - Send raw SCSI commands
  - [ ] `GetAllAttachedVirtualDiskPhysicalPaths` - List all attached VHDs

### Windows Hypervisor Platform (`Win32_System_Hypervisor`)

Low-level hypervisor APIs for custom VM implementations:

- [ ] **Partition Management**
  - [ ] `WHvGetCapability` - Query hypervisor capabilities
  - [ ] `WHvCreatePartition` - Create VM partition
  - [ ] `WHvSetupPartition` - Initialize partition
  - [ ] `WHvDeletePartition` - Delete partition
  - [ ] `WHvGetPartitionProperty` - Query partition properties
  - [ ] `WHvSetPartitionProperty` - Set partition properties

- [ ] **Virtual Processor Management**
  - [ ] `WHvCreateVirtualProcessor` - Create vCPU
  - [ ] `WHvDeleteVirtualProcessor` - Delete vCPU
  - [ ] `WHvRunVirtualProcessor` - Execute vCPU
  - [ ] `WHvGetVirtualProcessorRegisters` - Read CPU registers
  - [ ] `WHvSetVirtualProcessorRegisters` - Write CPU registers
  - [ ] `WHvGetVirtualProcessorInterruptControllerState`
  - [ ] `WHvSetVirtualProcessorInterruptControllerState`

- [ ] **Memory Management**
  - [ ] `WHvMapGpaRange` - Map guest physical memory
  - [ ] `WHvUnmapGpaRange` - Unmap memory
  - [ ] `WHvTranslateGva` - Translate guest virtual address
  - [ ] `WHvQueryGpaRangeDirtyBitmap` - Dirty page tracking

### VM Saved State Dump Provider

For reading VMRS/VMGS/VSV files (requires `vmsavedstatedumpprovider.dll`):

- [ ] **I/O Operations**
  - [ ] `LoadSavedStateFile` - Open .vmrs/.vsv file
  - [ ] `LoadSavedStateFiles` - Open multiple files
  - [ ] `LocateSavedStateFiles` - Find saved state for VM
  - [ ] `ReleaseSavedStateFiles` - Release provider

- [ ] **Query Operations**
  - [ ] `GetVpCount` - Get vCPU count
  - [ ] `GetArchitecture` - x86/x64/ARM64
  - [ ] `GetPagingMode` - Memory paging mode
  - [ ] `GetRegisterValue` - Read CPU register from saved state
  - [ ] `ReadGuestPhysicalAddress` - Read guest memory
  - [ ] `ReadGuestRawSavedMemory` - Read raw memory
  - [ ] `GuestVirtualAddressToPhysicalAddress` - Address translation
  - [ ] `GuestPhysicalAddressToRawSavedMemoryOffset` - Memory offset
  - [ ] `GetGuestPhysicalMemoryChunks` - Memory layout
  - [ ] `GetGuestRawSavedMemorySize` - Total memory size
  - [ ] `ApplyPendingSavedStateFileReplay` - Apply replay logs

### HCS Schema Extensions

- [ ] **VM Configuration**
  - [ ] Gen1 VM configuration schema
  - [ ] Network adapter configuration
  - [ ] GPU-P (GPU partitioning) configuration
  - [ ] Storage QoS configuration

- [ ] **Container Support**
  - [ ] Windows container configuration
  - [ ] Process isolation configuration
  - [ ] Hyper-V isolation configuration

### Notes

- VMCX/VMGS files use proprietary binary format - no direct API available
- Use HCS APIs or PowerShell for VM configuration changes
- Some APIs require Windows 10/11 or Server 2016+ specific versions
- `vmsavedstatedumpprovider.dll` requires Windows 10 SDK 10.0.18362.0+
