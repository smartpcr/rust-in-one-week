//! Advanced processor settings for Hyper-V VMs.
//!
//! This module provides support for:
//! - CPU count and resource controls (limit, reservation, weight)
//! - NUMA topology configuration
//! - Hardware thread configuration (SMT/HyperThreading)
//! - CPU groups
//! - Nested virtualization
//! - AMD CCX/CCD topology
//!
//! # Example
//!
//! ```no_run
//! use windows_hyperv::processor::{ProcessorSettings, CpuLimit, CpuWeight};
//!
//! let settings = ProcessorSettings::builder()
//!     .count(4)
//!     .limit(CpuLimit::from_percent(50.0).unwrap())
//!     .weight(CpuWeight::HIGH)
//!     .expose_virtualization_extensions(true)
//!     .build()?;
//! # Ok::<(), windows_hyperv::Error>(())
//! ```

mod settings;
mod topology;
mod types;

pub use settings::{ProcessorSettings, ProcessorSettingsBuilder};
pub use topology::{NumaNode, NumaTopology};
pub use types::{
    CpuGroupId, CpuLimit, CpuReservation, CpuWeight, HwThreadsPerCore,
    L3DistributionPolicy,
};
