//! Add a network adapter to a VM and connect to a switch.
//!
//! Run with: cargo run --example add_network -- <vm_name> <switch_name>
//! Example: cargo run --example add_network -- TestVM "Default Switch"
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{HyperV, NetworkAdapterSettings, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: add_network <vm_name> <switch_name>");
        println!("Example: add_network TestVM \"Default Switch\"");
        println!("\nAvailable switches:");

        let hyperv = HyperV::connect()?;
        for switch in hyperv.list_switches()? {
            println!("  - {}", switch.name());
        }
        return Ok(());
    }

    let vm_name = &args[1];
    let switch_name = &args[2];

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    println!("Finding VM '{}'...", vm_name);
    let vm = hyperv.get_vm(vm_name)?;

    println!("Finding switch '{}'...", switch_name);
    let switch = hyperv.get_switch(switch_name)?;

    println!("Adding network adapter...");
    let adapter_settings = NetworkAdapterSettings::builder()
        .name("Network Adapter")
        .switch(switch_name)
        .dynamic_mac(true)
        .build()?;

    let adapter = hyperv.add_network_adapter(&vm, &adapter_settings)?;

    println!("Connecting adapter to switch...");
    hyperv.connect_adapter_to_switch(&vm, &adapter, &switch)?;

    println!("\nNetwork adapter added and connected successfully!");
    println!("  Adapter: {}", adapter.name);
    println!("  Switch:  {}", switch.name());

    Ok(())
}
