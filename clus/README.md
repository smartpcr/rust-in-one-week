# clus

Rust bindings for Windows Failover Cluster API.

## Features

- Cluster connection management
- Node enumeration and operations (state, pause, resume)
- Resource enumeration and operations (state, online, offline)
- Group (role) enumeration and operations (state, online, offline, move)
- CSV (Cluster Shared Volume) management (list, info, maintenance mode)
- Safe handle management with automatic cleanup (RAII)
- Proper error handling with `thiserror`

## Modules

- `cluster` - Cluster handle, connection, and enumeration
- `node` - Cluster node operations
- `resource` - Cluster resource operations
- `group` - Cluster group (role) operations
- `csv` - Cluster Shared Volume operations
- `error` - Error types
- `utils` - String conversion helpers (UTF-16)

## Usage

```rust
use clus::{Cluster, ClusError};

fn main() -> Result<(), ClusError> {
    // Connect to local cluster
    let cluster = Cluster::open(None)?;

    // Or connect to a named cluster
    // let cluster = Cluster::open(Some("MyCluster"))?;

    // Get cluster name
    let name = cluster.name()?;
    println!("Connected to cluster: {}", name);

    // Enumerate all nodes
    let nodes = cluster.nodes()?;
    for node in &nodes {
        println!("Node: {} - {:?}", node.name(), node.state());
    }

    // Enumerate all resources
    let resources = cluster.resources()?;
    for resource in &resources {
        let (state, owner) = resource.state()?;
        println!("Resource: {} - {:?} on {:?}", resource.name(), state, owner);
    }

    // Enumerate all groups
    let groups = cluster.groups()?;
    for group in &groups {
        let (state, owner) = group.state()?;
        println!("Group: {} - {:?} on {:?}", group.name(), state, owner);
    }

    // Open specific resources
    let resource = cluster.open_resource("SQL Server")?;
    resource.offline()?;
    resource.online()?;

    // Move a group to a specific node
    let group = cluster.open_group("SQL Server Group")?;
    let node = cluster.open_node("Node2")?;
    group.move_to(&node)?;

    Ok(())
}
```

## API Reference

### Cluster

| Method | Description |
|--------|-------------|
| `open(name)` | Open connection to cluster (None for local) |
| `name()` | Get cluster name |
| `nodes()` | Enumerate all nodes |
| `resources()` | Enumerate all resources |
| `groups()` | Enumerate all groups |
| `open_node(name)` | Open specific node by name |
| `open_resource(name)` | Open specific resource by name |
| `open_group(name)` | Open specific group by name |

### Node

| Method | Description |
|--------|-------------|
| `name()` | Get node name |
| `state()` | Get node state (Up, Down, Paused, Joining) |
| `pause()` | Pause the node |
| `resume()` | Resume a paused node |

### Resource

| Method | Description |
|--------|-------------|
| `name()` | Get resource name |
| `state()` | Get state and owner node |
| `online()` | Bring resource online |
| `offline()` | Take resource offline |

### Group

| Method | Description |
|--------|-------------|
| `name()` | Get group name |
| `state()` | Get state and owner node |
| `online()` | Bring group online |
| `offline()` | Take group offline |
| `move_to(node)` | Move group to specific node |

### Csv (Cluster Shared Volume)

| Method | Description |
|--------|-------------|
| `is_path_on_csv(path)` | Check if a path is on a CSV |
| `get_volume_path(path)` | Get CSV mount point for a file path |
| `get_volume_name(mount)` | Get volume GUID for a mount point |
| `is_csv_resource(res)` | Check if a resource is a CSV |
| `set_maintenance_mode(res, enable)` | Enable/disable CSV maintenance mode |
| `get_volume_info(res)` | Get detailed CSV volume information |
| `set_snapshot_state(...)` | Set CSV snapshot state for VSS |

### Cluster CSV Methods

| Method | Description |
|--------|-------------|
| `csv_volumes()` | Enumerate all CSVs in the cluster |
| `csv_info()` | Get detailed info for all CSVs |

## Examples

```bash
# List cluster nodes, resources, and groups
cargo run -p clus --example list_cluster

# Move a VM to another node
cargo run -p clus --example move_vm -- "VM Name" "TargetNode"

# List all CSVs
cargo run -p clus --example csv_info -- list

# Get detailed CSV info
cargo run -p clus --example csv_info -- info

# Check if a path is on a CSV
cargo run -p clus --example csv_info -- check-path "C:\ClusterStorage\Volume1\data.txt"

# Set CSV maintenance mode
cargo run -p clus --example csv_info -- maintenance "Cluster Disk 1" on
cargo run -p clus --example csv_info -- maintenance "Cluster Disk 1" off
```

## Requirements

- Windows with Failover Clustering feature installed
- Appropriate cluster permissions

### Verifying a lab or CI runner

These quick checks help confirm the runner really has a local cluster before running CI jobs:

```powershell
Get-Cluster
Get-ClusterNode
Get-Service ClusSvc   # Should report Running

# Confirm the static address you plan to use is free
Test-Connection 172.16.0.100 -Count 1 -Quiet  # Expect False if unused
```

If any of these checks fail on a GitHub-hosted runner, switch the workflow to a self-hosted
Windows Server runner that already has the Failover Clustering feature enabled and is part of
the target cluster.

### Why GitHub Actions run 19843620672/56856879143 failed

- The `cluster-tests` job currently targets the GitHub-hosted `windows-2025` image (`runs-on: [windows-2025]`). That image does not include the Failover Clustering feature or a configured cluster, even though the workflow comments state the job is intended for a self-hosted Windows Server cluster runner.【F:.github/workflows/cluster-tests.yml†L22-L38】
- The first step after system info enforces those prerequisites by checking the Failover Clustering feature, the `ClusSvc` service, and connectivity to a cluster. On the hosted runner the feature check fails, so the workflow exits with a fatal error before any Rust build or tests run.【F:.github/workflows/cluster-tests.yml†L57-L112】
- Fix: point `runs-on` to a properly prepared self-hosted runner (labels: `self-hosted, windows, cluster`) or gate the cluster checks to skip when such a runner is unavailable.【F:.github/workflows/cluster-tests.yml†L22-L38】【F:.github/workflows/cluster-tests.yml†L57-L112】
