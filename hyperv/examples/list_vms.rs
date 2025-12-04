//! List all Hyper-V virtual machines.
//!
//! Run with: cargo run --example list_vms
//! Requires: Administrator privileges, Hyper-V enabled

use windows_hyperv::{HyperV, Result};

fn main() -> Result<()> {
    println!("Connecting to Hyper-V...\n");

    let hyperv = HyperV::connect()?;

    let vms = hyperv.list_vms()?;

    if vms.is_empty() {
        println!("No virtual machines found.");
        return Ok(());
    }

    println!("{:<30} {:<15} {:<12} {:>10} {:>6}", "NAME", "STATE", "GENERATION", "MEMORY", "CPUs");
    println!("{}", "-".repeat(75));

    for vm in vms {
        let memory = vm.memory_mb().unwrap_or(0);
        let cpus = vm.processor_count().unwrap_or(0);

        println!(
            "{:<30} {:<15} {:<12} {:>7} MB {:>6}",
            vm.name(),
            format!("{}", vm.state()),
            format!("{}", vm.generation()),
            memory,
            cpus,
        );
    }

    Ok(())
}
