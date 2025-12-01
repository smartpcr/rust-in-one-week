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
