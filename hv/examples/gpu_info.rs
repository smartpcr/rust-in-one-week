//! Example: GPU enumeration and information
//!
//! Lists all GPUs on the host and shows their capabilities for GPU-P and DDA.

#[cfg(windows)]
use hv::HyperV;

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hyperv = HyperV::new()?;

    println!("=== Host GPU Information ===\n");

    // List all GPUs
    let gpus = hyperv.list_gpus()?;

    if gpus.is_empty() {
        println!("No GPUs found on this system.");
        return Ok(());
    }

    println!("Found {} GPU(s):\n", gpus.len());

    for (i, gpu) in gpus.iter().enumerate() {
        println!("GPU #{}", i + 1);
        println!("  Name:         {}", gpu.name);
        println!("  Manufacturer: {}", gpu.manufacturer);
        println!("  Description:  {}", gpu.description);
        println!("  Instance ID:  {}", gpu.device_instance_id);
        if let Some(ref loc) = gpu.location {
            println!("  Location:     {}", loc);
        }
        if let Some(ref drv) = gpu.driver {
            println!("  Driver:       {}", drv);
        }
        println!(
            "  GPU-P Support: {}",
            if gpu.supports_partitioning {
                "Yes"
            } else {
                "No"
            }
        );

        // Try to get location path for DDA
        match hyperv.get_device_location_path(&gpu.device_instance_id) {
            Ok(path) => println!("  Location Path: {}", path),
            Err(_) => println!("  Location Path: (not available)"),
        }

        println!();
    }

    // Check for partitionable GPUs (GPU-P)
    println!("=== GPU-P (Partitionable GPUs) ===\n");
    let partitionable = hyperv.list_partitionable_gpus()?;

    if partitionable.is_empty() {
        println!("No GPUs support GPU-P on this system.");
        println!("GPU-P requires compatible GPU (NVIDIA, AMD, Intel) and driver support.\n");
    } else {
        println!("{} GPU(s) support GPU-P:\n", partitionable.len());
        for gpu in &partitionable {
            println!("  - {} ({})", gpu.name, gpu.device_instance_id);
        }
        println!();
    }

    // Check DDA support
    println!("=== DDA (Discrete Device Assignment) ===\n");
    let dda_support = hyperv.check_dda_support()?;

    println!("DDA Support Status:");
    println!("  Supported:      {}", dda_support.is_supported);
    println!("  Windows Server: {}", dda_support.is_server);
    println!("  IOMMU Present:  {}", dda_support.has_iommu);
    println!("  Cmdlets Available: {}", dda_support.cmdlet_available);

    if let Some(reason) = &dda_support.reason {
        println!("  Note: {}", reason);
    }

    if dda_support.is_supported {
        println!("\nAssignable Devices:");
        let devices = hyperv.get_assignable_devices()?;
        if devices.is_empty() {
            println!("  No devices currently dismounted for DDA.");
            println!("  Use 'dismount_device' to prepare a device for assignment.");
        } else {
            for dev in &devices {
                println!("  - {} ({})", dev.name, dev.location_path);
                println!("    Status: {}", dev.status);
                if let Some(ref vm) = dev.assigned_vm {
                    println!("    Assigned to: {}", vm);
                }
            }
        }
    }

    println!();
    println!("=== Comparison ===\n");
    println!("{:<20} {:<15} {:<15}", "Feature", "GPU-P", "DDA");
    println!("{}", "-".repeat(50));
    println!(
        "{:<20} {:<15} {:<15}",
        "Windows Client", "Yes", "No (Server)"
    );
    println!(
        "{:<20} {:<15} {:<15}",
        "GPU Sharing", "Yes", "No (Exclusive)"
    );
    println!("{:<20} {:<15} {:<15}", "Performance", "~90%", "~100%");
    println!("{:<20} {:<15} {:<15}", "Hot-swap", "Yes", "No");
    println!(
        "{:<20} {:<15} {:<15}",
        "Available Here",
        if !partitionable.is_empty() {
            "Yes"
        } else {
            "No"
        },
        if dda_support.is_supported {
            "Yes"
        } else {
            "No"
        }
    );

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Hyper-V installed.");
}
