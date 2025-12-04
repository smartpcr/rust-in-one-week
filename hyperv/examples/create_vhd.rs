//! Create a new VHD/VHDX virtual hard disk.
//!
//! Run with: cargo run --example create_vhd -- <path> <size_gb>
//! Example: cargo run --example create_vhd -- C:\VMs\disk.vhdx 100
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{HyperV, Result, VhdSettings, VhdType};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: create_vhd <path> <size_gb> [fixed|dynamic]");
        println!("Example: create_vhd C:\\VMs\\disk.vhdx 100 dynamic");
        return Ok(());
    }

    let path = &args[1];
    let size_gb: u64 = args[2].parse().expect("Invalid size");
    let disk_type = if args.len() > 3 && args[3] == "fixed" {
        VhdType::Fixed
    } else {
        VhdType::Dynamic
    };

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    println!("Creating VHD...");
    println!("  Path: {}", path);
    println!("  Size: {} GB", size_gb);
    println!("  Type: {:?}", disk_type);

    let settings = VhdSettings::builder()
        .path(path)
        .size_gb(size_gb)
        .disk_type(disk_type)
        .build()?;

    let vhd = hyperv.vhd().create(&settings)?;

    println!("\nVHD created successfully!");
    println!("  Path:   {}", vhd.path);
    println!("  Format: {:?}", vhd.format);
    println!("  Type:   {:?}", vhd.disk_type);

    Ok(())
}
