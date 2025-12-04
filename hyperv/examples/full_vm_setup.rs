//! Complete example: Create a VM with disk, network, and ISO.
//!
//! Run with: cargo run --example full_vm_setup -- <vm_name> <vhd_path> [iso_path] [switch_name]
//! Example: cargo run --example full_vm_setup -- TestVM C:\VMs\TestVM\disk.vhdx C:\ISOs\windows.iso "Default Switch"
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use std::path::Path;
use windows_hyperv::{
    ControllerType, DiskAttachment, Generation, HyperV, IsoAttachment,
    NetworkAdapterSettings, Result, VhdSettings, VmSettings,
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: full_vm_setup <vm_name> <vhd_path> [iso_path] [switch_name]");
        println!();
        println!("Creates a complete Gen2 VM with:");
        println!("  - 4 GB RAM (dynamic 512 MB - 8 GB)");
        println!("  - 2 virtual CPUs");
        println!("  - Secure Boot enabled");
        println!("  - A new VHDX at the specified path (or attached if exists)");
        println!("  - ISO mounted (if provided)");
        println!("  - Network adapter connected to switch (if provided)");
        println!();
        println!("Example:");
        println!("  full_vm_setup MyVM C:\\VMs\\MyVM\\disk.vhdx C:\\ISOs\\win11.iso \"Default Switch\"");
        return Ok(());
    }

    let vm_name = &args[1];
    let vhd_path = &args[2];
    let iso_path = args.get(3).map(|s| s.as_str());
    let switch_name = args.get(4).map(|s| s.as_str());

    println!("=== Hyper-V VM Setup ===\n");
    println!("VM Name:     {}", vm_name);
    println!("VHD Path:    {}", vhd_path);
    if let Some(iso) = iso_path {
        println!("ISO Path:    {}", iso);
    }
    if let Some(switch) = switch_name {
        println!("Switch:      {}", switch);
    }
    println!();

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    // Check if VM already exists
    if hyperv.get_vm(vm_name).is_ok() {
        println!("ERROR: VM '{}' already exists!", vm_name);
        return Ok(());
    }

    // Step 1: Create the VM
    println!("\n[1/5] Creating VM...");
    let settings = VmSettings::builder()
        .name(vm_name)
        .generation(Generation::Gen2)
        .memory_mb(4096)
        .processor_count(2)
        .dynamic_memory(true)
        .dynamic_memory_min_mb(512)
        .dynamic_memory_max_mb(8192)
        .secure_boot(true)
        .notes(&format!("Created by full_vm_setup example"))
        .build()?;

    let vm = hyperv.create_vm(&settings)?;
    println!("       VM created: {} ({})", vm.name(), vm.id());

    // Step 2: Create or attach VHD
    println!("\n[2/5] Setting up storage...");
    if !Path::new(vhd_path).exists() {
        println!("       Creating new 100 GB VHDX...");
        let vhd_settings = VhdSettings::builder()
            .path(vhd_path)
            .size_gb(100)
            .build()?;

        hyperv.vhd().create(&vhd_settings)?;
        println!("       VHD created: {}", vhd_path);
    } else {
        println!("       Using existing VHD: {}", vhd_path);
    }

    println!("       Attaching VHD to SCSI controller...");
    let disk = DiskAttachment::new(vhd_path)
        .controller_type(ControllerType::Scsi)
        .controller_number(0)
        .controller_location(0);

    hyperv.attach_vhd(&vm, &disk)?;
    println!("       VHD attached successfully");

    // Step 3: Mount ISO if provided
    if let Some(iso) = iso_path {
        println!("\n[3/5] Mounting ISO...");
        let iso_attachment = IsoAttachment::new(iso)
            .controller_type(ControllerType::Scsi)
            .controller_number(0)
            .controller_location(1);

        hyperv.mount_iso(&vm, &iso_attachment)?;
        println!("       ISO mounted: {}", iso);
    } else {
        println!("\n[3/5] Skipping ISO (not provided)");
    }

    // Step 4: Add network adapter if switch provided
    if let Some(switch) = switch_name {
        println!("\n[4/5] Configuring network...");

        let switch_obj = hyperv.get_switch(switch)?;

        let adapter_settings = NetworkAdapterSettings::builder()
            .name("Network Adapter")
            .dynamic_mac(true)
            .build()?;

        let adapter = hyperv.add_network_adapter(&vm, &adapter_settings)?;
        hyperv.connect_adapter_to_switch(&vm, &adapter, &switch_obj)?;

        println!("       Network adapter connected to: {}", switch);
    } else {
        println!("\n[4/5] Skipping network (no switch provided)");
    }

    // Step 5: Summary
    println!("\n[5/5] Setup complete!");
    println!();
    println!("=== VM Summary ===");
    println!("Name:        {}", vm.name());
    println!("ID:          {}", vm.id());
    println!("Generation:  {}", vm.generation());
    println!("State:       {:?}", vm.state());
    println!("Memory:      {} MB (dynamic: 512-8192 MB)", vm.memory_mb()?);
    println!("Processors:  {}", vm.processor_count()?);
    println!();
    println!("To start the VM:");
    println!("  cargo run --example vm_power -- {} start", vm_name);

    Ok(())
}
