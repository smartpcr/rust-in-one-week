//! Example: List all cluster nodes, groups, and resources

#[cfg(windows)]
use clus::{Cluster, GroupState, NodeState, ResourceState};

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to local cluster
    let cluster = Cluster::open(None)?;
    println!("Connected to cluster: {}", cluster.name()?);
    println!();

    // List all nodes
    println!("=== Nodes ===");
    let nodes = cluster.nodes()?;
    for node in &nodes {
        let state = node.state();
        let status = match state {
            NodeState::Up => "✓ Up",
            NodeState::Down => "✗ Down",
            NodeState::Paused => "⏸ Paused",
            NodeState::Joining => "… Joining",
            NodeState::Unknown(n) => &format!("? Unknown({})", n),
        };
        println!("  {} - {}", node.name(), status);
    }
    println!();

    // List all groups (roles)
    println!("=== Groups ===");
    let groups = cluster.groups()?;
    for group in &groups {
        let (state, owner) = group.state()?;
        let status = match state {
            GroupState::Online => "Online",
            GroupState::Offline => "Offline",
            GroupState::Failed => "FAILED",
            GroupState::PartialOnline => "Partial",
            GroupState::Pending => "Pending",
            GroupState::Unknown(_) => "Unknown",
        };
        let owner_str = owner.as_deref().unwrap_or("(none)");
        println!("  {} - {} on {}", group.name(), status, owner_str);
    }
    println!();

    // List all resources
    println!("=== Resources ===");
    let resources = cluster.resources()?;
    for resource in &resources {
        let (state, owner) = resource.state()?;
        let status = match state {
            ResourceState::Online => "Online",
            ResourceState::Offline => "Offline",
            ResourceState::Failed => "FAILED",
            ResourceState::OnlinePending => "Starting",
            ResourceState::OfflinePending => "Stopping",
            ResourceState::Unknown(_) => "Unknown",
        };
        let owner_str = owner.as_deref().unwrap_or("(none)");
        println!("  {} - {} on {}", resource.name(), status, owner_str);
    }

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only runs on Windows with Failover Clustering installed.");
}
