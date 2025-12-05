//! Strong types for VM configuration values.
//!
//! These types provide compile-time validation and clear semantics for
//! VM configuration parameters.

use core::fmt;

/// Memory size in megabytes.
///
/// Validates that memory is within Hyper-V limits:
/// - Minimum: 32 MB
/// - Maximum: 12 TB (12,582,912 MB)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryMB(u64);

impl MemoryMB {
    /// Minimum memory in MB (32 MB).
    pub const MIN: u64 = 32;
    /// Maximum memory in MB (12 TB).
    pub const MAX: u64 = 12 * 1024 * 1024; // 12 TB in MB

    /// Create from megabytes.
    ///
    /// Returns `None` if outside valid range (32 MB - 12 TB).
    pub fn new(mb: u64) -> Option<Self> {
        if mb >= Self::MIN && mb <= Self::MAX {
            Some(Self(mb))
        } else {
            None
        }
    }

    /// Create from gigabytes.
    pub fn from_gb(gb: u64) -> Option<Self> {
        Self::new(gb.saturating_mul(1024))
    }

    /// Get value in megabytes.
    pub fn as_mb(&self) -> u64 {
        self.0
    }

    /// Get value in gigabytes (rounded down).
    pub fn as_gb(&self) -> u64 {
        self.0 / 1024
    }

    /// Get value in bytes.
    pub fn as_bytes(&self) -> u64 {
        self.0.saturating_mul(1024 * 1024)
    }

    /// Common presets
    pub const fn mb_512() -> Self {
        Self(512)
    }
    pub const fn gb_1() -> Self {
        Self(1024)
    }
    pub const fn gb_2() -> Self {
        Self(2048)
    }
    pub const fn gb_4() -> Self {
        Self(4096)
    }
    pub const fn gb_8() -> Self {
        Self(8192)
    }
    pub const fn gb_16() -> Self {
        Self(16384)
    }
    pub const fn gb_32() -> Self {
        Self(32768)
    }
}

impl Default for MemoryMB {
    fn default() -> Self {
        Self::gb_1()
    }
}

impl fmt::Display for MemoryMB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 1024 && self.0 % 1024 == 0 {
            write!(f, "{} GB", self.0 / 1024)
        } else {
            write!(f, "{} MB", self.0)
        }
    }
}

/// Virtual processor count.
///
/// Validates that processor count is within Hyper-V limits:
/// - Minimum: 1
/// - Maximum: 240
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessorCount(u32);

impl ProcessorCount {
    /// Minimum processor count.
    pub const MIN: u32 = 1;
    /// Maximum processor count.
    pub const MAX: u32 = 240;

    /// Create a new processor count.
    ///
    /// Returns `None` if outside valid range (1-240).
    pub fn new(count: u32) -> Option<Self> {
        if count >= Self::MIN && count <= Self::MAX {
            Some(Self(count))
        } else {
            None
        }
    }

    /// Get the processor count.
    pub fn get(&self) -> u32 {
        self.0
    }

    /// Common presets
    pub const fn one() -> Self {
        Self(1)
    }
    pub const fn two() -> Self {
        Self(2)
    }
    pub const fn four() -> Self {
        Self(4)
    }
    pub const fn eight() -> Self {
        Self(8)
    }
}

impl Default for ProcessorCount {
    fn default() -> Self {
        Self::one()
    }
}

impl fmt::Display for ProcessorCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} vCPU{}", self.0, if self.0 == 1 { "" } else { "s" })
    }
}

/// Memory buffer percentage for dynamic memory (0-100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MemoryBufferPercent(u32);

impl MemoryBufferPercent {
    /// Create a new memory buffer percentage.
    ///
    /// Returns `None` if percentage > 100.
    pub fn new(percent: u32) -> Option<Self> {
        if percent <= 100 {
            Some(Self(percent))
        } else {
            None
        }
    }

    /// Get the percentage value.
    pub fn get(&self) -> u32 {
        self.0
    }

    /// Default 20% buffer.
    pub const fn default_20() -> Self {
        Self(20)
    }
}

impl fmt::Display for MemoryBufferPercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

/// SCSI controller location (0-63) or IDE location (0-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiskLocation(u32);

impl DiskLocation {
    /// Maximum SCSI location.
    pub const MAX_SCSI: u32 = 63;
    /// Maximum IDE location.
    pub const MAX_IDE: u32 = 1;

    /// Create a SCSI disk location (0-63).
    pub fn scsi(location: u32) -> Option<Self> {
        if location <= Self::MAX_SCSI {
            Some(Self(location))
        } else {
            None
        }
    }

    /// Create an IDE disk location (0-1).
    pub fn ide(location: u32) -> Option<Self> {
        if location <= Self::MAX_IDE {
            Some(Self(location))
        } else {
            None
        }
    }

    /// Get the location value.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl Default for DiskLocation {
    fn default() -> Self {
        Self(0)
    }
}

impl fmt::Display for DiskLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Location {}", self.0)
    }
}

/// Sector size for VHD/VHDX (512 or 4096 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectorSize {
    /// 512 bytes (legacy).
    Bytes512,
    /// 4096 bytes (4K native).
    Bytes4K,
}

impl SectorSize {
    /// Get the size in bytes.
    pub fn as_bytes(&self) -> u32 {
        match self {
            SectorSize::Bytes512 => 512,
            SectorSize::Bytes4K => 4096,
        }
    }

    /// Parse from bytes value.
    pub fn from_bytes(bytes: u32) -> Option<Self> {
        match bytes {
            512 => Some(SectorSize::Bytes512),
            4096 => Some(SectorSize::Bytes4K),
            _ => None,
        }
    }
}

impl Default for SectorSize {
    fn default() -> Self {
        SectorSize::Bytes512
    }
}

impl fmt::Display for SectorSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectorSize::Bytes512 => write!(f, "512 bytes"),
            SectorSize::Bytes4K => write!(f, "4K"),
        }
    }
}

/// VHD block size.
///
/// Valid values: 512 KB, 1 MB, 2 MB, 16 MB, 32 MB, 64 MB, 128 MB, 256 MB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockSize(u32);

impl BlockSize {
    /// 512 KB block size.
    pub const KB_512: Self = Self(512 * 1024);
    /// 1 MB block size.
    pub const MB_1: Self = Self(1024 * 1024);
    /// 2 MB block size (VHDX default for data disks).
    pub const MB_2: Self = Self(2 * 1024 * 1024);
    /// 16 MB block size.
    pub const MB_16: Self = Self(16 * 1024 * 1024);
    /// 32 MB block size (VHDX default).
    pub const MB_32: Self = Self(32 * 1024 * 1024);
    /// 64 MB block size.
    pub const MB_64: Self = Self(64 * 1024 * 1024);
    /// 128 MB block size.
    pub const MB_128: Self = Self(128 * 1024 * 1024);
    /// 256 MB block size.
    pub const MB_256: Self = Self(256 * 1024 * 1024);

    /// Create from bytes.
    pub fn from_bytes(bytes: u32) -> Option<Self> {
        match bytes {
            b if b == Self::KB_512.0 => Some(Self::KB_512),
            b if b == Self::MB_1.0 => Some(Self::MB_1),
            b if b == Self::MB_2.0 => Some(Self::MB_2),
            b if b == Self::MB_16.0 => Some(Self::MB_16),
            b if b == Self::MB_32.0 => Some(Self::MB_32),
            b if b == Self::MB_64.0 => Some(Self::MB_64),
            b if b == Self::MB_128.0 => Some(Self::MB_128),
            b if b == Self::MB_256.0 => Some(Self::MB_256),
            _ => None,
        }
    }

    /// Get the size in bytes.
    pub fn as_bytes(&self) -> u32 {
        self.0
    }
}

impl Default for BlockSize {
    fn default() -> Self {
        Self::MB_32
    }
}

impl fmt::Display for BlockSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 1024 * 1024 {
            write!(f, "{} MB", self.0 / (1024 * 1024))
        } else {
            write!(f, "{} KB", self.0 / 1024)
        }
    }
}

/// VHD/VHDX disk size.
///
/// Validates size limits:
/// - VHD: max 2 TB
/// - VHDX: max 64 TB
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiskSize(u64);

impl DiskSize {
    /// Maximum VHD size (2 TB).
    pub const MAX_VHD: u64 = 2 * 1024 * 1024 * 1024 * 1024;
    /// Maximum VHDX size (64 TB).
    pub const MAX_VHDX: u64 = 64 * 1024 * 1024 * 1024 * 1024;

    /// Create from bytes.
    pub fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Create from gigabytes.
    pub fn from_gb(gb: u64) -> Self {
        Self(gb.saturating_mul(1024 * 1024 * 1024))
    }

    /// Create from terabytes.
    pub fn from_tb(tb: u64) -> Self {
        Self(tb.saturating_mul(1024 * 1024 * 1024 * 1024))
    }

    /// Get size in bytes.
    pub fn as_bytes(&self) -> u64 {
        self.0
    }

    /// Get size in gigabytes (rounded down).
    pub fn as_gb(&self) -> u64 {
        self.0 / (1024 * 1024 * 1024)
    }

    /// Check if valid for VHD format.
    pub fn is_valid_vhd(&self) -> bool {
        self.0 > 0 && self.0 <= Self::MAX_VHD
    }

    /// Check if valid for VHDX format.
    pub fn is_valid_vhdx(&self) -> bool {
        self.0 > 0 && self.0 <= Self::MAX_VHDX
    }
}

impl fmt::Display for DiskSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tb = 1024 * 1024 * 1024 * 1024u64;
        let gb = 1024 * 1024 * 1024u64;
        if self.0 >= tb && self.0 % tb == 0 {
            write!(f, "{} TB", self.0 / tb)
        } else if self.0 >= gb && self.0 % gb == 0 {
            write!(f, "{} GB", self.0 / gb)
        } else {
            write!(f, "{} bytes", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_mb_valid_range() {
        assert!(MemoryMB::new(32).is_some());
        assert!(MemoryMB::new(1024).is_some());
        assert!(MemoryMB::new(12 * 1024 * 1024).is_some());
        assert!(MemoryMB::new(31).is_none());
        assert!(MemoryMB::new(12 * 1024 * 1024 + 1).is_none());
    }

    #[test]
    fn test_memory_mb_from_gb() {
        assert_eq!(MemoryMB::from_gb(1).unwrap().as_mb(), 1024);
        assert_eq!(MemoryMB::from_gb(4).unwrap().as_mb(), 4096);
    }

    #[test]
    fn test_memory_mb_display() {
        assert_eq!(format!("{}", MemoryMB::new(512).unwrap()), "512 MB");
        assert_eq!(format!("{}", MemoryMB::new(1024).unwrap()), "1 GB");
        assert_eq!(format!("{}", MemoryMB::new(2048).unwrap()), "2 GB");
        assert_eq!(format!("{}", MemoryMB::new(1536).unwrap()), "1536 MB");
    }

    #[test]
    fn test_processor_count_valid_range() {
        assert!(ProcessorCount::new(1).is_some());
        assert!(ProcessorCount::new(240).is_some());
        assert!(ProcessorCount::new(0).is_none());
        assert!(ProcessorCount::new(241).is_none());
    }

    #[test]
    fn test_processor_count_display() {
        assert_eq!(format!("{}", ProcessorCount::new(1).unwrap()), "1 vCPU");
        assert_eq!(format!("{}", ProcessorCount::new(4).unwrap()), "4 vCPUs");
    }

    #[test]
    fn test_memory_buffer_percent() {
        assert!(MemoryBufferPercent::new(0).is_some());
        assert!(MemoryBufferPercent::new(100).is_some());
        assert!(MemoryBufferPercent::new(101).is_none());
    }

    #[test]
    fn test_disk_location() {
        assert!(DiskLocation::scsi(0).is_some());
        assert!(DiskLocation::scsi(63).is_some());
        assert!(DiskLocation::scsi(64).is_none());
        assert!(DiskLocation::ide(0).is_some());
        assert!(DiskLocation::ide(1).is_some());
        assert!(DiskLocation::ide(2).is_none());
    }

    #[test]
    fn test_sector_size() {
        assert_eq!(SectorSize::from_bytes(512), Some(SectorSize::Bytes512));
        assert_eq!(SectorSize::from_bytes(4096), Some(SectorSize::Bytes4K));
        assert_eq!(SectorSize::from_bytes(1024), None);
    }

    #[test]
    fn test_block_size() {
        assert_eq!(
            BlockSize::from_bytes(32 * 1024 * 1024),
            Some(BlockSize::MB_32)
        );
        assert_eq!(BlockSize::MB_32.as_bytes(), 32 * 1024 * 1024);
    }

    #[test]
    fn test_disk_size() {
        let size = DiskSize::from_gb(100);
        assert_eq!(size.as_gb(), 100);
        assert!(size.is_valid_vhd());
        assert!(size.is_valid_vhdx());

        let large = DiskSize::from_tb(3);
        assert!(!large.is_valid_vhd());
        assert!(large.is_valid_vhdx());
    }

    #[test]
    fn test_disk_size_display() {
        assert_eq!(format!("{}", DiskSize::from_gb(100)), "100 GB");
        assert_eq!(format!("{}", DiskSize::from_tb(2)), "2 TB");
    }
}
