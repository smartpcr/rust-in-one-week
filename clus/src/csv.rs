//! Cluster Shared Volume (CSV) operations
//!
//! Provides functionality for managing Cluster Shared Volumes including:
//! - Path resolution and volume discovery
//! - State querying and monitoring
//! - Maintenance mode management
//! - Volume information retrieval

use std::ptr;

use crate::cluster::Cluster;
use crate::error::{ClusError, Result};
use crate::resource::Resource;
use crate::utils::{from_wide, to_wide};
use windows::core::PCWSTR;
use windows::Win32::Networking::Clustering::{
    ClusterResourceControl, ClusterSharedVolumeSetSnapshotState,
    CLCTL_STORAGE_GET_SHARED_VOLUME_INFO, CLCTL_STORAGE_IS_SHARED_VOLUME,
    CLCTL_SET_CSV_MAINTENANCE_MODE, CLUSTER_SHARED_VOLUME_SNAPSHOT_STATE,
    ClusterGetVolumeNameForVolumeMountPoint, ClusterGetVolumePathName,
    ClusterIsPathOnSharedVolume, CLUS_CSV_MAINTENANCE_MODE_INFO,
};

/// CSV volume state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvState {
    /// Volume is online and accessible
    Online,
    /// Volume is in paused/maintenance state
    Paused,
    /// Volume is draining (transitioning)
    Draining,
    /// Volume is in redirected access mode
    Redirected,
    /// State is unknown
    Unknown(i32),
}

/// CSV fault state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvFaultState {
    /// No fault
    NoFault,
    /// Volume has faulted
    Faulted,
    /// Unknown fault state
    Unknown(i32),
}

/// CSV backup state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvBackupState {
    /// No backup in progress
    None,
    /// Backup is in progress
    InProgress,
    /// Unknown backup state
    Unknown(i32),
}

/// Reason for redirected I/O on a CSV
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedirectedIoReason {
    /// No redirection
    None,
    /// User initiated redirection
    UserRequest,
    /// File system incompatibility
    FileSystemIncompatibility,
    /// Volume is in maintenance mode
    IncompatibleFileSystemFilter,
    /// Data verification needed
    DataVerification,
    /// Unknown reason
    Unknown(u64),
}

impl From<u64> for RedirectedIoReason {
    fn from(value: u64) -> Self {
        match value {
            0 => RedirectedIoReason::None,
            1 => RedirectedIoReason::UserRequest,
            2 => RedirectedIoReason::FileSystemIncompatibility,
            4 => RedirectedIoReason::IncompatibleFileSystemFilter,
            8 => RedirectedIoReason::DataVerification,
            _ => RedirectedIoReason::Unknown(value),
        }
    }
}

/// Information about a Cluster Shared Volume
#[derive(Debug, Clone)]
pub struct CsvInfo {
    /// Volume name (e.g., "\\?\Volume{guid}\")
    pub volume_name: String,
    /// Friendly name (e.g., "Volume1")
    pub friendly_name: String,
    /// Mount point path (e.g., "C:\ClusterStorage\Volume1")
    pub mount_point: String,
    /// Current state
    pub state: CsvState,
    /// Fault state
    pub fault_state: CsvFaultState,
    /// Backup state
    pub backup_state: CsvBackupState,
    /// Owner node name
    pub owner_node: Option<String>,
    /// Reason for redirected I/O (if any)
    pub redirected_io_reason: RedirectedIoReason,
    /// Whether the volume is in maintenance mode
    pub in_maintenance: bool,
}

/// Cluster Shared Volume operations
pub struct Csv;

impl Csv {
    /// Check if a path resides on a Cluster Shared Volume
    ///
    /// # Arguments
    /// * `path` - The file or directory path to check
    ///
    /// # Returns
    /// `true` if the path is on a CSV, `false` otherwise
    pub fn is_path_on_csv(path: &str) -> bool {
        let wide_path = to_wide(path);
        let result = unsafe { ClusterIsPathOnSharedVolume(PCWSTR(wide_path.as_ptr())) };
        result != 0
    }

    /// Get the CSV volume path for a given file path
    ///
    /// Returns the volume mount point (e.g., "C:\ClusterStorage\Volume1")
    /// for a file residing on a CSV.
    ///
    /// # Arguments
    /// * `path` - A file path on a CSV
    ///
    /// # Returns
    /// The volume path or an error if the path is not on a CSV
    pub fn get_volume_path(path: &str) -> Result<String> {
        let wide_path = to_wide(path);
        let mut buffer: [u16; 260] = [0; 260];

        let result = unsafe {
            ClusterGetVolumePathName(
                PCWSTR(wide_path.as_ptr()),
                PCWSTR(buffer.as_mut_ptr()),
                buffer.len() as u32,
            )
        };

        if result.is_err() {
            return Err(ClusError::OperationFailed(
                "Failed to get CSV volume path".to_string(),
            ));
        }

        Ok(from_wide(buffer.as_ptr()))
    }

    /// Get the volume GUID name for a CSV mount point
    ///
    /// Returns the volume GUID path (e.g., "\\?\Volume{guid}\")
    /// for a CSV mount point.
    ///
    /// # Arguments
    /// * `mount_point` - A CSV mount point (e.g., "C:\ClusterStorage\Volume1\")
    ///
    /// # Returns
    /// The volume GUID name or an error
    pub fn get_volume_name(mount_point: &str) -> Result<String> {
        let wide_mount = to_wide(mount_point);
        let mut buffer: [u16; 50] = [0; 50];

        let result = unsafe {
            ClusterGetVolumeNameForVolumeMountPoint(
                PCWSTR(wide_mount.as_ptr()),
                PCWSTR(buffer.as_mut_ptr()),
                buffer.len() as u32,
            )
        };

        if result.is_err() {
            return Err(ClusError::OperationFailed(
                "Failed to get CSV volume name".to_string(),
            ));
        }

        Ok(from_wide(buffer.as_ptr()))
    }

    /// Check if a cluster resource is a Cluster Shared Volume
    ///
    /// # Arguments
    /// * `resource` - The cluster resource to check
    ///
    /// # Returns
    /// `true` if the resource is a CSV, `false` otherwise
    pub fn is_csv_resource(resource: &Resource) -> Result<bool> {
        let mut bytes_returned: u32 = 0;

        let result = unsafe {
            ClusterResourceControl(
                resource.handle(),
                None,
                CLCTL_STORAGE_IS_SHARED_VOLUME.0 as u32,
                None,
                0,
                None,
                0,
                Some(&mut bytes_returned),
            )
        };

        // ERROR_SUCCESS (0) means it is a CSV
        Ok(result == 0)
    }

    /// Set maintenance mode for a CSV volume
    ///
    /// When in maintenance mode, I/O to the CSV is redirected through
    /// the coordinator node.
    ///
    /// # Arguments
    /// * `resource` - The CSV resource
    /// * `enable` - `true` to enable maintenance mode, `false` to disable
    pub fn set_maintenance_mode(resource: &Resource, enable: bool) -> Result<()> {
        let volume_name = resource.name();
        let wide_name = to_wide(volume_name);

        // Ensure the wide_name buffer is exactly 260 u16s for the struct
        let mut volume_name_array: [u16; 260] = [0; 260];
        for (i, &c) in wide_name.iter().take(259).enumerate() {
            volume_name_array[i] = c;
        }

        let info = CLUS_CSV_MAINTENANCE_MODE_INFO {
            InMaintenance: enable.into(),
            VolumeName: volume_name_array,
        };

        let info_ptr = &info as *const _ as *const std::ffi::c_void;
        let info_size = std::mem::size_of::<CLUS_CSV_MAINTENANCE_MODE_INFO>() as u32;

        let result = unsafe {
            ClusterResourceControl(
                resource.handle(),
                None,
                CLCTL_SET_CSV_MAINTENANCE_MODE.0 as u32,
                Some(info_ptr),
                info_size,
                None,
                0,
                None,
            )
        };

        if result != 0 {
            return Err(ClusError::OperationFailed(format!(
                "Failed to set CSV maintenance mode: error {}",
                result
            )));
        }

        Ok(())
    }

    /// Get CSV volume information for a resource
    ///
    /// # Arguments
    /// * `resource` - The CSV resource
    ///
    /// # Returns
    /// CSV volume information or an error
    pub fn get_volume_info(resource: &Resource) -> Result<CsvInfo> {
        let mut buffer: [u8; 2048] = [0; 2048];
        let mut bytes_returned: u32 = 0;

        let result = unsafe {
            ClusterResourceControl(
                resource.handle(),
                None,
                CLCTL_STORAGE_GET_SHARED_VOLUME_INFO.0 as u32,
                None,
                0,
                Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
                buffer.len() as u32,
                Some(&mut bytes_returned),
            )
        };

        if result != 0 {
            return Err(ClusError::OperationFailed(format!(
                "Failed to get CSV volume info: error {}",
                result
            )));
        }

        // Parse the returned data
        // The structure returned is CLUS_CSV_VOLUME_INFO
        if bytes_returned == 0 {
            return Err(ClusError::OperationFailed(
                "No CSV volume info returned".to_string(),
            ));
        }

        // For now, return basic info from the resource
        // Full parsing of CLUS_CSV_VOLUME_INFO would require more complex handling
        let (state, owner) = resource.state()?;

        Ok(CsvInfo {
            volume_name: String::new(),
            friendly_name: resource.name().to_string(),
            mount_point: format!("C:\\ClusterStorage\\{}", resource.name()),
            state: match state {
                crate::resource::ResourceState::Online => CsvState::Online,
                crate::resource::ResourceState::Offline => CsvState::Paused,
                crate::resource::ResourceState::OnlinePending => CsvState::Draining,
                crate::resource::ResourceState::OfflinePending => CsvState::Draining,
                _ => CsvState::Unknown(0),
            },
            fault_state: CsvFaultState::NoFault,
            backup_state: CsvBackupState::None,
            owner_node: owner,
            redirected_io_reason: RedirectedIoReason::None,
            in_maintenance: false,
        })
    }

    /// Set CSV snapshot state
    ///
    /// Used for VSS (Volume Shadow Copy Service) operations on CSVs.
    ///
    /// # Arguments
    /// * `guid` - The snapshot GUID
    /// * `volume_name` - The CSV volume name
    /// * `state` - The snapshot state to set
    pub fn set_snapshot_state(
        guid: &str,
        volume_name: &str,
        state: CsvSnapshotState,
    ) -> Result<()> {
        let wide_volume = to_wide(volume_name);

        // Parse GUID string to GUID struct
        let guid_bytes = parse_guid(guid)?;

        let win_state = match state {
            CsvSnapshotState::InitializedAndPersisted => {
                CLUSTER_SHARED_VOLUME_SNAPSHOT_STATE(1)
            }
            CsvSnapshotState::Deleted => CLUSTER_SHARED_VOLUME_SNAPSHOT_STATE(2),
        };

        let result = unsafe {
            ClusterSharedVolumeSetSnapshotState(
                guid_bytes,
                PCWSTR(wide_volume.as_ptr()),
                win_state,
            )
        };

        if result != 0 {
            return Err(ClusError::OperationFailed(format!(
                "Failed to set CSV snapshot state: error {}",
                result
            )));
        }

        Ok(())
    }
}

/// CSV snapshot state for VSS operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvSnapshotState {
    /// Snapshot is initialized and persisted
    InitializedAndPersisted,
    /// Snapshot is deleted
    Deleted,
}

/// Parse a GUID string into a windows GUID
fn parse_guid(guid_str: &str) -> Result<windows::core::GUID> {
    // Remove braces if present
    let s = guid_str.trim_start_matches('{').trim_end_matches('}');

    // Expected format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 {
        return Err(ClusError::OperationFailed(format!(
            "Invalid GUID format: {}",
            guid_str
        )));
    }

    let data1 = u32::from_str_radix(parts[0], 16)
        .map_err(|_| ClusError::OperationFailed("Invalid GUID".to_string()))?;
    let data2 = u16::from_str_radix(parts[1], 16)
        .map_err(|_| ClusError::OperationFailed("Invalid GUID".to_string()))?;
    let data3 = u16::from_str_radix(parts[2], 16)
        .map_err(|_| ClusError::OperationFailed("Invalid GUID".to_string()))?;

    let data4_high = u16::from_str_radix(parts[3], 16)
        .map_err(|_| ClusError::OperationFailed("Invalid GUID".to_string()))?;
    let data4_low = u64::from_str_radix(parts[4], 16)
        .map_err(|_| ClusError::OperationFailed("Invalid GUID".to_string()))?;

    let data4: [u8; 8] = [
        (data4_high >> 8) as u8,
        data4_high as u8,
        (data4_low >> 40) as u8,
        (data4_low >> 32) as u8,
        (data4_low >> 24) as u8,
        (data4_low >> 16) as u8,
        (data4_low >> 8) as u8,
        data4_low as u8,
    ];

    Ok(windows::core::GUID {
        data1,
        data2,
        data3,
        data4,
    })
}

// =============================================================================
// Cluster extension methods for CSV
// =============================================================================

impl Cluster {
    /// Enumerate all Cluster Shared Volumes in the cluster
    ///
    /// Returns a list of resources that are CSVs
    pub fn csv_volumes(&self) -> Result<Vec<Resource>> {
        let resources = self.resources()?;
        let mut csvs = Vec::new();

        for resource in resources {
            if Csv::is_csv_resource(&resource).unwrap_or(false) {
                csvs.push(resource);
            }
        }

        Ok(csvs)
    }

    /// Get CSV information for all shared volumes
    pub fn csv_info(&self) -> Result<Vec<CsvInfo>> {
        let csvs = self.csv_volumes()?;
        let mut info_list = Vec::new();

        for csv in &csvs {
            if let Ok(info) = Csv::get_volume_info(csv) {
                info_list.push(info);
            }
        }

        Ok(info_list)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_guid() {
        let guid = parse_guid("12345678-1234-1234-1234-123456789abc").unwrap();
        assert_eq!(guid.data1, 0x12345678);
        assert_eq!(guid.data2, 0x1234);
        assert_eq!(guid.data3, 0x1234);
    }

    #[test]
    fn test_parse_guid_with_braces() {
        let guid = parse_guid("{12345678-1234-1234-1234-123456789abc}").unwrap();
        assert_eq!(guid.data1, 0x12345678);
    }

    #[test]
    #[ignore] // Run manually on a cluster node
    fn test_is_path_on_csv() {
        // This path would only exist on a cluster with CSVs
        let is_csv = Csv::is_path_on_csv("C:\\ClusterStorage\\Volume1\\test.txt");
        println!("Path is on CSV: {}", is_csv);
    }
}
