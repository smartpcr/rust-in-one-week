//! List all Hyper-V virtual switches.
//!
//! Run with: cargo run --example list_switches
//! Requires: Administrator privileges, Hyper-V enabled

use windows_hyperv::{HyperV, Result};

fn main() -> Result<()> {
    println!("Connecting to Hyper-V...\n");

    let hyperv = HyperV::connect()?;

    let switches = hyperv.list_switches()?;

    if switches.is_empty() {
        println!("No virtual switches found.");
        return Ok(());
    }

    println!("{:<30} {:<15} {:<40}", "NAME", "TYPE", "ID");
    println!("{}", "-".repeat(85));

    for switch in switches {
        println!(
            "{:<30} {:<15} {:<40}",
            switch.name(),
            format!("{:?}", switch.switch_type()),
            switch.id(),
        );
    }

    Ok(())
}
