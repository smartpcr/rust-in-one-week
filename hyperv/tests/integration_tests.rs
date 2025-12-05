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
    CheckpointSettings, Generation, HyperV, MemoryMB, NetworkAdapterSettings, ProcessorCount,
    ShutdownType, VhdSettings, VmSettings, VmState,
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
