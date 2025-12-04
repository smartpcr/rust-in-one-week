//! Delete a Hyper-V virtual machine.
//!
//! Run with: cargo run --example delete_vm -- <vm_name>
//! Requires: Administrator privileges, Hyper-V enabled
//! Note: VM must be stopped before deletion

use std::env;
use windows_hyperv::{HyperV, Result, ShutdownType, VmState};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: delete_vm <vm_name> [--force]");
        println!("  --force: Stop VM if running before deleting");
        return Ok(());
    }

    let vm_name = &args[1];
    let force = args.get(2).map(|s| s == "--force").unwrap_or(false);

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    println!("Finding VM '{}'...", vm_name);
    let mut vm = hyperv.get_vm(vm_name)?;

    println!("Current state: {:?}", vm.state());

    // Stop VM if running and --force is specified
    if vm.state() != VmState::Off {
        if force {
            println!("Stopping VM (force)...");
            vm.stop(ShutdownType::Force)?;
            vm.refresh()?;

            // Wait for VM to stop
            let mut attempts = 0;
            while vm.state() != VmState::Off && attempts < 30 {
                std::thread::sleep(std::time::Duration::from_secs(1));
                vm.refresh()?;
                attempts += 1;
            }

            if vm.state() != VmState::Off {
                println!("Failed to stop VM within 30 seconds");
                return Ok(());
            }
        } else {
            println!("VM is not stopped. Use --force to stop and delete.");
            return Ok(());
        }
    }

    println!("Deleting VM '{}'...", vm_name);
    hyperv.delete_vm(&vm)?;

    println!("\nVM '{}' deleted successfully!", vm_name);

    Ok(())
}
