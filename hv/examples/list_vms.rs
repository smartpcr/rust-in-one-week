//! Example: List all Hyper-V VMs with their status

#[cfg(windows)]
use hv::{HyperV, VmState};

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create HyperV management interface
    let hyperv = HyperV::new()?;

    // Get host info
    let host = hyperv.host_info()?;
    println!("Hyper-V Host: {}", host.computer_name);
    println!(
        "CPUs: {}, Memory: {} GB",
        host.logical_processor_count,
        host.memory_capacity_bytes / 1024 / 1024 / 1024
    );
    println!("VM Path: {}", host.vm_path);
    println!("VHD Path: {}", host.vhd_path);
    println!();

    // List all VMs
    println!("=== Virtual Machines ===");
    let mut vms = hyperv.list_vms()?;

    if vms.is_empty() {
        println!("  No VMs found");
    } else {
        for vm in &mut vms {
            let state = vm.state()?;
            let status = match state {
                VmState::Running => "Running",
                VmState::Off => "Off",
                VmState::Saved => "Saved",
                VmState::Paused => "Paused",
                VmState::Starting => "Starting",
                VmState::Stopping => "Stopping",
                VmState::Saving => "Saving",
                VmState::Pausing => "Pausing",
                VmState::Resuming => "Resuming",
                _ => "Unknown",
            };

            println!("  {} - {}", vm.name(), status);

            if state.is_running() {
                let cpu = vm.cpu_count().unwrap_or(0);
                let mem = vm.memory_mb().unwrap_or(0);
                println!("    CPUs: {}, Memory: {} MB", cpu, mem);
            }
        }
    }
    println!();

    // List all switches
    println!("=== Virtual Switches ===");
    let switches = hyperv.list_switches()?;

    if switches.is_empty() {
        println!("  No switches found");
    } else {
        for switch in &switches {
            let switch_type = switch.switch_type()?;
            println!("  {} - {:?}", switch.name(), switch_type);
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Hyper-V installed.");
}
