//! Manage VM checkpoints (snapshots).
//!
//! Run with: cargo run --example checkpoints -- <vm_name> <action> [name]
//! Actions: list, create, apply, delete
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{CheckpointSettings, HyperV, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: checkpoints <vm_name> <action> [checkpoint_name]");
        println!("Actions:");
        println!("  list              - List all checkpoints");
        println!("  create <name>     - Create a new checkpoint");
        println!("  apply <name>      - Apply/restore a checkpoint");
        println!("  delete <name>     - Delete a checkpoint");
        return Ok(());
    }

    let vm_name = &args[1];
    let action = &args[2];

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    println!("Finding VM '{}'...", vm_name);
    let mut vm = hyperv.get_vm(vm_name)?;

    match action.as_str() {
        "list" => {
            let checkpoints = hyperv.list_checkpoints(&vm)?;

            if checkpoints.is_empty() {
                println!("\nNo checkpoints found for VM '{}'.", vm_name);
            } else {
                println!("\nCheckpoints for '{}':", vm_name);
                println!("{:<30} {:<25} {}", "NAME", "CREATED", "ID");
                println!("{}", "-".repeat(80));

                for cp in checkpoints {
                    println!(
                        "{:<30} {:<25} {}",
                        cp.name(),
                        cp.creation_time,
                        cp.id(),
                    );
                }
            }
        }
        "create" => {
            let name = args.get(3).map(|s| s.as_str()).unwrap_or("Checkpoint");

            println!("Creating checkpoint '{}'...", name);

            let settings = CheckpointSettings::builder()
                .name(name)
                .notes("Created by windows-hyperv example")
                .build()?;

            let checkpoint = hyperv.create_checkpoint(&vm, &settings)?;

            println!("\nCheckpoint created successfully!");
            println!("  Name: {}", checkpoint.name());
            println!("  ID:   {}", checkpoint.id());
        }
        "apply" => {
            let name = args.get(3).expect("Checkpoint name required");

            println!("Looking for checkpoint '{}'...", name);

            let checkpoints = hyperv.list_checkpoints(&vm)?;
            let checkpoint = checkpoints
                .iter()
                .find(|cp| cp.name() == name)
                .expect("Checkpoint not found");

            println!("Applying checkpoint...");
            println!("Note: VM must be off to apply a checkpoint.");

            hyperv.apply_checkpoint(&mut vm, checkpoint)?;

            println!("\nCheckpoint applied successfully!");
        }
        "delete" => {
            let name = args.get(3).expect("Checkpoint name required");

            println!("Looking for checkpoint '{}'...", name);

            let checkpoints = hyperv.list_checkpoints(&vm)?;
            let checkpoint = checkpoints
                .iter()
                .find(|cp| cp.name() == name)
                .expect("Checkpoint not found");

            println!("Deleting checkpoint...");
            hyperv.delete_checkpoint(checkpoint)?;

            println!("\nCheckpoint deleted successfully!");
        }
        _ => {
            println!("Unknown action: {}", action);
            println!("Valid actions: list, create, apply, delete");
        }
    }

    Ok(())
}
