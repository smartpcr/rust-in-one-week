//! Example: List VMs using WMI-based interface
//!
//! This example demonstrates the WMI-based Hyper-V management interface
//! which provides more detailed VM information than the HCS-based interface.
//!
//! Run with: cargo run -p hv --example wmi_vms

#[cfg(windows)]
use hv::HyperVWmi;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Hyper-V WMI...\n");

    let hv = HyperVWmi::new()?;

    // List all VMs
    println!("=== Virtual Machines ===\n");
    let vms = hv.list_vms()?;

    if vms.is_empty() {
        println!("No VMs found.");
    } else {
        for vm in &vms {
            println!("Name: {}", vm.name);
            println!("  ID: {}", vm.id);
            println!("  State: {}", vm.state_string());
            if let Some(gen) = vm.generation {
                println!("  Generation: {}", gen);
            }
            if let Some(mem) = vm.memory_mb {
                println!("  Memory: {} MB", mem);
            }
            if let Some(cpu) = vm.processor_count {
                println!("  CPUs: {}", cpu);
            }
            if let Some(ref notes) = vm.notes {
                if !notes.is_empty() {
                    println!("  Notes: {}", notes);
                }
            }
            println!();
        }
    }

    // List switches
    println!("=== Virtual Switches ===\n");
    let switches = hv.list_switches()?;

    if switches.is_empty() {
        println!("No switches found.");
    } else {
        for switch in &switches {
            println!("Name: {}", switch.name);
            println!("  ID: {}", switch.id);
            if let Some(ref desc) = switch.switch_type {
                println!("  Type: {}", desc);
            }
            println!();
        }
    }

    // If there are VMs, show snapshots for the first one
    if let Some(vm) = vms.first() {
        println!("=== Snapshots for {} ===\n", vm.name);
        match hv.list_snapshots(&vm.name) {
            Ok(snapshots) => {
                if snapshots.is_empty() {
                    println!("No snapshots found.");
                } else {
                    for snapshot in &snapshots {
                        println!("Name: {}", snapshot.name);
                        println!("  ID: {}", snapshot.id);
                        if let Some(ref time) = snapshot.creation_time {
                            println!("  Created: {}", time);
                        }
                        println!();
                    }
                }
            }
            Err(e) => {
                println!("Error listing snapshots: {}", e);
            }
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Hyper-V installed.");
}
