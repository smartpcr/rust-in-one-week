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

#[cfg(not(windows))]
compile_error!("windows-hyperv only supports Windows platforms");

#[cfg(windows)]
pub mod checkpoint;
#[cfg(windows)]
pub mod error;
#[cfg(windows)]
mod hyperv;
#[cfg(windows)]
pub mod network;
#[cfg(windows)]
pub mod storage;
#[cfg(windows)]
pub mod vm;
#[cfg(windows)]
pub mod wmi;

// Re-export main types at crate root
#[cfg(windows)]
pub use error::{Error, Result};
#[cfg(windows)]
pub use hyperv::HyperV;

// VM types
#[cfg(windows)]
pub use vm::{
    AutomaticStartAction, AutomaticStopAction, CheckpointType, Generation,
    OperationalStatus, RequestedState, ShutdownType, VirtualMachine, VmSettings,
    VmSettingsBuilder, VmState,
};

// Checkpoint types
#[cfg(windows)]
pub use checkpoint::{Checkpoint, CheckpointSettings, CheckpointSettingsBuilder, ConsistencyLevel};

// Storage types
#[cfg(windows)]
pub use storage::{
    ControllerType, DiskAttachment, IsoAttachment, StorageController,
    Vhd, VhdFormat, VhdManager, VhdSettings, VhdSettingsBuilder, VhdType,
};

// Network types
#[cfg(windows)]
pub use network::{
    BandwidthSettings, NetworkAdapter, NetworkAdapterSettings, NetworkAdapterSettingsBuilder,
    PortMirroringMode, SwitchType, VirtualSwitch, VirtualSwitchSettings,
    VirtualSwitchSettingsBuilder,
};
