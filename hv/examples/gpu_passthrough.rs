//! Example: GPU passthrough configuration (GPU-P and DDA)
//!
//! Demonstrates how to configure GPU passthrough for VMs using both
//! GPU-P (partitioning) and DDA (discrete device assignment).

#[cfg(windows)]
use hv::HyperV;
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
        // GPU-P (Partitioning) Commands
        // =================================================================
        "gpup-add" => {
            // Add GPU-P to a VM
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let gpu_path = args.get(3).map(|s| s.as_str()); // Optional specific GPU

            println!("Adding GPU-P adapter to VM '{}'...", vm_name);

            // Configure VM for GPU-P first
            println!("  Configuring VM settings...");
            hyperv.configure_vm_for_gpu(vm_name, 1, 32)?; // 1GB low, 32GB high MMIO

            // Add the adapter
            println!("  Adding GPU partition adapter...");
            hyperv.add_gpu_to_vm(vm_name, gpu_path)?;

            println!("GPU-P adapter added successfully!");
            println!();
            println!("Note: You may need to copy GPU drivers to the VM:");
            println!("  cargo run --example gpu_passthrough -- gpup-drivers <vm_vhd_path>");
        }

        "gpup-remove" => {
            // Remove GPU-P from a VM
            let vm_name = args.get(2).ok_or("Missing VM name")?;

            println!("Removing GPU-P adapter from VM '{}'...", vm_name);
            hyperv.remove_gpu_from_vm(vm_name)?;
            println!("GPU-P adapter removed.");
        }

        "gpup-config" => {
            // Configure GPU-P adapter resources
            let vm_name = args.get(2).ok_or("Missing VM name")?;

            println!("Configuring GPU-P adapter for VM '{}'...", vm_name);

            // Example: Set VRAM limits (in bytes)
            // Min: 1GB, Max: 8GB, Optimal: 4GB
            let min_vram = 1_073_741_824u64; // 1 GB
            let max_vram = 8_589_934_592u64; // 8 GB
            let optimal_vram = 4_294_967_296u64; // 4 GB

            hyperv.configure_vm_gpu_adapter(
                vm_name,
                Some(min_vram),
                Some(max_vram),
                Some(optimal_vram),
                None, // encode
                None,
                None,
                None, // decode
                None,
                None,
                None, // compute
                None,
                None,
            )?;

            println!("GPU-P adapter configured:");
            println!("  Min VRAM:     {} GB", min_vram / 1_073_741_824);
            println!("  Max VRAM:     {} GB", max_vram / 1_073_741_824);
            println!("  Optimal VRAM: {} GB", optimal_vram / 1_073_741_824);
        }

        "gpup-list" => {
            // List GPU-P adapters for a VM
            let vm_name = args.get(2).ok_or("Missing VM name")?;

            println!("GPU-P adapters for VM '{}':\n", vm_name);

            let adapters = hyperv.get_vm_gpu_adapters(vm_name)?;

            if adapters.is_empty() {
                println!("  No GPU-P adapters configured.");
            } else {
                for (i, adapter) in adapters.iter().enumerate() {
                    println!("Adapter #{}:", i + 1);
                    if let Some(ref path) = adapter.instance_path {
                        println!("  Instance Path: {}", path);
                    }
                    if let Some(vram) = adapter.optimal_partition_vram {
                        println!("  Optimal VRAM: {} bytes", vram);
                    }
                    if let Some(vram) = adapter.min_partition_vram {
                        println!("  Min VRAM: {} bytes", vram);
                    }
                    if let Some(vram) = adapter.max_partition_vram {
                        println!("  Max VRAM: {} bytes", vram);
                    }
                }
            }
        }

        "gpup-drivers" => {
            // Copy GPU drivers to VM's VHD
            let vhd_path = args.get(2).ok_or("Missing VHD path")?;

            println!("Copying GPU drivers to VHD '{}'...", vhd_path);
            println!("(The VHD will be temporarily mounted)");

            hyperv.copy_gpu_drivers_to_vm(vhd_path, None)?;

            println!("GPU drivers copied successfully!");
        }

        // =================================================================
        // DDA (Discrete Device Assignment) Commands
        // =================================================================
        "dda-check" => {
            // Check DDA support
            let support = hyperv.check_dda_support()?;

            println!("DDA Support Check:\n");
            println!("  Is Supported:     {}", support.is_supported);
            println!("  Windows Server:   {}", support.is_server);
            println!("  IOMMU Available:  {}", support.has_iommu);
            println!("  Cmdlets Present:  {}", support.cmdlet_available);

            if let Some(ref reason) = support.reason {
                println!("\n  Note: {}", reason);
            }

            if !support.is_supported {
                println!();
                println!("DDA Requirements:");
                println!("  - Windows Server 2016 or later");
                println!("  - Hardware IOMMU (Intel VT-d or AMD-Vi)");
                println!("  - ACS-capable motherboard");
            }
        }

        "dda-list" => {
            // List assignable devices
            println!("DDA Assignable Devices:\n");

            let devices = hyperv.get_assignable_devices()?;

            if devices.is_empty() {
                println!("No devices are currently available for DDA assignment.");
                println!();
                println!("To prepare a device:");
                println!("  1. Get the device instance ID from 'gpu-info' example");
                println!("  2. Get location path: dda-path <instance_id>");
                println!("  3. Dismount device:   dda-dismount <location_path>");
            } else {
                for dev in &devices {
                    println!("Device: {}", dev.name);
                    println!("  Instance ID:   {}", dev.instance_id);
                    println!("  Location Path: {}", dev.location_path);
                    println!("  Status:        {}", dev.status);
                    if let Some(ref vm) = dev.assigned_vm {
                        println!("  Assigned to:   {}", vm);
                    }
                    println!();
                }
            }
        }

        "dda-path" => {
            // Get location path for a device
            let instance_id = args.get(2).ok_or("Missing device instance ID")?;

            println!("Getting location path for device...");
            let path = hyperv.get_device_location_path(instance_id)?;

            println!("Location Path: {}", path);
            println!();
            println!("Use this path with dda-dismount and dda-assign commands.");
        }

        "dda-dismount" => {
            // Dismount device from host
            let location_path = args.get(2).ok_or("Missing location path")?;

            println!("Dismounting device from host...");
            println!("WARNING: This will disable the device on the host!");
            println!();

            hyperv.dismount_device(location_path)?;

            println!("Device dismounted successfully.");
            println!("The device is now available for DDA assignment.");
        }

        "dda-mount" => {
            // Mount device back to host
            let location_path = args.get(2).ok_or("Missing location path")?;

            println!("Mounting device back to host...");
            hyperv.mount_device(location_path)?;

            println!("Device mounted successfully.");
            println!("The device is now available on the host.");
        }

        "dda-assign" => {
            // Assign device to VM
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let location_path = args.get(3).ok_or("Missing location path")?;

            println!("Assigning device to VM '{}'...", vm_name);

            // Configure VM for DDA first
            println!("  Configuring VM for DDA...");
            hyperv.configure_vm_for_dda(vm_name)?;

            // Set MMIO space (important for GPUs)
            println!("  Setting MMIO space...");
            hyperv.set_vm_mmio_space(vm_name, 256, 32)?; // 256MB low, 32GB high

            // Assign the device
            println!("  Assigning device...");
            hyperv.assign_device_to_vm(vm_name, location_path)?;

            println!("Device assigned successfully!");
            println!();
            println!("The VM now has exclusive access to this device.");
        }

        "dda-remove" => {
            // Remove assigned device from VM
            let vm_name = args.get(2).ok_or("Missing VM name")?;
            let location_path = args.get(3).ok_or("Missing location path")?;

            println!("Removing device from VM '{}'...", vm_name);
            hyperv.remove_assigned_device(vm_name, location_path)?;

            println!("Device removed from VM.");
            println!("Use 'dda-mount' to re-enable it on the host.");
        }

        "dda-move" => {
            // Move device between VMs
            let source_vm = args.get(2).ok_or("Missing source VM name")?;
            let target_vm = args.get(3).ok_or("Missing target VM name")?;
            let location_path = args.get(4).ok_or("Missing location path")?;

            println!("Moving device from '{}' to '{}'...", source_vm, target_vm);

            // Configure target VM for DDA
            println!("  Configuring target VM...");
            hyperv.configure_vm_for_dda(target_vm)?;
            hyperv.set_vm_mmio_space(target_vm, 256, 32)?;

            // Move the device
            println!("  Moving device...");
            hyperv.move_assigned_device(source_vm, target_vm, location_path)?;

            println!("Device moved successfully!");
        }

        "dda-vm" => {
            // List devices assigned to a VM
            let vm_name = args.get(2).ok_or("Missing VM name")?;

            println!("DDA devices assigned to VM '{}':\n", vm_name);

            let devices = hyperv.get_vm_assigned_devices(vm_name)?;

            if devices.is_empty() {
                println!("  No DDA devices assigned to this VM.");
            } else {
                for dev in &devices {
                    println!("  - {} ({})", dev.name, dev.location_path);
                }
            }
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
    eprintln!("GPU-P (Partitioning) Commands:");
    eprintln!("  gpup-add <vm_name> [gpu_path]  - Add GPU-P adapter to VM");
    eprintln!("  gpup-remove <vm_name>          - Remove GPU-P adapter from VM");
    eprintln!("  gpup-config <vm_name>          - Configure GPU-P adapter resources");
    eprintln!("  gpup-list <vm_name>            - List GPU-P adapters for VM");
    eprintln!("  gpup-drivers <vhd_path>        - Copy GPU drivers to VHD");
    eprintln!();
    eprintln!("DDA (Discrete Device Assignment) Commands:");
    eprintln!("  dda-check                      - Check DDA support on host");
    eprintln!("  dda-list                       - List available DDA devices");
    eprintln!("  dda-path <instance_id>         - Get location path for device");
    eprintln!("  dda-dismount <location_path>   - Dismount device from host");
    eprintln!("  dda-mount <location_path>      - Mount device back to host");
    eprintln!("  dda-assign <vm> <location>     - Assign device to VM");
    eprintln!("  dda-remove <vm> <location>     - Remove device from VM");
    eprintln!("  dda-move <src> <dst> <loc>     - Move device between VMs");
    eprintln!("  dda-vm <vm_name>               - List devices assigned to VM");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  # GPU-P workflow (Windows 10/11 Pro)");
    eprintln!("  {} gpup-add MyVM", program);
    eprintln!("  {} gpup-config MyVM", program);
    eprintln!("  {} gpup-drivers \"C:\\VMs\\MyVM.vhdx\"", program);
    eprintln!();
    eprintln!("  # DDA workflow (Windows Server only)");
    eprintln!("  {} dda-check", program);
    eprintln!("  {} dda-path \"PCI\\\\VEN_10DE&DEV_2204...\"", program);
    eprintln!(
        "  {} dda-dismount \"PCIROOT(0)#PCI(0100)#PCI(0000)\"",
        program
    );
    eprintln!(
        "  {} dda-assign MyVM \"PCIROOT(0)#PCI(0100)#PCI(0000)\"",
        program
    );
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Hyper-V installed.");
}
