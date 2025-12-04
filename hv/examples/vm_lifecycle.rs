//! Example: VM lifecycle management (create, start, stop, snapshot, delete)

#[cfg(windows)]
use hv::{HyperV, SnapshotType, VmGeneration, VmState};
#[cfg(windows)]
use std::env;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  list                    - List all VMs");
        eprintln!("  info <vm_name>          - Show VM details");
        eprintln!("  start <vm_name>         - Start a VM");
        eprintln!("  stop <vm_name>          - Stop a VM (graceful)");
        eprintln!("  force-stop <vm_name>    - Force stop a VM");
        eprintln!("  pause <vm_name>         - Pause a VM");
        eprintln!("  resume <vm_name>        - Resume a paused VM");
        eprintln!("  save <vm_name>          - Save VM state");
        eprintln!("  snapshot <vm_name> <snapshot_name> - Create a snapshot");
        eprintln!("  restore <vm_name> <snapshot_name>  - Restore a snapshot");
        eprintln!("  create <vm_name> <memory_mb> [vhd_gb] - Create a new Gen2 VM with VHD");
        eprintln!("  delete <vm_name>        - Delete a VM");
        std::process::exit(1);
    }

    let hyperv = HyperV::new()?;
    let command = args[1].as_str();

    match command {
        "list" => {
            let mut vms = hyperv.list_vms()?;
            println!(
                "{:<30} {:<15} {:>8} {:>10}",
                "Name", "State", "CPUs", "Memory"
            );
            println!("{}", "-".repeat(70));
            for vm in &mut vms {
                let state = vm.state()?;
                let cpu = vm.cpu_count().unwrap_or(0);
                let mem = vm.memory_mb().unwrap_or(0);
                println!(
                    "{:<30} {:<15} {:>8} {:>7} MB",
                    vm.name(),
                    format!("{:?}", state),
                    cpu,
                    mem
                );
            }
        }

        "info" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let mut vm = hyperv.get_vm(vm_name)?;

            println!("VM: {}", vm.name());
            println!("ID: {}", vm.id());
            println!("State: {:?}", vm.state()?);
            println!("CPUs: {}", vm.cpu_count()?);
            println!("Memory: {} MB", vm.memory_mb()?);

            println!();
            println!("Snapshots:");
            let snapshots = hyperv.list_snapshots(vm_name)?;
            if snapshots.is_empty() {
                println!("  (none)");
            } else {
                for snap in &snapshots {
                    let created = snap.creation_time().unwrap_or_default();
                    println!("  - {} ({})", snap.name(), created);
                }
            }
        }

        "start" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let mut vm = hyperv.get_vm(vm_name)?;
            println!("Starting VM '{}'...", vm_name);
            vm.start()?;
            println!("VM started");
        }

        "stop" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let mut vm = hyperv.get_vm(vm_name)?;
            println!("Stopping VM '{}' (graceful)...", vm_name);
            vm.stop()?;
            println!("VM stopped");
        }

        "force-stop" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let mut vm = hyperv.get_vm(vm_name)?;
            println!("Force stopping VM '{}'...", vm_name);
            vm.force_stop()?;
            println!("VM force stopped");
        }

        "pause" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let mut vm = hyperv.get_vm(vm_name)?;
            println!("Pausing VM '{}'...", vm_name);
            vm.pause()?;
            println!("VM paused");
        }

        "resume" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let mut vm = hyperv.get_vm(vm_name)?;
            println!("Resuming VM '{}'...", vm_name);
            vm.resume()?;
            println!("VM resumed");
        }

        "save" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let mut vm = hyperv.get_vm(vm_name)?;
            println!("Saving VM '{}' state...", vm_name);
            vm.save()?;
            println!("VM state saved");
        }

        "snapshot" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let snap_name = args.get(3).ok_or("Missing snapshot name")?;
            println!("Creating snapshot '{}' for VM '{}'...", snap_name, vm_name);
            let snapshot = hyperv.create_snapshot(vm_name, snap_name, SnapshotType::Standard)?;
            println!("Snapshot created: {}", snapshot.name());
        }

        "restore" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let snap_name = args.get(3).ok_or("Missing snapshot name")?;
            println!("Restoring snapshot '{}' for VM '{}'...", snap_name, vm_name);
            let snapshot = hyperv.get_snapshot(vm_name, snap_name)?;
            snapshot.apply()?;
            println!("Snapshot restored");
        }

        "create" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let memory_mb: u64 = args
                .get(3)
                .ok_or("Missing memory size")?
                .parse()
                .map_err(|_| "Invalid memory size")?;
            let vhd_size_gb: u64 = args.get(4).map(|s| s.parse().unwrap_or(64)).unwrap_or(64);

            // Default VHD path
            let vhd_path = format!("C:\\Hyper-V\\Virtual Hard Disks\\{}.vhdx", vm_name);
            let vhd_size_bytes = vhd_size_gb * 1024 * 1024 * 1024;

            println!(
                "Creating VM '{}' with {} MB memory, {}GB disk...",
                vm_name, memory_mb, vhd_size_gb
            );
            let vm = hyperv.create_vm(
                vm_name,
                memory_mb,
                2,
                VmGeneration::Gen2,
                &vhd_path,
                vhd_size_bytes,
                None, // switch_name
            )?;
            println!("VM created: {} ({})", vm.name(), vm.id());
            println!("VHD: {}", vhd_path);
        }

        "delete" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            print!("Are you sure you want to delete VM '{}'? [y/N] ", vm_name);

            // Simple confirmation (in real code, use proper input handling)
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "y" {
                println!("Deleting VM '{}'...", vm_name);
                hyperv.delete_vm(vm_name)?;
                println!("VM deleted");
            } else {
                println!("Cancelled");
            }
        }

        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Hyper-V installed.");
}
