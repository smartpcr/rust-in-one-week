//! Validation module for Hyper-V property and capability checks.
//!
//! This module provides utilities for:
//! - Checking if WMI properties are supported on the current host
//! - Validating VM version compatibility
//! - Querying host capabilities
//!
//! This mirrors the C++ wmiv2 `Supports*Property` family of functions.

mod capabilities;
mod property;

pub use capabilities::{HostCapabilities, VmVersionInfo};
pub use property::{PropertySupport, PropertyValidator};
