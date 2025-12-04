//! Mount an ISO file to a virtual machine.
//!
//! Run with: cargo run --example mount_iso -- <vm_name> <iso_path>
//! Example: cargo run --example mount_iso -- TestVM C:\ISOs\windows.iso
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{ControllerType, HyperV, IsoAttachment, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: mount_iso <vm_name> <iso_path>");
        println!("Example: mount_iso TestVM C:\\ISOs\\windows.iso");
        return Ok(());
    }

    let vm_name = &args[1];
    let iso_path = &args[2];

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    println!("Finding VM '{}'...", vm_name);
    let vm = hyperv.get_vm(vm_name)?;

    println!("Mounting ISO...");
    println!("  ISO Path: {}", iso_path);

    // For Gen1, use IDE controller 1 (secondary)
    // For Gen2, use SCSI controller
    let attachment = if vm.generation() == windows_hyperv::Generation::Gen2 {
        IsoAttachment::new(iso_path)
            .controller_type(ControllerType::Scsi)
            .controller_number(0)
            .controller_location(1)
    } else {
        IsoAttachment::new(iso_path)
            .controller_type(ControllerType::Ide)
            .controller_number(1)
            .controller_location(0)
    };

    hyperv.mount_iso(&vm, &attachment)?;

    println!("\nISO mounted successfully!");

    Ok(())
}
