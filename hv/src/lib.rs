//! Windows Hyper-V management library
//!
//! Provides Rust bindings for managing Hyper-V virtual machines, switches,
//! virtual hard disks, and snapshots via HCS (Host Compute Service) and
//! native Windows APIs.
//!
//! # Architecture
//!
//! This library uses:
//! - **hcs-rs**: For compute system (VM) management via Host Compute Service
//! - **windows-rs**: For VHD operations and low-level Windows APIs
//!
//! # Example
//!
//! ```ignore
//! use hv::{HyperV, VmState};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let hyperv = HyperV::new()?;
//!
//!     // List all VMs
//!     for vm in hyperv.list_vms()? {
//!         println!("{}: {:?}", vm.name(), vm.state()?);
//!     }
//!
//!     Ok(())
//! }
//! ```

#[cfg(windows)]
mod disk;
mod error;
#[cfg(windows)]
mod gpu;
#[cfg(windows)]
mod hcs;
#[cfg(windows)]
mod hyperv;
#[cfg(windows)]
mod snapshot;
#[cfg(windows)]
mod switch;
#[cfg(windows)]
mod vhd;
#[cfg(windows)]
mod vm;

#[cfg(windows)]
pub use disk::{DvdDrive, FileSystem, HardDiskDrive, PartitionStyle, WindowsEdition};
pub use error::{HvError, Result};
#[cfg(windows)]
pub use gpu::{AssignableDevice, DdaSupportInfo, GpuInfo, GpuPartitionAdapter};
#[cfg(windows)]
pub use hyperv::HyperV;
#[cfg(windows)]
pub use snapshot::{Snapshot, SnapshotType};
#[cfg(windows)]
pub use switch::{SwitchType, VirtualSwitch};
#[cfg(windows)]
pub use vhd::{Vhd, VhdFormat, VhdType};
#[cfg(windows)]
pub use vm::{Vm, VmGeneration, VmState};
