//! Processor-related types for Hyper-V VMs.

use crate::error::{Error, Result};
use std::fmt;

/// CPU limit as a percentage (0-100%).
///
/// Internally stored as 0-100000 (units of 0.001%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CpuLimit(u64);

impl CpuLimit {
    /// No limit (100%).
    pub const NONE: Self = Self(100000);

    /// Minimum value (0%).
    pub const MIN: Self = Self(0);

    /// Create from percentage (0.0 - 100.0).
    pub fn from_percent(percent: f64) -> Option<Self> {
        if (0.0..=100.0).contains(&percent) {
            Some(Self((percent * 1000.0) as u64))
        } else {
            None
        }
    }

    /// Create from raw value (0-100000).
    pub fn from_raw(value: u64) -> Option<Self> {
        if value <= 100000 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get as percentage (0.0 - 100.0).
    pub fn as_percent(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Get raw value (0-100000).
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Default for CpuLimit {
    fn default() -> Self {
        Self::NONE
    }
}

impl fmt::Display for CpuLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.as_percent())
    }
}

/// CPU reservation as a percentage (0-100%).
///
/// Internally stored as 0-100000 (units of 0.001%).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CpuReservation(u64);

impl CpuReservation {
    /// No reservation.
    pub const NONE: Self = Self(0);

    /// Create from percentage (0.0 - 100.0).
    pub fn from_percent(percent: f64) -> Option<Self> {
        if (0.0..=100.0).contains(&percent) {
            Some(Self((percent * 1000.0) as u64))
        } else {
            None
        }
    }

    /// Create from raw value (0-100000).
    pub fn from_raw(value: u64) -> Option<Self> {
        if value <= 100000 {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get as percentage (0.0 - 100.0).
    pub fn as_percent(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Get raw value (0-100000).
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for CpuReservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.as_percent())
    }
}

/// CPU weight for relative priority (0-10000).
///
/// Higher weight means higher priority when competing for CPU resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CpuWeight(u32);

impl CpuWeight {
    /// Lowest weight.
    pub const LOW: Self = Self(1);

    /// Default weight.
    pub const DEFAULT: Self = Self(100);

    /// High weight.
    pub const HIGH: Self = Self(200);

    /// Maximum weight.
    pub const MAX: Self = Self(10000);

    /// Create from value (1-10000).
    pub fn new(value: u32) -> Option<Self> {
        if (1..=10000).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Get the weight value.
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for CpuWeight {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for CpuWeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// CPU group ID for CPU group assignment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CpuGroupId(pub String);

impl CpuGroupId {
    /// Create a new CPU group ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CpuGroupId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CpuGroupId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for CpuGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Re-implement without String to allow Copy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CpuGroupIdRef<'a>(pub &'a str);

/// Hardware threads per core (SMT/HyperThreading configuration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HwThreadsPerCore(u32);

impl HwThreadsPerCore {
    /// Single thread per core (disable SMT for guest).
    pub const ONE: Self = Self(1);

    /// Two threads per core (typical SMT).
    pub const TWO: Self = Self(2);

    /// Create from value.
    pub fn new(threads: u32) -> Result<Self> {
        if threads == 0 {
            return Err(Error::Validation {
                field: "hw_threads_per_core",
                message: "Threads per core must be at least 1".to_string(),
            });
        }
        // Typical max is 2, but some processors support more
        if threads > 8 {
            return Err(Error::Validation {
                field: "hw_threads_per_core",
                message: format!("Threads per core {} exceeds maximum of 8", threads),
            });
        }
        Ok(Self(threads))
    }

    /// Get the value.
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for HwThreadsPerCore {
    fn default() -> Self {
        Self::ONE
    }
}

impl fmt::Display for HwThreadsPerCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// L3 cache distribution policy for AMD processors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum L3DistributionPolicy {
    /// Default policy - let hypervisor decide.
    #[default]
    Default = 0,
    /// Round-robin distribution across L3 caches.
    RoundRobin = 1,
    /// Localized - keep VPs close to their L3.
    Localized = 2,
}

impl From<u32> for L3DistributionPolicy {
    fn from(value: u32) -> Self {
        match value {
            1 => L3DistributionPolicy::RoundRobin,
            2 => L3DistributionPolicy::Localized,
            _ => L3DistributionPolicy::Default,
        }
    }
}

impl fmt::Display for L3DistributionPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            L3DistributionPolicy::Default => write!(f, "Default"),
            L3DistributionPolicy::RoundRobin => write!(f, "Round Robin"),
            L3DistributionPolicy::Localized => write!(f, "Localized"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_limit_from_percent() {
        let limit = CpuLimit::from_percent(50.0).unwrap();
        assert_eq!(limit.as_percent(), 50.0);
        assert_eq!(limit.raw(), 50000);

        assert!(CpuLimit::from_percent(0.0).is_some());
        assert!(CpuLimit::from_percent(100.0).is_some());
        assert!(CpuLimit::from_percent(-1.0).is_none());
        assert!(CpuLimit::from_percent(100.1).is_none());
    }

    #[test]
    fn test_cpu_limit_display() {
        assert_eq!(format!("{}", CpuLimit::NONE), "100.0%");
        assert_eq!(format!("{}", CpuLimit::from_percent(50.0).unwrap()), "50.0%");
    }

    #[test]
    fn test_cpu_limit_default() {
        assert_eq!(CpuLimit::default(), CpuLimit::NONE);
    }

    #[test]
    fn test_cpu_reservation_from_percent() {
        let res = CpuReservation::from_percent(25.0).unwrap();
        assert_eq!(res.as_percent(), 25.0);
        assert_eq!(res.raw(), 25000);

        assert!(CpuReservation::from_percent(0.0).is_some());
        assert!(CpuReservation::from_percent(100.0).is_some());
        assert!(CpuReservation::from_percent(-1.0).is_none());
    }

    #[test]
    fn test_cpu_reservation_default() {
        assert_eq!(CpuReservation::default(), CpuReservation::NONE);
    }

    #[test]
    fn test_cpu_weight() {
        assert!(CpuWeight::new(0).is_none());
        assert!(CpuWeight::new(1).is_some());
        assert!(CpuWeight::new(10000).is_some());
        assert!(CpuWeight::new(10001).is_none());

        assert_eq!(CpuWeight::DEFAULT.value(), 100);
        assert!(CpuWeight::HIGH > CpuWeight::DEFAULT);
    }

    #[test]
    fn test_cpu_weight_display() {
        assert_eq!(format!("{}", CpuWeight::DEFAULT), "100");
    }

    #[test]
    fn test_hw_threads_per_core() {
        assert!(HwThreadsPerCore::new(0).is_err());
        assert!(HwThreadsPerCore::new(1).is_ok());
        assert!(HwThreadsPerCore::new(2).is_ok());
        assert!(HwThreadsPerCore::new(8).is_ok());
        assert!(HwThreadsPerCore::new(9).is_err());

        assert_eq!(HwThreadsPerCore::ONE.value(), 1);
        assert_eq!(HwThreadsPerCore::TWO.value(), 2);
    }

    #[test]
    fn test_l3_distribution_policy() {
        assert_eq!(L3DistributionPolicy::from(0), L3DistributionPolicy::Default);
        assert_eq!(L3DistributionPolicy::from(1), L3DistributionPolicy::RoundRobin);
        assert_eq!(L3DistributionPolicy::from(2), L3DistributionPolicy::Localized);
        assert_eq!(L3DistributionPolicy::from(99), L3DistributionPolicy::Default);
    }

    #[test]
    fn test_cpu_group_id() {
        let id = CpuGroupId::new("test-group");
        assert_eq!(id.as_str(), "test-group");
        assert_eq!(format!("{}", id), "test-group");

        let id2: CpuGroupId = "another-group".into();
        assert_eq!(id2.as_str(), "another-group");
    }
}
