//! Integration tests for windows-hyperv crate.
//!
//! These tests require:
//! - Windows with Hyper-V enabled
//! - Administrator privileges
//! - The `integration` feature enabled
//!
//! Run with: cargo test -p windows-hyperv --features integration -- --test-threads=1
//!
//! Tests are guarded by both #[cfg(windows)] and #[cfg(feature = "integration")]
//! to prevent them from running in CI or on non-Windows systems.

#![cfg(all(windows, feature = "integration"))]

use windows_hyperv::{
    CheckpointSettings, ExportSettings, Generation, HyperV, ImportSettings, MemoryMB,
    NetworkAdapterSettings, ProcessorCount, ShutdownType, VhdSettings, VmSettings, VmState,
};

const TEST_VM_PREFIX: &str = "HyperV_IntegTest_";

fn test_vm_name(suffix: &str) -> String {
    format!("{}{}", TEST_VM_PREFIX, suffix)
}

fn cleanup_test_vm(hyperv: &HyperV, name: &str) {
    // Try multiple times in case VM is in transitional state
    for _ in 0..3 {
        if let Ok(vm) = hyperv.get_vm(name) {
            // Stop if not off
            if vm.state() != VmState::Off {
                let _ = hyperv
                    .get_vm(name)
                    .and_then(|mut v| v.stop(ShutdownType::Force));
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
            // Delete
            if let Ok(vm) = hyperv.get_vm(name) {
                if vm.state() == VmState::Off {
                    let _ = hyperv.delete_vm(&vm);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        } else {
            // VM doesn't exist, we're done
            break;
        }
    }
    // Final delay to ensure Hyper-V has fully processed the deletion
    std::thread::sleep(std::time::Duration::from_millis(500));
}

#[test]
fn test_connect() {
    let result = HyperV::connect();
    assert!(
        result.is_ok(),
        "Failed to connect to Hyper-V: {:?}",
        result.err()
    );
}

#[test]
fn test_list_vms() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let result = hyperv.list_vms();
    assert!(result.is_ok(), "Failed to list VMs: {:?}", result.err());
}

#[test]
fn test_list_switches() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let result = hyperv.list_switches();
    assert!(
        result.is_ok(),
        "Failed to list switches: {:?}",
        result.err()
    );
}

#[test]
fn test_create_and_delete_vm() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("CreateDelete");

    // Cleanup any leftover test VM
    cleanup_test_vm(&hyperv, &vm_name);

    // Create VM using strong types
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let vm = hyperv.create_vm(&settings);
    assert!(vm.is_ok(), "Failed to create VM: {:?}", vm.err());
    let vm = vm.unwrap();
    assert_eq!(vm.name(), vm_name);
    assert_eq!(vm.state(), VmState::Off);

    // Verify VM exists
    let found = hyperv.get_vm(&vm_name);
    assert!(found.is_ok(), "Failed to find created VM");

    // Delete VM
    let delete_result = hyperv.delete_vm(&vm);
    assert!(
        delete_result.is_ok(),
        "Failed to delete VM: {:?}",
        delete_result.err()
    );

    // Verify VM is gone
    let not_found = hyperv.get_vm(&vm_name);
    assert!(not_found.is_err(), "VM should not exist after deletion");
}

#[test]
fn test_vm_power_cycle() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("PowerCycle");

    // Cleanup any leftover test VM
    cleanup_test_vm(&hyperv, &vm_name);

    // Create VM using strong types
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");

    // Start VM
    let mut vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");
    let start_result = vm.start();
    assert!(
        start_result.is_ok(),
        "Failed to start VM: {:?}",
        start_result.err()
    );

    // Wait for VM to start
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Refresh and check state
    vm.refresh().expect("Failed to refresh VM state");
    assert_eq!(vm.state(), VmState::Running, "VM should be running");

    // Stop VM
    let stop_result = vm.stop(ShutdownType::Force);
    assert!(
        stop_result.is_ok(),
        "Failed to stop VM: {:?}",
        stop_result.err()
    );

    // Wait for VM to stop
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Refresh and check state
    vm.refresh().expect("Failed to refresh VM state");
    assert_eq!(vm.state(), VmState::Off, "VM should be off");

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
}

#[test]
fn test_vm_get_by_id() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("GetById");

    // Cleanup any leftover test VM
    cleanup_test_vm(&hyperv, &vm_name);

    // Create VM using strong types
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen1)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let vm = hyperv.create_vm(&settings).expect("Failed to create VM");
    let vm_id = vm.id().to_string();

    // Get by ID
    let found = hyperv.get_vm_by_id(&vm_id);
    assert!(found.is_ok(), "Failed to get VM by ID: {:?}", found.err());
    assert_eq!(found.unwrap().name(), vm_name);

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
}

#[test]
fn test_add_network_adapter() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("Network");

    // Cleanup any leftover test VM
    cleanup_test_vm(&hyperv, &vm_name);

    // Create VM using strong types
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let vm = hyperv.create_vm(&settings).expect("Failed to create VM");

    // Add network adapter
    let adapter_settings = NetworkAdapterSettings::builder()
        .name("TestAdapter")
        .build()
        .expect("Failed to build adapter settings");

    let adapter = hyperv.add_network_adapter(&vm, &adapter_settings);
    assert!(
        adapter.is_ok(),
        "Failed to add network adapter: {:?}",
        adapter.err()
    );

    // List adapters
    let adapters = hyperv.list_network_adapters(&vm);
    assert!(
        adapters.is_ok(),
        "Failed to list adapters: {:?}",
        adapters.err()
    );
    assert!(
        !adapters.unwrap().is_empty(),
        "Should have at least one adapter"
    );

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
}

#[test]
fn test_create_checkpoint() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("Checkpoint");

    // Cleanup any leftover test VM
    cleanup_test_vm(&hyperv, &vm_name);

    // Create VM using strong types
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");
    let vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");

    // Create checkpoint (VM must be off or running for standard checkpoint)
    let cp_settings = CheckpointSettings::builder()
        .name("TestCheckpoint")
        .notes("Integration test checkpoint")
        .build()
        .expect("Failed to build checkpoint settings");

    let checkpoint = hyperv.create_checkpoint(&vm, &cp_settings);
    assert!(
        checkpoint.is_ok(),
        "Failed to create checkpoint: {:?}",
        checkpoint.err()
    );

    // List checkpoints
    let checkpoints = hyperv.list_checkpoints(&vm);
    assert!(
        checkpoints.is_ok(),
        "Failed to list checkpoints: {:?}",
        checkpoints.err()
    );

    // Delete checkpoint
    if let Ok(cp) = checkpoint {
        let delete_result = hyperv.delete_checkpoint(&cp);
        assert!(
            delete_result.is_ok(),
            "Failed to delete checkpoint: {:?}",
            delete_result.err()
        );
    }

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
}

#[test]
fn test_vhd_manager_create() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vhd_path = std::env::temp_dir().join("hyperv_test.vhdx");
    let vhd_path_str = vhd_path.to_string_lossy().to_string();

    // Remove if exists
    let _ = std::fs::remove_file(&vhd_path);

    // Create VHD
    let settings = VhdSettings::builder()
        .path(&vhd_path_str)
        .size_gb(1)
        .build()
        .expect("Failed to build VHD settings");

    let vhd = hyperv.vhd().create(&settings);
    assert!(vhd.is_ok(), "Failed to create VHD: {:?}", vhd.err());
    assert!(vhd_path.exists(), "VHD file should exist");

    // Cleanup
    let _ = std::fs::remove_file(&vhd_path);
}

#[test]
fn test_vm_hibernate() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("Hibernate");

    // Cleanup any leftover test VM
    cleanup_test_vm(&hyperv, &vm_name);

    // Create VM
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");

    // Start VM
    let mut vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");
    vm.start().expect("Failed to start VM");

    // Wait for VM to start
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Try to hibernate (may fail if VM doesn't support it without OS)
    // This is more of a smoke test to ensure the method doesn't panic
    let hibernate_result = vm.hibernate();
    // Note: Hibernate typically requires a running OS with hibernate support
    // This test mainly verifies the API works without panicking
    if hibernate_result.is_err() {
        println!(
            "Hibernate not supported (expected without running OS): {:?}",
            hibernate_result.err()
        );
    }

    // Stop VM
    let _ = vm.stop(ShutdownType::Force);
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
}

#[test]
fn test_vm_operational_status() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("OpStatus");

    // Cleanup any leftover test VM
    cleanup_test_vm(&hyperv, &vm_name);

    // Create VM
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");
    let vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");

    // Get operational status
    let status = vm.get_operational_status();
    assert!(
        status.is_ok(),
        "Failed to get operational status: {:?}",
        status.err()
    );

    let (primary, secondary) = status.unwrap();
    println!("Operational status: {:?}, {:?}", primary, secondary);

    // Check is_migrating (should be false for a new VM)
    let is_migrating = vm.is_migrating();
    assert!(
        is_migrating.is_ok(),
        "Failed to check migration status: {:?}",
        is_migrating.err()
    );
    assert!(!is_migrating.unwrap(), "New VM should not be migrating");

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
}

#[test]
fn test_vm_export_config() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("ExportConfig");
    let export_dir = std::env::temp_dir().join("hyperv_export_test");

    // Cleanup any leftover test VM and export directory
    cleanup_test_vm(&hyperv, &vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);

    // Create export directory
    std::fs::create_dir_all(&export_dir).expect("Failed to create export directory");

    // Create VM
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");
    let vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");

    // Export config only
    let export_result = vm.export_config(&export_dir);
    assert!(
        export_result.is_ok(),
        "Failed to export VM config: {:?}",
        export_result.err()
    );

    // Verify export directory was created
    let vm_export_dir = export_dir.join(&vm_name);
    assert!(vm_export_dir.exists(), "VM export directory should exist");

    // Verify Virtual Machines folder exists
    let vm_machines_dir = vm_export_dir.join("Virtual Machines");
    assert!(
        vm_machines_dir.exists(),
        "Virtual Machines folder should exist"
    );

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);
}

#[test]
fn test_vm_export_full() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("ExportFull");
    let export_dir = std::env::temp_dir().join("hyperv_export_full_test");

    // Cleanup any leftover test VM and export directory
    cleanup_test_vm(&hyperv, &vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);

    // Create export directory
    std::fs::create_dir_all(&export_dir).expect("Failed to create export directory");

    // Create VM
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");
    let vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");

    // Export with full settings
    let export_settings = ExportSettings::full();
    let export_result = vm.export(&export_dir, &export_settings);
    assert!(
        export_result.is_ok(),
        "Failed to export VM: {:?}",
        export_result.err()
    );

    // Verify export directory was created
    let vm_export_dir = export_dir.join(&vm_name);
    assert!(vm_export_dir.exists(), "VM export directory should exist");

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);
}

#[test]
fn test_vm_export_and_import() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("ExportImport");
    let imported_vm_name = test_vm_name("ExportImport"); // Same name after import
    let export_dir = std::env::temp_dir().join("hyperv_export_import_test");

    // Cleanup any leftover test VMs and export directory
    cleanup_test_vm(&hyperv, &vm_name);
    cleanup_test_vm(&hyperv, &imported_vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);

    // Create export directory
    std::fs::create_dir_all(&export_dir).expect("Failed to create export directory");

    // Create VM
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");
    let vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");

    // Export config only
    let export_result = vm.export_config(&export_dir);
    assert!(
        export_result.is_ok(),
        "Failed to export VM: {:?}",
        export_result.err()
    );

    // Delete the original VM
    let delete_result = hyperv.delete_vm(&vm);
    assert!(
        delete_result.is_ok(),
        "Failed to delete VM: {:?}",
        delete_result.err()
    );

    // Wait for deletion to complete
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Import the VM with new ID to avoid conflicts
    let import_settings = ImportSettings::new_id();
    let import_result = hyperv.import_vm(&vm_name, &export_dir, &import_settings);
    assert!(
        import_result.is_ok(),
        "Failed to import VM: {:?}",
        import_result.err()
    );

    let imported_vm = import_result.unwrap();
    assert_eq!(imported_vm.name(), vm_name);
    assert_eq!(imported_vm.state(), VmState::Off);

    // Cleanup
    cleanup_test_vm(&hyperv, &imported_vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);
}

#[test]
fn test_vm_export_with_custom_settings() {
    let hyperv = HyperV::connect().expect("Failed to connect");
    let vm_name = test_vm_name("ExportCustom");
    let export_dir = std::env::temp_dir().join("hyperv_export_custom_test");

    // Cleanup any leftover test VM and export directory
    cleanup_test_vm(&hyperv, &vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);

    // Create export directory
    std::fs::create_dir_all(&export_dir).expect("Failed to create export directory");

    // Create VM
    let settings = VmSettings::builder()
        .name(&vm_name)
        .generation(Generation::Gen2)
        .memory(MemoryMB::mb_512())
        .processors(ProcessorCount::one())
        .build()
        .expect("Failed to build VM settings");

    let _vm = hyperv.create_vm(&settings).expect("Failed to create VM");
    let vm = hyperv.get_vm(&vm_name).expect("Failed to get VM");

    // Export with custom settings using builder pattern
    let export_settings = ExportSettings::config_only()
        .with_overwrite(true);

    let export_result = vm.export(&export_dir, &export_settings);
    assert!(
        export_result.is_ok(),
        "Failed to export VM with custom settings: {:?}",
        export_result.err()
    );

    // Cleanup
    cleanup_test_vm(&hyperv, &vm_name);
    let _ = std::fs::remove_dir_all(&export_dir);
}
