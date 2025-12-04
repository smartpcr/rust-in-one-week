//! Control VM power state (start, stop, pause, resume, save).
//!
//! Run with: cargo run --example vm_power -- <vm_name> <action>
//! Actions: start, stop, pause, resume, save, reset
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{HyperV, Result, ShutdownType};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: vm_power <vm_name> <action>");
        println!("Actions: start, stop, force-stop, pause, resume, save, reset");
        return Ok(());
    }

    let vm_name = &args[1];
    let action = &args[2];

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    println!("Finding VM '{}'...", vm_name);
    let mut vm = hyperv.get_vm(vm_name)?;

    println!("Current state: {:?}", vm.state());

    match action.as_str() {
        "start" => {
            println!("Starting VM...");
            vm.start()?;
            println!("VM started successfully!");
        }
        "stop" => {
            println!("Stopping VM (graceful)...");
            vm.stop(ShutdownType::Graceful)?;
            println!("VM stopped successfully!");
        }
        "force-stop" => {
            println!("Stopping VM (force)...");
            vm.stop(ShutdownType::Force)?;
            println!("VM stopped successfully!");
        }
        "pause" => {
            println!("Pausing VM...");
            vm.pause()?;
            println!("VM paused successfully!");
        }
        "resume" => {
            println!("Resuming VM...");
            vm.resume()?;
            println!("VM resumed successfully!");
        }
        "save" => {
            println!("Saving VM state...");
            vm.save()?;
            println!("VM state saved successfully!");
        }
        "reset" => {
            println!("Resetting VM...");
            vm.reset()?;
            println!("VM reset successfully!");
        }
        _ => {
            println!("Unknown action: {}", action);
            println!("Valid actions: start, stop, force-stop, pause, resume, save, reset");
        }
    }

    vm.refresh()?;
    println!("New state: {:?}", vm.state());

    Ok(())
}
