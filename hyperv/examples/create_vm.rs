//! Create a new Hyper-V virtual machine.
//!
//! Run with: cargo run --example create_vm -- <vm_name>
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{Generation, HyperV, Result, VmSettings};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let vm_name = if args.len() > 1 {
        &args[1]
    } else {
        "TestVM"
    };

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    // Check if VM already exists
    if hyperv.get_vm(vm_name).is_ok() {
        println!("VM '{}' already exists!", vm_name);
        return Ok(());
    }

    println!("Creating VM '{}'...", vm_name);

    let settings = VmSettings::builder()
        .name(vm_name)
        .generation(Generation::Gen2)
        .memory_mb(4096)
        .processor_count(2)
        .secure_boot(true)
        .dynamic_memory(true)
        .dynamic_memory_min_mb(512)
        .dynamic_memory_max_mb(8192)
        .notes("Created by windows-hyperv example")
        .build()?;

    let vm = hyperv.create_vm(&settings)?;

    println!("\nVM created successfully!");
    println!("  Name:       {}", vm.name());
    println!("  ID:         {}", vm.id());
    println!("  State:      {:?}", vm.state());
    println!("  Generation: {}", vm.generation());

    Ok(())
}
