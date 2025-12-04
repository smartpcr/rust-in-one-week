//! Attach a VHD to a virtual machine.
//!
//! Run with: cargo run --example attach_disk -- <vm_name> <vhd_path>
//! Example: cargo run --example attach_disk -- TestVM C:\VMs\disk.vhdx
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{ControllerType, DiskAttachment, HyperV, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: attach_disk <vm_name> <vhd_path> [scsi|ide] [controller_num] [location]");
        println!("Example: attach_disk TestVM C:\\VMs\\disk.vhdx scsi 0 0");
        return Ok(());
    }

    let vm_name = &args[1];
    let vhd_path = &args[2];
    let controller_type = if args.len() > 3 && args[3] == "ide" {
        ControllerType::Ide
    } else {
        ControllerType::Scsi
    };
    let controller_num: u32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let location: u32 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    println!("Finding VM '{}'...", vm_name);
    let vm = hyperv.get_vm(vm_name)?;

    println!("Attaching VHD...");
    println!("  VHD Path:    {}", vhd_path);
    println!("  Controller:  {:?} #{}", controller_type, controller_num);
    println!("  Location:    {}", location);

    let attachment = DiskAttachment::new(vhd_path)
        .controller_type(controller_type)
        .controller_number(controller_num)
        .controller_location(location);

    hyperv.attach_vhd(&vm, &attachment)?;

    println!("\nVHD attached successfully!");

    Ok(())
}
