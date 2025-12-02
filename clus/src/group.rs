//! Cluster group (role) operations

use std::ptr;

use crate::cluster::Cluster;
use crate::error::{ClusError, Result};
use crate::node::Node;
use crate::utils::{from_wide, to_wide};
use windows::core::{Error as WinError, PCWSTR, PWSTR};
use windows::Win32::Networking::Clustering::{
    CloseClusterGroup, ClusterGroupFailed, ClusterGroupOffline, ClusterGroupOnline,
    ClusterGroupPartialOnline, ClusterGroupPending, ClusterGroupStateUnknown,
    GetClusterGroupState, MoveClusterGroup, OfflineClusterGroup, OnlineClusterGroup,
    OpenClusterGroup, CLUSTER_GROUP_STATE, HGROUP, HNODE,
};

/// Represents a group (role) in a Windows Failover Cluster
pub struct Group {
    handle: HGROUP,
    name: String,
}

/// Group state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupState {
    Online,
    Offline,
    Failed,
    PartialOnline,
    Pending,
    Unknown(u32),
}

impl From<CLUSTER_GROUP_STATE> for GroupState {
    fn from(state: CLUSTER_GROUP_STATE) -> Self {
        match state {
            s if s == ClusterGroupOnline => GroupState::Online,
            s if s == ClusterGroupOffline => GroupState::Offline,
            s if s == ClusterGroupFailed => GroupState::Failed,
            s if s == ClusterGroupPartialOnline => GroupState::PartialOnline,
            s if s == ClusterGroupPending => GroupState::Pending,
            s => GroupState::Unknown(s.0 as u32),
        }
    }
}

impl Group {
    /// Opens a group by name within a cluster
    pub fn open(cluster: &Cluster, group_name: &str) -> Result<Self> {
        let wide_name = to_wide(group_name);

        let handle = unsafe { OpenClusterGroup(cluster.handle(), PCWSTR(wide_name.as_ptr())) };

        if handle.0.is_null() {
            return Err(ClusError::NotFound(group_name.to_string()));
        }

        Ok(Group {
            handle,
            name: group_name.to_string(),
        })
    }

    /// Returns the group name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the raw group handle
    pub fn handle(&self) -> HGROUP {
        self.handle
    }

    /// Gets the current state of the group and optionally the owner node name
    pub fn state(&self) -> Result<(GroupState, Option<String>)> {
        let mut size: u32 = 0;

        let state =
            unsafe { GetClusterGroupState(self.handle, None, Some(&mut size)) };

        if state == ClusterGroupStateUnknown && size == 0 {
            return Ok((GroupState::Unknown(0), None));
        }

        size += 1;
        let mut buffer: Vec<u16> = vec![0; size as usize];

        let state =
            unsafe { GetClusterGroupState(self.handle, Some(PWSTR(buffer.as_mut_ptr())), Some(&mut size)) };

        let owner_node = from_wide(buffer.as_ptr());
        let owner = if owner_node.is_empty() {
            None
        } else {
            Some(owner_node)
        };

        Ok((state.into(), owner))
    }

    /// Bring the group online on the current or best possible node.
    pub fn online(&self) -> Result<()> {
        let result = unsafe { OnlineClusterGroup(self.handle, None) };
        // ERROR_IO_PENDING (997) is acceptable - means operation is in progress
        if result != 0 && result != 997 {
            return Err(ClusError::WindowsError(WinError::from_thread()));
        }
        Ok(())
    }

    /// Take the group offline.
    pub fn offline(&self) -> Result<()> {
        let result = unsafe { OfflineClusterGroup(self.handle) };
        // ERROR_IO_PENDING (997) is acceptable - means operation is in progress
        if result != 0 && result != 997 {
            return Err(ClusError::WindowsError(WinError::from_thread()));
        }
        Ok(())
    }

    /// Move the group to a specific node.
    pub fn move_to(&self, node: &Node) -> Result<()> {
        let result = unsafe { MoveClusterGroup(self.handle, Some(node.handle)) };
        // ERROR_IO_PENDING (997) is acceptable - means operation is in progress
        if result != 0 && result != 997 {
            return Err(ClusError::WindowsError(WinError::from_thread()));
        }
        Ok(())
    }
}

impl Drop for Group {
    fn drop(&mut self) {
        if !self.handle.0.is_null() {
            unsafe {
                let _ = CloseClusterGroup(self.handle);
            }
        }
    }
}
