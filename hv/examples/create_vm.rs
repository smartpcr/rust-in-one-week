//! Example: Create VMs with OS disk
//!
//! Demonstrates different ways to create VMs:
//! 1. Empty VM with ISO for manual installation
//! 2. VM from existing VHDX
//! 3. VM with Windows from ISO (automated)

#[cfg(windows)]
use hv::{FileSystem, HyperV, PartitionStyle, VhdType, VmGeneration};
#[cfg(windows)]
use std::env;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let hyperv = HyperV::new()?;
    let command = args[1].as_str();

    match command {
        // =================================================================
        // ISO Information
        // =================================================================
        "iso-info" => {
            let iso_path = args.get(2).ok_or("Missing ISO path")?;

            println!("Scanning ISO for Windows editions...\n");
            let editions = hyperv.get_windows_editions(iso_path)?;

            if editions.is_empty() {
                println!("No Windows editions found in ISO.");
                println!("This might not be a valid Windows installation ISO.");
            } else {
                println!("Available Windows editions:\n");
                println!("{:<6} {:<40} {:>12}", "Index", "Name", "Size");
                println!("{}", "-".repeat(60));
                for edition in &editions {
                    let size_gb = edition.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                    println!(
                        "{:<6} {:<40} {:>10.1} GB",
                        edition.index, edition.name, size_gb
                    );
                }
                println!();
                println!("Use the Index number with 'from-iso' command.");
            }
        }

        // =================================================================
        // Create VM from ISO (Full Automated)
        // =================================================================
        "from-iso" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let iso_path = args.get(3).ok_or("Missing ISO path")?;
            let vhdx_path = args.get(4).ok_or("Missing VHDX path")?;

            let size_gb: u64 = args.get(5).map(|s| s.parse().unwrap_or(64)).unwrap_or(64);
            let memory_mb: u64 = args
                .get(6)
                .map(|s| s.parse().unwrap_or(4096))
                .unwrap_or(4096);
            let edition_index: u32 = args.get(7).map(|s| s.parse().unwrap_or(1)).unwrap_or(1);

            println!("Creating Windows VM from ISO...\n");
            println!("  VM Name:    {}", vm_name);
            println!("  ISO:        {}", iso_path);
            println!("  VHDX:       {}", vhdx_path);
            println!("  Size:       {} GB", size_gb);
            println!("  Memory:     {} MB", memory_mb);
            println!("  Edition:    Index {}", edition_index);
            println!();
            println!("This process may take 10-20 minutes...\n");

            hyperv.quick_create_windows_vm(
                vm_name,
                iso_path,
                vhdx_path,
                size_gb,
                memory_mb,
                2, // CPU count
                edition_index,
            )?;

            println!("VM created successfully!");
            println!();
            println!("The VM is ready to boot with Windows pre-installed.");
            println!(
                "Start it with: cargo run --example vm_lifecycle -- start {}",
                vm_name
            );
        }

        // =================================================================
        // Create Empty VM with ISO (Manual Installation)
        // =================================================================
        "with-iso" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let iso_path = args.get(3).ok_or("Missing ISO path")?;

            let size_gb: u64 = args.get(4).map(|s| s.parse().unwrap_or(64)).unwrap_or(64);
            let memory_mb: u64 = args
                .get(5)
                .map(|s| s.parse().unwrap_or(4096))
                .unwrap_or(4096);

            let vhdx_path = format!("C:\\Hyper-V\\Virtual Hard Disks\\{}.vhdx", vm_name);

            println!("Creating VM for manual OS installation...\n");

            // 1. Create empty VHDX
            println!("  Creating {}GB VHDX...", size_gb);
            let size_bytes = size_gb * 1024 * 1024 * 1024;
            hyperv.create_vhd(&vhdx_path, size_bytes, VhdType::Dynamic, None)?;

            // 2. Create VM with the VHDX
            println!("  Creating Gen2 VM...");
            let vm =
                hyperv.create_vm(vm_name, memory_mb, 2, VmGeneration::Gen2, Some(&vhdx_path))?;

            // 3. Add DVD drive and mount ISO
            println!("  Adding DVD drive...");
            hyperv.add_dvd_drive(vm_name)?;

            println!("  Mounting ISO...");
            hyperv.mount_iso(vm_name, iso_path)?;

            // 4. Set boot order to DVD first
            println!("  Setting boot order (DVD first)...");
            hyperv.set_boot_order(vm_name, &["DVD", "VHD"])?;

            println!();
            println!("VM '{}' created successfully!", vm_name);
            println!("  ID: {}", vm.id());
            println!("  VHDX: {}", vhdx_path);
            println!("  ISO: {} (mounted)", iso_path);
            println!();
            println!("Start the VM to begin OS installation:");
            println!("  cargo run --example vm_lifecycle -- start {}", vm_name);
        }

        // =================================================================
        // Create VM from Existing VHDX
        // =================================================================
        "from-vhdx" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let vhdx_path = args.get(3).ok_or("Missing VHDX path")?;

            let memory_mb: u64 = args
                .get(4)
                .map(|s| s.parse().unwrap_or(4096))
                .unwrap_or(4096);
            let cpu_count: u32 = args.get(5).map(|s| s.parse().unwrap_or(2)).unwrap_or(2);

            println!("Creating VM from existing VHDX...\n");

            let vm = hyperv.create_vm(
                vm_name,
                memory_mb,
                cpu_count,
                VmGeneration::Gen2,
                Some(vhdx_path),
            )?;

            println!("VM '{}' created successfully!", vm_name);
            println!("  ID: {}", vm.id());
            println!("  VHDX: {}", vhdx_path);
            println!("  Memory: {} MB", memory_mb);
            println!("  CPUs: {}", cpu_count);
        }

        // =================================================================
        // Create VHDX from ISO (disk only, no VM)
        // =================================================================
        "create-vhdx" => {
            let iso_path = args.get(2).ok_or("Missing ISO path")?;
            let vhdx_path = args.get(3).ok_or("Missing VHDX path")?;

            let size_gb: u64 = args.get(4).map(|s| s.parse().unwrap_or(64)).unwrap_or(64);
            let edition_index: u32 = args.get(5).map(|s| s.parse().unwrap_or(1)).unwrap_or(1);

            println!("Creating bootable VHDX from ISO...\n");
            println!("  ISO: {}", iso_path);
            println!("  VHDX: {}", vhdx_path);
            println!("  Size: {} GB", size_gb);
            println!("  Edition Index: {}", edition_index);
            println!();
            println!("This process may take 10-20 minutes...\n");

            hyperv.create_vhdx_from_iso(iso_path, vhdx_path, size_gb, edition_index)?;

            println!("VHDX created successfully!");
            println!();
            println!("Use it to create a VM:");
            println!(
                "  cargo run --example create_vm -- from-vhdx MyVM \"{}\"",
                vhdx_path
            );
        }

        // =================================================================
        // Create Empty Data Disk
        // =================================================================
        "create-data-disk" => {
            let vhdx_path = args.get(2).ok_or("Missing VHDX path")?;

            let size_gb: u64 = args.get(3).map(|s| s.parse().unwrap_or(100)).unwrap_or(100);
            let label = args.get(4).map(|s| s.as_str()).unwrap_or("Data");

            println!("Creating and initializing data disk...\n");

            // Create VHDX
            let size_bytes = size_gb * 1024 * 1024 * 1024;
            println!("  Creating {}GB VHDX...", size_gb);
            hyperv.create_vhd(vhdx_path, size_bytes, VhdType::Dynamic, None)?;

            // Initialize with NTFS
            println!("  Initializing with GPT and NTFS...");
            let drive_letter = hyperv.initialize_vhd(
                vhdx_path,
                PartitionStyle::Gpt,
                FileSystem::Ntfs,
                Some(label),
            )?;

            println!("  Mounted at {}:\\", drive_letter);

            // Dismount
            println!("  Dismounting...");
            hyperv.dismount_vhd(vhdx_path)?;

            println!();
            println!("Data disk created: {}", vhdx_path);
            println!();
            println!("Attach to a VM:");
            println!(
                "  cargo run --example create_vm -- attach-disk MyVM \"{}\"",
                vhdx_path
            );
        }

        // =================================================================
        // Attach Disk to VM
        // =================================================================
        "attach-disk" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let vhdx_path = args.get(3).ok_or("Missing VHDX path")?;

            println!("Attaching disk to VM '{}'...", vm_name);
            hyperv.add_hard_disk_drive(vm_name, vhdx_path)?;

            println!("Disk attached successfully.");
        }

        // =================================================================
        // Detach Disk from VM
        // =================================================================
        "detach-disk" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let controller_num: u32 = args
                .get(3)
                .ok_or("Missing controller number")?
                .parse()
                .map_err(|_| "Invalid controller number")?;
            let controller_loc: u32 = args
                .get(4)
                .ok_or("Missing controller location")?
                .parse()
                .map_err(|_| "Invalid controller location")?;

            println!("Detaching disk from VM '{}'...", vm_name);
            hyperv.remove_hard_disk_drive(vm_name, controller_num, controller_loc)?;

            println!("Disk detached successfully.");
        }

        // =================================================================
        // List VM Disks
        // =================================================================
        "list-disks" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;

            println!("Disks attached to VM '{}':\n", vm_name);

            // Hard disks
            let hard_disks = hyperv.get_hard_disk_drives(vm_name)?;
            if hard_disks.is_empty() {
                println!("  No hard disks attached.");
            } else {
                println!("Hard Disks:");
                for disk in &hard_disks {
                    println!(
                        "  [{} {}:{}] {}",
                        disk.controller_type,
                        disk.controller_number,
                        disk.controller_location,
                        disk.path.as_deref().unwrap_or("(no path)")
                    );
                }
            }

            // DVD drives
            println!();
            let dvd_drives = hyperv.get_dvd_drives(vm_name)?;
            if dvd_drives.is_empty() {
                println!("  No DVD drives.");
            } else {
                println!("DVD Drives:");
                for dvd in &dvd_drives {
                    let iso = dvd.path.as_deref().unwrap_or("(empty)");
                    println!(
                        "  [{} {}:{}] {}",
                        dvd.controller_type, dvd.controller_number, dvd.controller_location, iso
                    );
                }
            }
        }

        // =================================================================
        // ISO Operations
        // =================================================================
        "mount-iso" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let iso_path = args.get(3).ok_or("Missing ISO path")?;

            println!("Mounting ISO to VM '{}'...", vm_name);
            hyperv.mount_iso(vm_name, iso_path)?;
            println!("ISO mounted successfully.");
        }

        "eject-iso" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;

            println!("Ejecting ISO from VM '{}'...", vm_name);
            hyperv.eject_iso(vm_name)?;
            println!("ISO ejected successfully.");
        }

        "set-boot" => {
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let boot_order: Vec<&str> = args[3..].iter().map(|s| s.as_str()).collect();

            if boot_order.is_empty() {
                println!("Usage: set-boot <vm_name> <device1> [device2] ...");
                println!("  Devices: DVD, VHD, Network");
                std::process::exit(1);
            }

            println!("Setting boot order for VM '{}'...", vm_name);
            println!("  Order: {:?}", boot_order);
            hyperv.set_boot_order(vm_name, &boot_order)?;
            println!("Boot order set successfully.");
        }

        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn print_usage(program: &str) {
    eprintln!("Usage: {} <command> [args...]", program);
    eprintln!();
    eprintln!("VM Creation Commands:");
    eprintln!("  from-iso <name> <iso> <vhdx> [size_gb] [mem_mb] [edition]");
    eprintln!("                              - Create VM with Windows from ISO (automated)");
    eprintln!("  with-iso <name> <iso> [size_gb] [mem_mb]");
    eprintln!("                              - Create empty VM with ISO for manual install");
    eprintln!("  from-vhdx <name> <vhdx> [mem_mb] [cpus]");
    eprintln!("                              - Create VM from existing VHDX");
    eprintln!();
    eprintln!("Disk Creation Commands:");
    eprintln!("  iso-info <iso_path>         - List Windows editions in ISO");
    eprintln!("  create-vhdx <iso> <vhdx> [size_gb] [edition]");
    eprintln!("                              - Create bootable VHDX from ISO");
    eprintln!("  create-data-disk <vhdx> [size_gb] [label]");
    eprintln!("                              - Create empty data disk");
    eprintln!();
    eprintln!("Disk Management Commands:");
    eprintln!("  list-disks <vm_name>        - List VM's disks and DVDs");
    eprintln!("  attach-disk <vm> <vhdx>     - Attach VHDX to VM");
    eprintln!("  detach-disk <vm> <ctrl#> <loc#>  - Detach disk from VM");
    eprintln!();
    eprintln!("ISO Commands:");
    eprintln!("  mount-iso <vm_name> <iso>   - Mount ISO to VM");
    eprintln!("  eject-iso <vm_name>         - Eject ISO from VM");
    eprintln!("  set-boot <vm> <dev1> ...    - Set boot order (DVD, VHD, Network)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!();
    eprintln!("  # Quick create Windows VM from ISO");
    eprintln!(
        "  {} from-iso Win11VM \"C:\\ISOs\\Win11.iso\" \"C:\\VMs\\Win11.vhdx\" 64 4096",
        program
    );
    eprintln!();
    eprintln!("  # Create VM for manual installation");
    eprintln!(
        "  {} with-iso UbuntuVM \"C:\\ISOs\\ubuntu.iso\" 50 2048",
        program
    );
    eprintln!();
    eprintln!("  # Create VM from existing disk");
    eprintln!(
        "  {} from-vhdx DevVM \"C:\\VMs\\dev-template.vhdx\" 8192 4",
        program
    );
    eprintln!();
    eprintln!("  # Create data disk and attach to VM");
    eprintln!(
        "  {} create-data-disk \"C:\\VMs\\data.vhdx\" 500 \"Data\"",
        program
    );
    eprintln!("  {} attach-disk MyVM \"C:\\VMs\\data.vhdx\"", program);
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Hyper-V installed.");
}
