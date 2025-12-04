//! Integration tests for the hv library
//!
//! These tests require Windows with Hyper-V installed and admin privileges.
//! Run with: cargo test --package hv -- --ignored

#[cfg(windows)]
mod hyperv_tests {
    use hv::{HvError, HyperV, SnapshotType, SwitchType, VhdType, VmGeneration, VmState};

    /// Helper to check if an error is a permission error
    fn is_permission_error(err: &HvError) -> bool {
        let msg = match err {
            HvError::OperationFailed(msg) => msg.as_str(),
            HvError::WmiError(msg) => msg.as_str(),
            HvError::ConnectionFailed(msg) => msg.as_str(),
            _ => return false,
        };
        msg.contains("permission")
            || msg.contains("authorization")
            || msg.contains("Access is denied")
            || msg.contains("access denied")
            || msg.contains("0x80070005") // E_ACCESSDENIED
            || msg.contains("0x80041003") // WBEM_E_ACCESS_DENIED
    }

    /// Test connecting to Hyper-V
    #[test]
    #[ignore] // Requires Hyper-V
    fn test_connect() {
        let result = HyperV::new();
        assert!(result.is_ok(), "Should be able to connect to Hyper-V");
    }

    /// Test getting host information
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_host_info() {
        let hyperv = HyperV::new().expect("Failed to connect");
        match hyperv.host_info() {
            Ok(info) => {
                assert!(
                    !info.computer_name.is_empty(),
                    "Computer name should not be empty"
                );
                assert!(
                    info.logical_processor_count > 0,
                    "Should have at least 1 CPU"
                );
                assert!(info.memory_capacity_bytes > 0, "Should have memory");
                println!("Host: {}", info.computer_name);
                println!("CPUs: {}", info.logical_processor_count);
                println!(
                    "Memory: {} GB",
                    info.memory_capacity_bytes / 1024 / 1024 / 1024
                );
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_host_info: insufficient permissions");
            }
            Err(e) => panic!("Failed to get host info: {:?}", e),
        }
    }

    /// Test listing VMs
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_list_vms() {
        let hyperv = HyperV::new().expect("Failed to connect");
        match hyperv.list_vms() {
            Ok(mut vms) => {
                println!("Found {} VMs", vms.len());
                for vm in &mut vms {
                    let name = vm.name().to_string();
                    let state = vm.state().unwrap_or(VmState::Unknown);
                    println!("  - {}: {:?}", name, state);
                }
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_list_vms: insufficient permissions");
            }
            Err(e) => panic!("Failed to list VMs: {:?}", e),
        }
    }

    /// Test listing virtual switches
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_list_switches() {
        let hyperv = HyperV::new().expect("Failed to connect");
        match hyperv.list_switches() {
            Ok(switches) => {
                println!("Found {} switches", switches.len());
                for switch in &switches {
                    println!(
                        "  - {}: {:?}",
                        switch.name(),
                        switch.switch_type().unwrap_or(SwitchType::Private)
                    );
                }
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_list_switches: insufficient permissions");
            }
            Err(e) => panic!("Failed to list switches: {:?}", e),
        }
    }

    /// Test getting a non-existent VM
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_get_nonexistent_vm() {
        let hyperv = HyperV::new().expect("Failed to connect");
        let result = hyperv.get_vm("NonExistentVM12345");

        match result {
            Err(HvError::VmNotFound(name)) => {
                assert_eq!(name, "NonExistentVM12345");
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_get_nonexistent_vm: insufficient permissions");
            }
            Ok(_) => panic!("Should fail for non-existent VM"),
            Err(e) => panic!("Expected VmNotFound error, got: {:?}", e),
        }
    }

    /// Test VM state enumeration
    #[test]
    fn test_vm_state_from_hcs_state() {
        assert_eq!(VmState::from_hcs_state("running"), VmState::Running);
        assert_eq!(VmState::from_hcs_state("off"), VmState::Off);
        assert_eq!(VmState::from_hcs_state("stopped"), VmState::Off);
        assert_eq!(VmState::from_hcs_state("paused"), VmState::Paused);
        assert_eq!(VmState::from_hcs_state("saved"), VmState::Saved);
        assert_eq!(VmState::from_hcs_state("starting"), VmState::Starting);
        assert_eq!(VmState::from_hcs_state("stopping"), VmState::Stopping);
        assert_eq!(VmState::from_hcs_state("unknown_state"), VmState::Unknown);
    }

    /// Test VM state helper methods
    #[test]
    fn test_vm_state_helpers() {
        assert!(VmState::Running.is_running());
        assert!(!VmState::Off.is_running());

        assert!(VmState::Off.is_off());
        assert!(!VmState::Running.is_off());

        assert!(VmState::Starting.is_transitioning());
        assert!(VmState::Stopping.is_transitioning());
        assert!(VmState::Saving.is_transitioning());
        assert!(!VmState::Running.is_transitioning());
        assert!(!VmState::Off.is_transitioning());
    }

    /// Test switch type conversion
    #[test]
    fn test_switch_type_from_u16() {
        assert_eq!(SwitchType::from(0), SwitchType::Private);
        assert_eq!(SwitchType::from(1), SwitchType::Internal);
        assert_eq!(SwitchType::from(2), SwitchType::External);
    }

    /// Test VHD type conversion
    #[test]
    fn test_vhd_type_from_u16() {
        assert_eq!(VhdType::from(2), VhdType::Fixed);
        assert_eq!(VhdType::from(3), VhdType::Dynamic);
        assert_eq!(VhdType::from(4), VhdType::Differencing);
    }

    /// Test VHD format detection
    #[test]
    fn test_vhd_format_from_path() {
        use hv::VhdFormat;

        assert_eq!(VhdFormat::from_path("test.vhd"), VhdFormat::Vhd);
        assert_eq!(VhdFormat::from_path("test.VHD"), VhdFormat::Vhd);
        assert_eq!(VhdFormat::from_path("test.vhdx"), VhdFormat::Vhdx);
        assert_eq!(VhdFormat::from_path("test.VHDX"), VhdFormat::Vhdx);
        assert_eq!(VhdFormat::from_path("C:\\VMs\\disk.vhdx"), VhdFormat::Vhdx);
    }

    /// Test getting a specific VM (if any exist)
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_get_existing_vm() {
        let hyperv = HyperV::new().expect("Failed to connect");
        match hyperv.list_vms() {
            Ok(vms) => {
                if let Some(first_vm) = vms.first() {
                    let vm_name = first_vm.name();
                    match hyperv.get_vm(vm_name) {
                        Ok(mut vm) => {
                            assert_eq!(vm.name(), vm_name);
                            println!("Got VM: {} ({})", vm.name(), vm.id());
                            println!("State: {:?}", vm.state().unwrap_or(VmState::Unknown));
                        }
                        Err(e) if is_permission_error(&e) => {
                            println!("Skipping test_get_existing_vm: insufficient permissions");
                        }
                        Err(e) => panic!("Failed to get VM: {:?}", e),
                    }
                } else {
                    println!("No VMs found to test get_vm");
                }
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_get_existing_vm: insufficient permissions");
            }
            Err(e) => panic!("Failed to list VMs: {:?}", e),
        }
    }

    /// Test VM creation and deletion
    /// WARNING: This test creates and deletes a real VM!
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_vm_lifecycle() {
        use std::path::Path;

        let hyperv = HyperV::new().expect("Failed to connect");
        let vm_name = "HvTestVM_IntegrationTest";

        // Clean up if exists from previous run
        let _ = hyperv.delete_vm(vm_name);

        // Ensure VHD directory exists
        let vhd_dir = "C:\\Hyper-V\\Virtual Hard Disks";
        if !Path::new(vhd_dir).exists() {
            std::fs::create_dir_all(vhd_dir).expect("Failed to create VHD directory");
        }

        // Create VM with a new VHD
        let vhd_path = format!("{}\\{}.vhdx", vhd_dir, vm_name);

        // Clean up VHD if exists from previous run
        let _ = std::fs::remove_file(&vhd_path);

        let vhd_size = 10 * 1024 * 1024 * 1024u64; // 10GB for test
        match hyperv.create_vm(
            vm_name,
            512,
            2,
            VmGeneration::Gen2,
            &vhd_path,
            vhd_size,
            None, // switch_name
        ) {
            Ok(mut vm) => {
                assert_eq!(vm.name(), vm_name);
                assert_eq!(vm.state().unwrap(), VmState::Off);

                // Delete VM
                hyperv.delete_vm(vm_name).expect("Failed to delete VM");

                // Clean up VHD file
                let _ = std::fs::remove_file(&vhd_path);

                // Verify deleted
                assert!(hyperv.get_vm(vm_name).is_err());
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_vm_lifecycle: insufficient permissions");
            }
            Err(e) => panic!("Failed to create VM: {:?}", e),
        }
    }

    /// Test snapshot operations
    /// WARNING: This test creates real snapshots!
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_snapshot_operations() {
        let hyperv = HyperV::new().expect("Failed to connect");
        match hyperv.list_vms() {
            Ok(vms) => {
                if let Some(vm) = vms.first() {
                    let vm_name = vm.name();
                    let snap_name = "HvTestSnapshot_IntegrationTest";

                    // Create snapshot
                    match hyperv.create_snapshot(vm_name, snap_name, SnapshotType::Standard) {
                        Ok(snapshot) => {
                            assert_eq!(snapshot.name(), snap_name);
                            assert_eq!(snapshot.vm_name(), vm_name);

                            // List snapshots
                            let snapshots = hyperv
                                .list_snapshots(vm_name)
                                .expect("Failed to list snapshots");

                            assert!(snapshots.iter().any(|s| s.name() == snap_name));

                            // Delete snapshot
                            snapshot.delete().expect("Failed to delete snapshot");

                            // Verify deleted
                            let snapshots_after = hyperv
                                .list_snapshots(vm_name)
                                .expect("Failed to list snapshots");

                            assert!(!snapshots_after.iter().any(|s| s.name() == snap_name));
                        }
                        Err(e) if is_permission_error(&e) => {
                            println!("Skipping test_snapshot_operations: insufficient permissions");
                        }
                        Err(e) => panic!("Failed to create snapshot: {:?}", e),
                    }
                } else {
                    println!("No VMs found to test snapshot operations");
                }
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_snapshot_operations: insufficient permissions");
            }
            Err(e) => panic!("Failed to list VMs: {:?}", e),
        }
    }

    /// Test switch creation and deletion
    /// WARNING: This test creates and deletes a real switch!
    #[test]
    #[ignore] // Requires Hyper-V and admin privileges
    fn test_switch_lifecycle() {
        let hyperv = HyperV::new().expect("Failed to connect");
        let switch_name = "HvTestSwitch_IntegrationTest";

        // Clean up if exists from previous run
        if let Ok(switch) = hyperv.get_switch(switch_name) {
            let _ = switch.delete();
        }

        // Create internal switch
        match hyperv.create_switch(switch_name, SwitchType::Internal) {
            Ok(switch) => {
                assert_eq!(switch.name(), switch_name);
                assert_eq!(switch.switch_type().unwrap(), SwitchType::Internal);

                // Delete switch
                switch.delete().expect("Failed to delete switch");

                // Verify deleted
                assert!(hyperv.get_switch(switch_name).is_err());
            }
            Err(e) if is_permission_error(&e) => {
                println!("Skipping test_switch_lifecycle: insufficient permissions");
            }
            Err(e) => panic!("Failed to create switch: {:?}", e),
        }
    }
}

/// Unit tests that don't require Hyper-V
mod unit_tests {
    use hv::HvError;

    #[test]
    fn test_error_display() {
        let err = HvError::VmNotFound("TestVM".to_string());
        assert_eq!(format!("{}", err), "VM not found: TestVM");

        let err = HvError::SwitchNotFound("TestSwitch".to_string());
        assert_eq!(format!("{}", err), "Virtual switch not found: TestSwitch");

        let err = HvError::InvalidState("Running".to_string());
        assert_eq!(
            format!("{}", err),
            "VM is in invalid state for this operation: Running"
        );

        let err = HvError::ConnectionFailed("WMI error".to_string());
        assert_eq!(
            format!("{}", err),
            "Failed to connect to Hyper-V WMI: WMI error"
        );
    }

    #[test]
    fn test_error_debug() {
        let err = HvError::VmNotFound("TestVM".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("VmNotFound"));
        assert!(debug_str.contains("TestVM"));
    }
}
