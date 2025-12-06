//! Export and import virtual machines.
//!
//! Run with: cargo run --example export_import -- <action> <vm_name> <directory>
//! Actions:
//!   export <vm_name> <export_dir>       - Export VM configuration
//!   export-full <vm_name> <export_dir>  - Export VM with runtime state
//!   import <vm_name> <import_dir>       - Import VM from export
//!
//! Requires: Administrator privileges, Hyper-V enabled

use std::env;
use windows_hyperv::{ExportSettings, HyperV, ImportSettings, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        print_usage();
        return Ok(());
    }

    let action = &args[1];
    let vm_name = &args[2];
    let directory = &args[3];

    println!("Connecting to Hyper-V...");
    let hyperv = HyperV::connect()?;

    match action.as_str() {
        "export" => {
            println!("Finding VM '{}'...", vm_name);
            let vm = hyperv.get_vm(vm_name)?;

            println!("Exporting VM configuration to '{}'...", directory);
            vm.export_config(directory)?;
            println!("VM configuration exported successfully!");
            println!("Export location: {}\\{}", directory, vm_name);
        }
        "export-full" => {
            println!("Finding VM '{}'...", vm_name);
            let vm = hyperv.get_vm(vm_name)?;

            println!("Exporting VM with runtime state to '{}'...", directory);
            let settings = ExportSettings::full();
            vm.export(directory, &settings)?;
            println!("VM exported successfully!");
            println!("Export location: {}\\{}", directory, vm_name);
        }
        "export-storage" => {
            println!("Finding VM '{}'...", vm_name);
            let vm = hyperv.get_vm(vm_name)?;

            println!("Exporting VM with storage to '{}'...", directory);
            let settings = ExportSettings::full().with_storage(true);
            vm.export(directory, &settings)?;
            println!("VM exported with storage successfully!");
            println!("Export location: {}\\{}", directory, vm_name);
        }
        "import" => {
            println!("Importing VM '{}' from '{}'...", vm_name, directory);

            // Use new ID to avoid conflicts with existing VMs
            let settings = ImportSettings::new_id();
            let vm = hyperv.import_vm(vm_name, directory, &settings)?;

            println!("VM imported successfully!");
            println!("VM Name: {}", vm.name());
            println!("VM ID: {}", vm.id());
            println!("VM State: {:?}", vm.state());
        }
        "import-retain-id" => {
            println!(
                "Importing VM '{}' from '{}' (retaining original ID)...",
                vm_name, directory
            );

            // Retain original ID (may conflict if VM already exists)
            let settings = ImportSettings::retain_id();
            let vm = hyperv.import_vm(vm_name, directory, &settings)?;

            println!("VM imported successfully!");
            println!("VM Name: {}", vm.name());
            println!("VM ID: {}", vm.id());
            println!("VM State: {:?}", vm.state());
        }
        _ => {
            println!("Unknown action: {}", action);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage: export_import <action> <vm_name> <directory>");
    println!();
    println!("Actions:");
    println!("  export <vm_name> <export_dir>        - Export VM configuration only");
    println!("  export-full <vm_name> <export_dir>   - Export VM with runtime state");
    println!("  export-storage <vm_name> <export_dir> - Export VM with storage (VHDs)");
    println!("  import <vm_name> <import_dir>        - Import VM with new ID");
    println!("  import-retain-id <vm_name> <import_dir> - Import VM keeping original ID");
    println!();
    println!("Examples:");
    println!("  export_import export MyVM C:\\Exports");
    println!("  export_import import MyVM C:\\Exports");
}
