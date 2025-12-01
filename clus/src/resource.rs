//! Cluster resource operations

use std::ptr;

use crate::cluster::Cluster;
use crate::error::{ClusError, Result};
use crate::utils::{from_wide, to_wide};
use windows::core::{Error as WinError, PCWSTR};
use windows::Win32::Networking::Clustering::{
    CloseClusterResource, ClusterResourceFailed, ClusterResourceOffline,
    ClusterResourceOfflinePending, ClusterResourceOnline, ClusterResourceOnlinePending,
    ClusterResourceStateUnknown, GetClusterResourceState, OfflineClusterResource,
    OnlineClusterResource, OpenClusterResourceW, CLUSTER_RESOURCE_STATE, HRESOURCE,
};

/// Represents a resource in a Windows Failover Cluster
pub struct Resource {
    handle: HRESOURCE,
    name: String,
}

/// Resource state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Online,
    Offline,
    Failed,
    OnlinePending,
    OfflinePending,
    Unknown(u32),
}

impl From<CLUSTER_RESOURCE_STATE> for ResourceState {
    fn from(state: CLUSTER_RESOURCE_STATE) -> Self {
        match state {
            s if s == ClusterResourceOnline => ResourceState::Online,
            s if s == ClusterResourceOffline => ResourceState::Offline,
            s if s == ClusterResourceFailed => ResourceState::Failed,
            s if s == ClusterResourceOnlinePending => ResourceState::OnlinePending,
            s if s == ClusterResourceOfflinePending => ResourceState::OfflinePending,
            s => ResourceState::Unknown(s.0 as u32),
        }
    }
}

impl Resource {
    /// Opens a resource by name within a cluster
    pub fn open(cluster: &Cluster, resource_name: &str) -> Result<Self> {
        let wide_name = to_wide(resource_name);

        let handle =
            unsafe { OpenClusterResourceW(cluster.handle(), PCWSTR(wide_name.as_ptr())) };

        if handle.is_invalid() {
            return Err(ClusError::NotFound(resource_name.to_string()));
        }

        Ok(Resource {
            handle,
            name: resource_name.to_string(),
        })
    }

    /// Returns the resource name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the raw resource handle
    pub fn handle(&self) -> HRESOURCE {
        self.handle
    }

    /// Gets the current state of the resource and optionally the owner node name
    pub fn state(&self) -> Result<(ResourceState, Option<String>)> {
        let mut size: u32 = 0;

        // Get required size for node name
        let state = unsafe {
            GetClusterResourceState(self.handle, PCWSTR(ptr::null_mut()), &mut size, None, None)
        };

        if state == ClusterResourceStateUnknown && size == 0 {
            return Ok((ResourceState::Unknown(0), None));
        }

        size += 1;
        let mut buffer: Vec<u16> = vec![0; size as usize];

        let state = unsafe {
            GetClusterResourceState(
                self.handle,
                PCWSTR(buffer.as_mut_ptr()),
                &mut size,
                None,
                None,
            )
        };

        let owner_node = from_wide(buffer.as_ptr());
        let owner = if owner_node.is_empty() {
            None
        } else {
            Some(owner_node)
        };

        Ok((state.into(), owner))
    }

    /// Brings the resource online
    pub fn online(&self) -> Result<()> {
        let result = unsafe { OnlineClusterResource(self.handle) };
        // ERROR_IO_PENDING (997) is acceptable - means operation is in progress
        if result != 0 && result != 997 {
            return Err(ClusError::WindowsError(WinError::from_win32()));
        }
        Ok(())
    }

    /// Takes the resource offline
    pub fn offline(&self) -> Result<()> {
        let result = unsafe { OfflineClusterResource(self.handle) };
        // ERROR_IO_PENDING (997) is acceptable - means operation is in progress
        if result != 0 && result != 997 {
            return Err(ClusError::WindowsError(WinError::from_win32()));
        }
        Ok(())
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                let _ = CloseClusterResource(self.handle);
            }
        }
    }
}
