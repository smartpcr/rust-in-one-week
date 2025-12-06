//! # windows-hyperv
//!
//! Typed Hyper-V management API for Windows.
//!
//! This crate provides strongly-typed Rust bindings for Hyper-V VM management operations,
//! built on top of the WMI-based Hyper-V management APIs (`root\virtualization\v2`).
//!
//! ## Features
//!
//! - **Type-safe VM operations**: Create, start, stop, delete VMs with compile-time type checking
//! - **Builder pattern**: Configure VMs with validated settings
//! - **Proper error handling**: Typed errors instead of generic WMI failures
//! - **Full VM lifecycle**: Memory, processor, storage, network, and checkpoint management
//!
//! ## Example
//!
//! ```no_run
//! use windows_hyperv::{HyperV, VmSettings, Generation};
//!
//! fn main() -> windows_hyperv::Result<()> {
//!     let hyperv = HyperV::connect()?;
//!
//!     // List all VMs
//!     for vm in hyperv.list_vms()? {
//!         println!("{}: {:?}", vm.name(), vm.state());
//!     }
//!
//!     // Create a new VM
//!     let settings = VmSettings::builder()
//!         .name("MyVM")
//!         .generation(Generation::Gen2)
//!         .memory_mb(4096)
//!         .processor_count(2)
//!         .build()?;
//!
//!     let mut vm = hyperv.create_vm(&settings)?;
//!     vm.start()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Requirements
//!
//! - Windows 10/11 or Windows Server 2016+
//! - Hyper-V feature enabled
//! - Administrator privileges

#[cfg(windows)]
pub mod checkpoint;
#[cfg(windows)]
pub mod error;
#[cfg(windows)]
pub mod gpu;
#[cfg(windows)]
mod hyperv;
#[cfg(windows)]
pub mod network;
#[cfg(windows)]
pub mod processor;
#[cfg(windows)]
pub mod security;
#[cfg(windows)]
pub mod storage;
#[cfg(windows)]
pub mod validation;
#[cfg(windows)]
pub mod vm;
#[cfg(windows)]
pub mod wmi;

// Re-export main types at crate root
#[cfg(windows)]
pub use error::{Error, FailureType, JobState, MigrationError, Result, SecurityError};
#[cfg(windows)]
pub use hyperv::HyperV;

// VM types
#[cfg(windows)]
pub use vm::{
    AutomaticStartAction, AutomaticStopAction, BlockSize, CaptureLiveState, CheckpointType,
    DiskLocation, DiskSize, ExportSettings, Generation, ImportSettings, MemoryBufferPercent,
    MemoryMB, OperationalStatus, OperationalStatusSecondary, ProcessorCount, RequestedState,
    SectorSize, ShutdownType, SnapshotExportMode, StartupDelay, VirtualMachine, VmSettings,
    VmSettingsBuilder, VmState,
};

// Checkpoint types
#[cfg(windows)]
pub use checkpoint::{Checkpoint, CheckpointSettings, CheckpointSettingsBuilder, ConsistencyLevel};

// Storage types
#[cfg(windows)]
pub use storage::{
    ControllerType, DiskAttachment, IsoAttachment, StorageController, Vhd, VhdFormat, VhdManager,
    VhdSettings, VhdSettingsBuilder, VhdType,
};

// Network types
#[cfg(windows)]
pub use network::{
    BandwidthSettings, NetworkAdapter, NetworkAdapterSettings, NetworkAdapterSettingsBuilder,
    PortMirroringMode, SwitchType, VirtualSwitch, VirtualSwitchSettings,
    VirtualSwitchSettingsBuilder,
};

// WMI types for advanced usage
#[cfg(windows)]
pub use wmi::{
    ConnectionConfig, Credentials, JobProgress, JobWaiter, WbemClassObjectExt, WmiConnection,
};

// Validation types
#[cfg(windows)]
pub use validation::{HostCapabilities, PropertySupport, PropertyValidator, VmVersionInfo};

// Security types
#[cfg(windows)]
pub use security::{
    FirmwareType, GuestIsolationType, KeyProtectorType, SecureBootTemplate, SecuritySettings,
    SecuritySettingsBuilder, TpmState,
};

// Processor types
#[cfg(windows)]
pub use processor::{
    CpuGroupId, CpuLimit, CpuReservation, CpuWeight, HwThreadsPerCore, L3DistributionPolicy,
    NumaNode, NumaTopology, ProcessorSettings, ProcessorSettingsBuilder,
};
