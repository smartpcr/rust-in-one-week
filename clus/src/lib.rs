//! Windows Failover Cluster library
//!
//! Provides Rust bindings for Windows Failover Cluster API operations.

#[cfg(windows)]
mod cluster;
#[cfg(windows)]
mod csv;
mod error;
#[cfg(windows)]
mod group;
#[cfg(windows)]
mod node;
#[cfg(windows)]
mod resource;
#[cfg(windows)]
mod utils;

#[cfg(windows)]
pub use cluster::Cluster;
#[cfg(windows)]
pub use csv::{
    Csv, CsvBackupState, CsvFaultState, CsvInfo, CsvSnapshotState, CsvState, RedirectedIoReason,
};
pub use error::{ClusError, Result};
#[cfg(windows)]
pub use group::{Group, GroupState};
#[cfg(windows)]
pub use node::{Node, NodeState};
#[cfg(windows)]
pub use resource::{Resource, ResourceState};
