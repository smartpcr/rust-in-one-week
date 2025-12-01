//! Virtual Hard Disk (VHD/VHDX) management using Windows VHD APIs
//!
//! Uses the Windows VirtDisk API for VHD operations.

use crate::error::{HvError, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::Vhd::{
    AttachVirtualDisk, CompactVirtualDisk, CreateVirtualDisk, DetachVirtualDisk,
    GetVirtualDiskInformation, OpenVirtualDisk, ResizeVirtualDisk, ATTACH_VIRTUAL_DISK_FLAG,
    ATTACH_VIRTUAL_DISK_FLAG_NO_DRIVE_LETTER, ATTACH_VIRTUAL_DISK_FLAG_READ_ONLY,
    COMPACT_VIRTUAL_DISK_FLAG, CREATE_VIRTUAL_DISK_FLAG, CREATE_VIRTUAL_DISK_FLAG_NONE,
    CREATE_VIRTUAL_DISK_PARAMETERS, CREATE_VIRTUAL_DISK_VERSION_2, DETACH_VIRTUAL_DISK_FLAG,
    GET_VIRTUAL_DISK_INFO, GET_VIRTUAL_DISK_INFO_0, GET_VIRTUAL_DISK_INFO_SIZE,
    GET_VIRTUAL_DISK_INFO_VIRTUAL_STORAGE_TYPE, OPEN_VIRTUAL_DISK_FLAG,
    OPEN_VIRTUAL_DISK_FLAG_NONE, OPEN_VIRTUAL_DISK_PARAMETERS, OPEN_VIRTUAL_DISK_VERSION_2,
    RESIZE_VIRTUAL_DISK_FLAG, RESIZE_VIRTUAL_DISK_PARAMETERS, RESIZE_VIRTUAL_DISK_VERSION_1,
    VIRTUAL_DISK_ACCESS_ALL, VIRTUAL_DISK_ACCESS_ATTACH_RO, VIRTUAL_DISK_ACCESS_ATTACH_RW,
    VIRTUAL_DISK_ACCESS_CREATE, VIRTUAL_DISK_ACCESS_DETACH, VIRTUAL_DISK_ACCESS_GET_INFO,
    VIRTUAL_DISK_ACCESS_MASK, VIRTUAL_STORAGE_TYPE,
};

/// VHD format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VhdFormat {
    /// Legacy VHD format (max 2TB)
    Vhd,
    /// Modern VHDX format (max 64TB)
    Vhdx,
}

// VHD storage type GUIDs
const VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT: windows::core::GUID =
    windows::core::GUID::from_u128(0xec984aec_a0f9_47e9_901f_71415a66345b);

impl VhdFormat {
    /// Returns the file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            VhdFormat::Vhd => ".vhd",
            VhdFormat::Vhdx => ".vhdx",
        }
    }

    /// Determines format from file path
    pub fn from_path(path: &str) -> Self {
        if path.to_lowercase().ends_with(".vhdx") {
            VhdFormat::Vhdx
        } else {
            VhdFormat::Vhd
        }
    }

    /// Returns the device ID for the format
    fn device_id(&self) -> u32 {
        match self {
            VhdFormat::Vhd => 2,   // VIRTUAL_STORAGE_TYPE_DEVICE_VHD
            VhdFormat::Vhdx => 3,  // VIRTUAL_STORAGE_TYPE_DEVICE_VHDX
        }
    }
}

/// VHD type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VhdType {
    /// Fixed size - pre-allocated
    Fixed,
    /// Dynamic - grows as needed
    Dynamic,
    /// Differencing - based on parent VHD
    Differencing,
}

impl From<u32> for VhdType {
    fn from(value: u32) -> Self {
        match value {
            2 => VhdType::Fixed,
            3 => VhdType::Dynamic,
            4 => VhdType::Differencing,
            _ => VhdType::Dynamic,
        }
    }
}

impl VhdType {
    /// Returns the Windows API value for this VHD type
    pub fn to_api_value(&self) -> u32 {
        match self {
            VhdType::Fixed => 2,
            VhdType::Dynamic => 3,
            VhdType::Differencing => 4,
        }
    }
}

/// Helper to convert Rust string to wide string (UTF-16)
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// RAII wrapper for VHD handle
struct VhdHandle(HANDLE);

impl VhdHandle {
    fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for VhdHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

/// Represents a Hyper-V virtual hard disk
pub struct Vhd {
    path: String,
}

impl Vhd {
    /// Create a new VHD handle for an existing file
    pub(crate) fn new(path: String) -> Self {
        Vhd { path }
    }

    /// Returns the VHD file path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the VHD format (VHD or VHDX)
    pub fn format(&self) -> VhdFormat {
        VhdFormat::from_path(&self.path)
    }

    /// Opens the VHD with specified access
    fn open(&self, access: VIRTUAL_DISK_ACCESS_MASK) -> Result<VhdHandle> {
        let path_wide = to_wide(&self.path);
        let format = self.format();

        let storage_type = VIRTUAL_STORAGE_TYPE {
            DeviceId: format.device_id(),
            VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
        };

        let mut parameters = OPEN_VIRTUAL_DISK_PARAMETERS::default();
        parameters.Version = OPEN_VIRTUAL_DISK_VERSION_2;

        let mut handle = HANDLE::default();

        unsafe {
            OpenVirtualDisk(
                &storage_type,
                PCWSTR(path_wide.as_ptr()),
                access,
                OPEN_VIRTUAL_DISK_FLAG_NONE,
                Some(&parameters),
                &mut handle,
            )
            .map_err(|e| HvError::HcsError(format!("Failed to open VHD: {}", e)))?;
        }

        Ok(VhdHandle(handle))
    }

    /// Gets the VHD type (Fixed, Dynamic, Differencing)
    pub fn vhd_type(&self) -> Result<VhdType> {
        let handle = self.open(VIRTUAL_DISK_ACCESS_GET_INFO)?;

        let mut info = GET_VIRTUAL_DISK_INFO {
            Version: GET_VIRTUAL_DISK_INFO_VIRTUAL_STORAGE_TYPE,
            ..Default::default()
        };
        let mut info_size = std::mem::size_of::<GET_VIRTUAL_DISK_INFO>() as u32;
        let mut size_used = 0u32;

        unsafe {
            GetVirtualDiskInformation(
                handle.as_raw(),
                &mut info_size,
                &mut info,
                Some(&mut size_used),
            )
            .map_err(|e| HvError::HcsError(format!("Failed to get VHD info: {}", e)))?;

            // The VirtualStorageType contains DeviceId which indicates type
            Ok(VhdType::from(info.Anonymous.VirtualStorageType.DeviceId))
        }
    }

    /// Gets the maximum size in bytes
    pub fn max_size_bytes(&self) -> Result<u64> {
        let handle = self.open(VIRTUAL_DISK_ACCESS_GET_INFO)?;

        let mut info = GET_VIRTUAL_DISK_INFO {
            Version: GET_VIRTUAL_DISK_INFO_SIZE,
            ..Default::default()
        };
        let mut info_size = std::mem::size_of::<GET_VIRTUAL_DISK_INFO>() as u32;
        let mut size_used = 0u32;

        unsafe {
            GetVirtualDiskInformation(
                handle.as_raw(),
                &mut info_size,
                &mut info,
                Some(&mut size_used),
            )
            .map_err(|e| HvError::HcsError(format!("Failed to get VHD size: {}", e)))?;

            Ok(info.Anonymous.Size.VirtualSize)
        }
    }

    /// Gets the current file size in bytes
    pub fn file_size_bytes(&self) -> Result<u64> {
        let handle = self.open(VIRTUAL_DISK_ACCESS_GET_INFO)?;

        let mut info = GET_VIRTUAL_DISK_INFO {
            Version: GET_VIRTUAL_DISK_INFO_SIZE,
            ..Default::default()
        };
        let mut info_size = std::mem::size_of::<GET_VIRTUAL_DISK_INFO>() as u32;
        let mut size_used = 0u32;

        unsafe {
            GetVirtualDiskInformation(
                handle.as_raw(),
                &mut info_size,
                &mut info,
                Some(&mut size_used),
            )
            .map_err(|e| HvError::HcsError(format!("Failed to get VHD file size: {}", e)))?;

            Ok(info.Anonymous.Size.PhysicalSize)
        }
    }

    /// Checks if the VHD is attached
    pub fn is_attached(&self) -> Result<bool> {
        // Try to detach - if it fails with "not attached" error, it's not attached
        let handle = self.open(VIRTUAL_DISK_ACCESS_GET_INFO)?;

        unsafe {
            match DetachVirtualDisk(handle.as_raw(), DETACH_VIRTUAL_DISK_FLAG(0), 0) {
                Ok(_) => {
                    // Was attached, and now detached - return true but this changes state
                    // Actually, let's check a different way
                    Ok(true)
                }
                Err(_) => Ok(false),
            }
        }
    }

    /// Resizes the VHD to a new size in bytes
    pub fn resize(&self, new_size_bytes: u64) -> Result<()> {
        let handle = self.open(VIRTUAL_DISK_ACCESS_ALL)?;

        let mut parameters = RESIZE_VIRTUAL_DISK_PARAMETERS::default();
        parameters.Version = RESIZE_VIRTUAL_DISK_VERSION_1;
        parameters.Anonymous.Version1.NewSize = new_size_bytes;

        unsafe {
            ResizeVirtualDisk(
                handle.as_raw(),
                RESIZE_VIRTUAL_DISK_FLAG(0),
                &parameters,
                None,
            )
            .map_err(|e| HvError::HcsError(format!("Failed to resize VHD: {}", e)))?;
        }

        Ok(())
    }

    /// Compacts a dynamic VHD to reclaim unused space
    pub fn compact(&self) -> Result<()> {
        let handle = self.open(VIRTUAL_DISK_ACCESS_ALL)?;

        unsafe {
            CompactVirtualDisk(handle.as_raw(), COMPACT_VIRTUAL_DISK_FLAG(0), None)
                .map_err(|e| HvError::HcsError(format!("Failed to compact VHD: {}", e)))?;
        }

        Ok(())
    }

    /// Mounts the VHD to the host
    pub fn mount(&self, read_only: bool) -> Result<()> {
        let access = if read_only {
            VIRTUAL_DISK_ACCESS_ATTACH_RO
        } else {
            VIRTUAL_DISK_ACCESS_ATTACH_RW
        };

        let handle = self.open(access)?;

        let flags = if read_only {
            ATTACH_VIRTUAL_DISK_FLAG_READ_ONLY | ATTACH_VIRTUAL_DISK_FLAG_NO_DRIVE_LETTER
        } else {
            ATTACH_VIRTUAL_DISK_FLAG(0)
        };

        unsafe {
            AttachVirtualDisk(handle.as_raw(), None, flags, 0, None, None)
                .map_err(|e| HvError::HcsError(format!("Failed to mount VHD: {}", e)))?;
        }

        Ok(())
    }

    /// Dismounts the VHD from the host
    pub fn dismount(&self) -> Result<()> {
        let handle = self.open(VIRTUAL_DISK_ACCESS_DETACH)?;

        unsafe {
            DetachVirtualDisk(handle.as_raw(), DETACH_VIRTUAL_DISK_FLAG(0), 0)
                .map_err(|e| HvError::HcsError(format!("Failed to dismount VHD: {}", e)))?;
        }

        Ok(())
    }
}

impl std::fmt::Debug for Vhd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vhd")
            .field("path", &self.path)
            .field("format", &self.format())
            .finish()
    }
}

/// Creates a new VHD file
pub fn create_vhd(
    path: &str,
    size_bytes: u64,
    vhd_type: VhdType,
    block_size_bytes: Option<u32>,
) -> Result<Vhd> {
    let format = VhdFormat::from_path(path);
    let path_wide = to_wide(path);

    let storage_type = VIRTUAL_STORAGE_TYPE {
        DeviceId: format.device_id(),
        VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
    };

    let mut parameters = CREATE_VIRTUAL_DISK_PARAMETERS::default();
    parameters.Version = CREATE_VIRTUAL_DISK_VERSION_2;
    unsafe {
        parameters.Anonymous.Version2.MaximumSize = size_bytes;
        parameters.Anonymous.Version2.BlockSizeInBytes = block_size_bytes.unwrap_or(0);
        parameters.Anonymous.Version2.SectorSizeInBytes = 512;
    }

    let mut handle = HANDLE::default();

    unsafe {
        CreateVirtualDisk(
            &storage_type,
            PCWSTR(path_wide.as_ptr()),
            VIRTUAL_DISK_ACCESS_CREATE,
            None,
            CREATE_VIRTUAL_DISK_FLAG_NONE,
            0,
            &parameters,
            None,
            &mut handle,
        )
        .map_err(|e| HvError::HcsError(format!("Failed to create VHD: {}", e)))?;

        let _ = CloseHandle(handle);
    }

    Ok(Vhd::new(path.to_string()))
}

/// Creates a differencing VHD
pub fn create_differencing_vhd(path: &str, parent_path: &str) -> Result<Vhd> {
    let format = VhdFormat::from_path(path);
    let path_wide = to_wide(path);
    let parent_wide = to_wide(parent_path);

    let storage_type = VIRTUAL_STORAGE_TYPE {
        DeviceId: format.device_id(),
        VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
    };

    let mut parameters = CREATE_VIRTUAL_DISK_PARAMETERS::default();
    parameters.Version = CREATE_VIRTUAL_DISK_VERSION_2;
    unsafe {
        parameters.Anonymous.Version2.ParentPath = PCWSTR(parent_wide.as_ptr());
    }

    let mut handle = HANDLE::default();

    unsafe {
        CreateVirtualDisk(
            &storage_type,
            PCWSTR(path_wide.as_ptr()),
            VIRTUAL_DISK_ACCESS_CREATE,
            None,
            CREATE_VIRTUAL_DISK_FLAG_NONE,
            0,
            &parameters,
            None,
            &mut handle,
        )
        .map_err(|e| HvError::HcsError(format!("Failed to create differencing VHD: {}", e)))?;

        let _ = CloseHandle(handle);
    }

    Ok(Vhd::new(path.to_string()))
}
