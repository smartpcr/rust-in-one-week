# Windows Infrastructure Management API

REST API for Windows Failover Cluster and Hyper-V management.

## Features

- **Failover Cluster Management**: Nodes, groups, resources, and Cluster Shared Volumes (CSV)
- **Hyper-V Management**: VMs, VHDs, snapshots, switches, GPU (GPU-P and DDA)
- Cross-platform build support (Windows-only functionality returns appropriate errors on other platforms)

## Project Structure

```
api/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── main.rs         # Binary entry point
│   ├── dto.rs          # Data Transfer Objects
│   ├── response.rs     # API response types
│   ├── routes.rs       # Route definitions
│   └── handlers/
│       ├── mod.rs      # Handler module
│       ├── cluster.rs  # Cluster API handlers
│       └── hyperv.rs   # Hyper-V API handlers
└── tests/
    └── integration_tests.rs
```

## Build

```bash
cargo build -p api
```

## Run

```bash
cargo run -p api
```

The server starts on `http://0.0.0.0:3000`.

## Test

```bash
cargo test -p api
```

## API Endpoints

### Root

- `GET /` - API info
- `GET /health` - Health check

### Cluster API (`/api/v1/cluster`)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | Get cluster info |
| GET | `/connect/{name}` | Connect to cluster |
| GET | `/nodes` | List nodes |
| GET | `/nodes/{name}` | Get node |
| POST | `/nodes/{name}/pause` | Pause node |
| POST | `/nodes/{name}/resume` | Resume node |
| GET | `/groups` | List groups |
| GET | `/groups/{name}` | Get group |
| POST | `/groups/{name}/online` | Bring group online |
| POST | `/groups/{name}/offline` | Take group offline |
| POST | `/groups/{name}/move/{target}` | Move group to node |
| GET | `/resources` | List resources |
| GET | `/resources/{name}` | Get resource |
| POST | `/resources/{name}/online` | Bring resource online |
| POST | `/resources/{name}/offline` | Take resource offline |
| GET | `/csv` | List CSV volumes |
| GET | `/csv/check-path` | Check if path is on CSV |
| POST | `/csv/{name}/maintenance` | Set maintenance mode |

### Hyper-V API (`/api/v1/hyperv`)

#### Host & Network

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/host` | Get host info |
| GET | `/adapters` | List network adapters |

#### Virtual Machines

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/vms` | List VMs |
| POST | `/vms` | Create VM |
| GET | `/vms/{name}` | Get VM |
| DELETE | `/vms/{name}` | Delete VM |
| POST | `/vms/{name}/start` | Start VM |
| POST | `/vms/{name}/stop` | Stop VM (graceful) |
| POST | `/vms/{name}/force-stop` | Force stop VM |
| POST | `/vms/{name}/pause` | Pause VM |
| POST | `/vms/{name}/resume` | Resume VM |
| POST | `/vms/{name}/save` | Save VM state |
| POST | `/vms/{name}/reset` | Reset VM |
| POST | `/vms/{name}/export` | Export VM |

#### VM Disks

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/vms/{name}/disks` | List VM disks |
| POST | `/vms/{name}/disks/attach` | Attach disk |
| POST | `/vms/{name}/disks/detach` | Detach disk |
| GET | `/vms/{name}/dvd` | Get DVD drives |
| POST | `/vms/{name}/dvd/mount` | Mount ISO |
| POST | `/vms/{name}/dvd/eject` | Eject ISO |
| POST | `/vms/{name}/boot-order` | Set boot order |

#### VM Snapshots

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/vms/{name}/snapshots` | List snapshots |
| POST | `/vms/{name}/snapshots` | Create snapshot |
| GET | `/vms/{name}/snapshots/{snap}` | Get snapshot |
| POST | `/vms/{name}/snapshots/{snap}/apply` | Apply snapshot |
| DELETE | `/vms/{name}/snapshots/{snap}/delete` | Delete snapshot |

#### VM GPU (GPU-P)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/vms/{name}/gpu` | Get GPU adapters |
| POST | `/vms/{name}/gpu/add` | Add GPU |
| POST | `/vms/{name}/gpu/remove` | Remove GPU |
| POST | `/vms/{name}/gpu/configure` | Configure GPU MMIO |

#### VM DDA (Discrete Device Assignment)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/vms/{name}/dda` | Get assigned devices |
| POST | `/vms/{name}/dda/assign` | Assign device |
| POST | `/vms/{name}/dda/remove` | Remove device |

#### Virtual Switches

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/switches` | List switches |
| POST | `/switches` | Create switch |
| GET | `/switches/{name}` | Get switch |
| DELETE | `/switches/{name}` | Delete switch |

#### VHDs

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/vhds` | Create VHD |
| GET | `/vhds/info` | Get VHD info |
| POST | `/vhds/resize` | Resize VHD |
| POST | `/vhds/compact` | Compact VHD |
| POST | `/vhds/mount` | Mount VHD |
| POST | `/vhds/dismount` | Dismount VHD |
| POST | `/vhds/differencing` | Create differencing VHD |
| POST | `/vhds/initialize` | Initialize VHD |

#### Windows Image

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/iso/editions` | List Windows editions in ISO |
| POST | `/iso/create-vhdx` | Create VHDX from ISO |

#### GPUs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/gpus` | List all GPUs |
| GET | `/gpus/partitionable` | List partitionable GPUs |

#### DDA (Discrete Device Assignment)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/dda/support` | Check DDA support |
| GET | `/dda/devices` | List assignable devices |
| GET | `/dda/device-path` | Get device location path |
| POST | `/dda/dismount` | Dismount device from host |
| POST | `/dda/mount` | Mount device to host |

## Response Format

All API responses follow this format:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

Error responses:

```json
{
  "success": false,
  "data": null,
  "error": "Error message"
}
```

## Platform Support

This API is designed for Windows Server with Failover Clustering and Hyper-V roles. On non-Windows platforms, the API will build and run but return `501 Not Implemented` for cluster and Hyper-V endpoints.

## Dependencies

- `axum` - Web framework
- `tokio` - Async runtime
- `tower-http` - HTTP middleware (tracing, CORS)
- `serde` / `serde_json` - Serialization
- `clus` - Failover Cluster bindings (Windows only)
- `hv` - Hyper-V bindings (Windows only)
