//! Integration tests for the clus library
//!
//! These tests require a Windows Failover Cluster to be available.
//!
//! Test organization:
//! - Unit tests: Run with `cargo test` (always enabled)
//! - Integration tests: Run with `cargo test --features integration` (requires cluster)

#[cfg(all(windows, feature = "integration"))]
mod cluster_tests {
    use clus::{ClusError, Cluster, GroupState, NodeState, ResourceState};

    /// Test opening a connection to the local cluster
    #[test]
    fn test_open_local_cluster() {
        let result = Cluster::open(None);
        assert!(result.is_ok(), "Should be able to open local cluster");

        let cluster = result.unwrap();
        let name = cluster.name();
        assert!(name.is_ok(), "Should be able to get cluster name");
        println!("Connected to cluster: {}", name.unwrap());
    }

    /// Test opening a connection to a named cluster
    #[test]
    fn test_open_named_cluster() {
        // This will fail if the cluster doesn't exist, which is expected
        let result = Cluster::open(Some("NonExistentCluster"));
        assert!(result.is_err(), "Should fail for non-existent cluster");
    }

    /// Test enumerating all nodes in the cluster
    #[test]
    fn test_enumerate_nodes() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let nodes = cluster.nodes().expect("Failed to enumerate nodes");

        assert!(!nodes.is_empty(), "Cluster should have at least one node");

        for node in &nodes {
            assert!(!node.name().is_empty(), "Node name should not be empty");
            let state = node.state();
            println!("Node: {} - {:?}", node.name(), state);

            // Verify state is a known value
            match state {
                NodeState::Up
                | NodeState::Down
                | NodeState::Paused
                | NodeState::Joining
                | NodeState::Unknown(_) => {}
            }
        }
    }

    /// Test enumerating all resources in the cluster
    #[test]
    fn test_enumerate_resources() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let resources = cluster.resources().expect("Failed to enumerate resources");

        println!("Found {} resources", resources.len());

        for resource in &resources {
            assert!(
                !resource.name().is_empty(),
                "Resource name should not be empty"
            );
            let (state, owner) = resource.state().expect("Failed to get resource state");
            println!("Resource: {} - {:?} on {:?}", resource.name(), state, owner);

            // Verify state is a known value
            match state {
                ResourceState::Online
                | ResourceState::Offline
                | ResourceState::Failed
                | ResourceState::OnlinePending
                | ResourceState::OfflinePending
                | ResourceState::Unknown(_) => {}
            }
        }
    }

    /// Test enumerating all groups in the cluster
    #[test]
    fn test_enumerate_groups() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let groups = cluster.groups().expect("Failed to enumerate groups");

        println!("Found {} groups", groups.len());

        for group in &groups {
            assert!(!group.name().is_empty(), "Group name should not be empty");
            let (state, owner) = group.state().expect("Failed to get group state");
            println!("Group: {} - {:?} on {:?}", group.name(), state, owner);

            // Verify state is a known value
            match state {
                GroupState::Online
                | GroupState::Offline
                | GroupState::Failed
                | GroupState::PartialOnline
                | GroupState::Pending
                | GroupState::Unknown(_) => {}
            }
        }
    }

    /// Test opening a specific node by name
    #[test]
    fn test_open_node_by_name() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let nodes = cluster.nodes().expect("Failed to enumerate nodes");

        if let Some(first_node) = nodes.first() {
            let node_name = first_node.name();
            let opened_node = cluster
                .open_node(node_name)
                .expect("Failed to open node by name");

            assert_eq!(opened_node.name(), node_name);
            println!(
                "Opened node: {} - {:?}",
                opened_node.name(),
                opened_node.state()
            );
        }
    }

    /// Test opening a non-existent node
    #[test]
    fn test_open_nonexistent_node() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let result = cluster.open_node("NonExistentNode12345");

        assert!(result.is_err(), "Should fail for non-existent node");
        match result {
            Err(ClusError::NotFound(name)) => {
                assert_eq!(name, "NonExistentNode12345");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    /// Test opening a specific resource by name
    #[test]
    fn test_open_resource_by_name() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let resources = cluster.resources().expect("Failed to enumerate resources");

        if let Some(first_resource) = resources.first() {
            let resource_name = first_resource.name();
            let opened_resource = cluster
                .open_resource(resource_name)
                .expect("Failed to open resource by name");

            assert_eq!(opened_resource.name(), resource_name);
            let (state, owner) = opened_resource.state().expect("Failed to get state");
            println!(
                "Opened resource: {} - {:?} on {:?}",
                opened_resource.name(),
                state,
                owner
            );
        }
    }

    /// Test opening a specific group by name
    #[test]
    fn test_open_group_by_name() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let groups = cluster.groups().expect("Failed to enumerate groups");

        if let Some(first_group) = groups.first() {
            let group_name = first_group.name();
            let opened_group = cluster
                .open_group(group_name)
                .expect("Failed to open group by name");

            assert_eq!(opened_group.name(), group_name);
            let (state, owner) = opened_group.state().expect("Failed to get state");
            println!(
                "Opened group: {} - {:?} on {:?}",
                opened_group.name(),
                state,
                owner
            );
        }
    }

    /// Test node pause and resume operations
    /// WARNING: This test will actually pause a node! Use with caution.
    #[test]
    fn test_node_pause_resume() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let nodes = cluster.nodes().expect("Failed to enumerate nodes");

        // Find a node that is Up
        let up_node = nodes.iter().find(|n| n.state() == NodeState::Up);

        if let Some(node) = up_node {
            println!("Testing pause/resume on node: {}", node.name());

            // Pause the node
            let pause_result = node.pause();
            if pause_result.is_ok() {
                println!("Node paused successfully");

                // Give it a moment
                std::thread::sleep(std::time::Duration::from_secs(1));

                // Resume the node
                let resume_result = node.resume();
                assert!(resume_result.is_ok(), "Should be able to resume node");
                println!("Node resumed successfully");
            } else {
                println!("Could not pause node (may require admin privileges)");
            }
        } else {
            println!("No Up nodes found to test pause/resume");
        }
    }

    /// Test resource online/offline operations
    /// WARNING: This test will actually take a resource offline! Use with caution.
    #[test]
    fn test_resource_online_offline() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let resources = cluster.resources().expect("Failed to enumerate resources");

        // Find a resource that is Online
        let online_resource = resources.iter().find(|r| {
            if let Ok((state, _)) = r.state() {
                state == ResourceState::Online
            } else {
                false
            }
        });

        if let Some(resource) = online_resource {
            println!("Testing offline/online on resource: {}", resource.name());

            // Take offline
            let offline_result = resource.offline();
            if offline_result.is_ok() {
                println!("Resource offline initiated");

                // Wait for it to go offline
                std::thread::sleep(std::time::Duration::from_secs(5));

                // Bring back online
                let online_result = resource.online();
                assert!(
                    online_result.is_ok(),
                    "Should be able to bring resource online"
                );
                println!("Resource online initiated");
            } else {
                println!("Could not take resource offline (may require admin privileges)");
            }
        } else {
            println!("No Online resources found to test offline/online");
        }
    }

    /// Test group move operation
    /// WARNING: This test will actually move a group! Use with caution.
    #[test]
    fn test_group_move() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let groups = cluster.groups().expect("Failed to enumerate groups");
        let nodes = cluster.nodes().expect("Failed to enumerate nodes");

        // Find an online group
        let online_group = groups.iter().find(|g| {
            if let Ok((state, _)) = g.state() {
                state == GroupState::Online
            } else {
                false
            }
        });

        // Find another node to move to
        let up_nodes: Vec<_> = nodes
            .iter()
            .filter(|n| n.state() == NodeState::Up)
            .collect();

        if let (Some(group), true) = (online_group, up_nodes.len() >= 2) {
            let (_, current_owner) = group.state().expect("Failed to get group state");
            let current_owner_name = current_owner.as_deref().unwrap_or("");

            // Find a different node
            let target_node = up_nodes.iter().find(|n| n.name() != current_owner_name);

            if let Some(target) = target_node {
                println!(
                    "Testing move of group '{}' from '{}' to '{}'",
                    group.name(),
                    current_owner_name,
                    target.name()
                );

                let move_result = group.move_to(target);
                if move_result.is_ok() {
                    println!("Move initiated successfully");

                    // Wait for move to complete
                    std::thread::sleep(std::time::Duration::from_secs(5));

                    let (new_state, new_owner) = group.state().expect("Failed to get state");
                    println!("Group is now {:?} on {:?}", new_state, new_owner);
                } else {
                    println!("Could not move group (may require admin privileges)");
                }
            }
        } else {
            println!("Not enough nodes or no online groups to test move");
        }
    }
}

/// Unit tests that don't require a cluster environment
mod unit_tests {
    use clus::ClusError;

    #[test]
    fn test_error_display() {
        let err = ClusError::OpenClusterFailed("TestCluster".to_string());
        assert_eq!(format!("{}", err), "Failed to open cluster: TestCluster");

        let err = ClusError::NotFound("TestResource".to_string());
        assert_eq!(format!("{}", err), "Resource not found: TestResource");

        let err = ClusError::InvalidHandle;
        assert_eq!(format!("{}", err), "Invalid cluster handle");

        let err = ClusError::OperationFailed("test operation".to_string());
        assert_eq!(
            format!("{}", err),
            "Cluster operation failed: test operation"
        );
    }

    #[test]
    fn test_error_debug() {
        let err = ClusError::OpenClusterFailed("TestCluster".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("OpenClusterFailed"));
        assert!(debug_str.contains("TestCluster"));
    }
}
