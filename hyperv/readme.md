# windows-hyperv

Typed Hyper-V management API for Windows.

This crate provides strongly-typed Rust bindings for Hyper-V VM management operations, built on top of the WMI-based Hyper-V management APIs.

## Features

- **Type-safe VM operations**: Create, start, stop, delete VMs with compile-time type checking
- **Builder pattern**: Configure VMs with validated settings
- **Proper error handling**: Typed errors instead of generic WMI failures
- **Full VM lifecycle**: Memory, processor, storage, network, and checkpoint management

## Requirements

- Windows 10/11 or Windows Server 2016+
- Hyper-V feature enabled
- **Administrator privileges** (run terminal as Administrator)

## Examples

All examples require Administrator privileges. Run from an elevated terminal.

### List VMs

```bash
cargo run --example list_vms
```

### Create a VM

```bash
cargo run --example create_vm -- MyTestVM
```

### Control VM Power State

```bash
cargo run --example vm_power -- MyTestVM start
cargo run --example vm_power -- MyTestVM pause
cargo run --example vm_power -- MyTestVM resume
cargo run --example vm_power -- MyTestVM stop
cargo run --example vm_power -- MyTestVM force-stop
```

### Create a VHD

```bash
cargo run --example create_vhd -- C:\VMs\disk.vhdx 100
cargo run --example create_vhd -- C:\VMs\disk.vhdx 50 fixed
```

### Attach VHD to VM

```bash
cargo run --example attach_disk -- MyTestVM C:\VMs\disk.vhdx
```

### Mount ISO

```bash
cargo run --example mount_iso -- MyTestVM C:\ISOs\windows.iso
```

### List Virtual Switches

```bash
cargo run --example list_switches
```

### Add Network Adapter

```bash
cargo run --example add_network -- MyTestVM "Default Switch"
```

### Manage Checkpoints

```bash
cargo run --example checkpoints -- MyTestVM list
cargo run --example checkpoints -- MyTestVM create "Before Update"
cargo run --example checkpoints -- MyTestVM apply "Before Update"
cargo run --example checkpoints -- MyTestVM delete "Before Update"
```

### Delete VM

```bash
cargo run --example delete_vm -- MyTestVM
cargo run --example delete_vm -- MyTestVM --force  # Stop first if running
```

### Full VM Setup (Complete Example)

Creates a Gen2 VM with disk, network, and optional ISO in one command:

```bash
cargo run --example full_vm_setup -- MyTestVM C:\VMs\MyTestVM\disk.vhdx C:\ISOs\win11.iso "Default Switch"
```

## Usage in Code

```rust
use windows_hyperv::{HyperV, VmSettings, Generation, VhdSettings, DiskAttachment};

fn main() -> windows_hyperv::Result<()> {
    let hyperv = HyperV::connect()?;

    // List all VMs
    for vm in hyperv.list_vms()? {
        println!("{}: {:?}", vm.name(), vm.state());
    }

    // Create a new VM with validation
    let settings = VmSettings::builder()
        .name("MyVM")
        .generation(Generation::Gen2)
        .memory_mb(4096)
        .processor_count(2)
        .secure_boot(true)
        .build()?;  // Validates all settings

    let mut vm = hyperv.create_vm(&settings)?;

    // Create and attach a VHD
    let vhd = hyperv.vhd().create(
        &VhdSettings::builder()
            .path(r"C:\VMs\MyVM\disk.vhdx")
            .size_gb(100)
            .build()?
    )?;

    hyperv.attach_vhd(&vm, &DiskAttachment::new(&vhd.path))?;

    // Start the VM
    vm.start()?;

    Ok(())
}
```

## Validation

All settings are validated both at build time (required fields) and runtime:

```rust
// This won't compile - missing required fields
let settings = VmSettings::builder()
    .name("MyVM")
    .build()?;  // Error: missing generation, memory_mb, processor_count

// This fails at runtime - invalid combination
let settings = VmSettings::builder()
    .name("MyVM")
    .generation(Generation::Gen1)
    .memory_mb(4096)
    .processor_count(2)
    .secure_boot(true)  // Error: Secure Boot only available for Gen2
    .build()?;
```

## Error Handling

```rust
use windows_hyperv::{HyperV, Error, VmState};

match hyperv.get_vm("NonExistent") {
    Ok(vm) => println!("Found: {}", vm.name()),
    Err(Error::VmNotFound(name)) => println!("VM '{}' not found", name),
    Err(e) => println!("Error: {}", e),
}

// State validation prevents invalid operations
let mut vm = hyperv.get_vm("MyVM")?;
match vm.start() {
    Ok(()) => println!("Started"),
    Err(Error::InvalidState { current, .. }) => {
        println!("Cannot start VM in state {:?}", current);
    }
    Err(e) => return Err(e),
}
```
