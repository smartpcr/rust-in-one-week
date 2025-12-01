//! Example: Move a Hyper-V VM to another cluster node
//!
//! In Hyper-V clusters, each VM is typically represented as a cluster group.
//! Moving a VM to another node is done by moving its cluster group.

#[cfg(windows)]
use clus::{Cluster, GroupState};
#[cfg(windows)]
use std::env;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <vm_name> <target_node>", args[0]);
        eprintln!("Example: {} MyVM Node2", args[0]);
        std::process::exit(1);
    }

    let vm_name = &args[1];
    let target_node_name = &args[2];

    let cluster = Cluster::open(None)?;
    println!("Connected to cluster: {}", cluster.name()?);

    // Open the VM's cluster group
    let vm_group = cluster.open_group(vm_name)?;
    let (current_state, current_owner) = vm_group.state()?;

    println!(
        "VM '{}' is currently {:?} on {}",
        vm_name,
        current_state,
        current_owner.as_deref().unwrap_or("unknown")
    );

    // Check if already on target node
    if current_owner.as_deref() == Some(target_node_name) {
        println!("VM is already on the target node.");
        return Ok(());
    }

    // Open the target node
    let target_node = cluster.open_node(target_node_name)?;
    println!(
        "Target node '{}' state: {:?}",
        target_node.name(),
        target_node.state()
    );

    // Perform live migration
    println!("Moving VM to {}...", target_node_name);
    vm_group.move_to(&target_node)?;

    // Wait and check result
    std::thread::sleep(std::time::Duration::from_secs(2));

    let (new_state, new_owner) = vm_group.state()?;
    println!(
        "VM '{}' is now {:?} on {}",
        vm_name,
        new_state,
        new_owner.as_deref().unwrap_or("unknown")
    );

    match new_state {
        GroupState::Online if new_owner.as_deref() == Some(target_node_name) => {
            println!("✓ Migration completed successfully!");
        }
        GroupState::Pending => {
            println!("… Migration still in progress");
        }
        _ => {
            println!("⚠ Migration may have failed, please check cluster manager");
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Failover Clustering installed.");
}
