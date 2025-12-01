//! Cluster handle and operations

use std::ptr;

use crate::error::{ClusError, Result};
use crate::group::Group;
use crate::node::Node;
use crate::resource::Resource;
use crate::utils::{from_wide, to_wide};
use windows::core::{Error as WinError, PCWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, WIN32_ERROR};
use windows::Win32::Networking::Clustering::{
    CloseCluster, ClusterCloseEnum, ClusterEnum, ClusterOpenEnum, GetClusterInformation,
    OpenClusterW, CLUSTER_ENUM_GROUP, CLUSTER_ENUM_NODE, CLUSTER_ENUM_RESOURCE, HCLUSENUM,
    HCLUSTER,
};

/// Represents a connection to a Windows Failover Cluster
pub struct Cluster {
    handle: HCLUSTER,
}

/// RAII guard for cluster enum handles
struct ClusterEnumGuard(HCLUSENUM);

impl Drop for ClusterEnumGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = ClusterCloseEnum(self.0);
        }
    }
}

impl Cluster {
    /// Opens a connection to a cluster by name.
    ///
    /// # Arguments
    /// * `cluster_name` - The name of the cluster to connect to. Pass `None` to connect to the local cluster.
    ///
    /// # Returns
    /// A `Result` containing the `Cluster` instance or a `ClusError`
    pub fn open(cluster_name: Option<&str>) -> Result<Self> {
        let handle = unsafe {
            match cluster_name {
                Some(name) => {
                    let wide = to_wide(name);
                    OpenClusterW(PCWSTR(wide.as_ptr()))
                }
                None => OpenClusterW(PCWSTR(ptr::null())),
            }
        };

        if handle.is_invalid() {
            return Err(ClusError::WindowsError(WinError::from_win32()));
        }

        Ok(Cluster { handle })
    }

    /// Get the cluster name dynamically from the cluster.
    pub fn name(&self) -> Result<String> {
        let mut size: u32 = 0;

        // First call to get required size
        unsafe {
            let _ = GetClusterInformation(self.handle, PCWSTR(ptr::null_mut()), &mut size, None);
        }

        size += 1; // Include null terminator
        let mut buffer: Vec<u16> = vec![0; size as usize];

        let result = unsafe {
            GetClusterInformation(self.handle, PCWSTR(buffer.as_mut_ptr()), &mut size, None)
        };

        if result != 0 {
            return Err(ClusError::WindowsError(WinError::from_win32()));
        }

        Ok(from_wide(buffer.as_ptr()))
    }

    /// Returns the raw cluster handle
    pub fn handle(&self) -> HCLUSTER {
        self.handle
    }

    /// Enumerate all nodes in the cluster.
    pub fn nodes(&self) -> Result<Vec<Node>> {
        let mut nodes = Vec::new();
        let enum_handle = unsafe { ClusterOpenEnum(self.handle, CLUSTER_ENUM_NODE) };

        if enum_handle.is_invalid() {
            return Err(ClusError::WindowsError(WinError::from_win32()));
        }

        let _guard = ClusterEnumGuard(enum_handle);

        let mut index: u32 = 0;
        loop {
            let mut obj_type: u32 = 0;
            let mut size: u32 = 0;

            // Get required size
            let result = unsafe {
                ClusterEnum(
                    enum_handle,
                    index,
                    &mut obj_type,
                    PCWSTR(ptr::null_mut()),
                    &mut size,
                )
            };

            if WIN32_ERROR(result) == ERROR_NO_MORE_ITEMS {
                break;
            }

            size += 1;
            let mut buffer: Vec<u16> = vec![0; size as usize];

            let result = unsafe {
                ClusterEnum(
                    enum_handle,
                    index,
                    &mut obj_type,
                    PCWSTR(buffer.as_mut_ptr()),
                    &mut size,
                )
            };

            if result != 0 {
                return Err(ClusError::WindowsError(WinError::from_win32()));
            }

            let node_name = from_wide(buffer.as_ptr());
            if let Ok(node) = self.open_node(&node_name) {
                nodes.push(node);
            }

            index += 1;
        }

        Ok(nodes)
    }

    /// Opens a node in this cluster by name
    pub fn open_node(&self, node_name: &str) -> Result<Node> {
        Node::open(self, node_name)
    }

    /// Enumerate all resources in the cluster.
    pub fn resources(&self) -> Result<Vec<Resource>> {
        let mut resources = Vec::new();
        let enum_handle = unsafe { ClusterOpenEnum(self.handle, CLUSTER_ENUM_RESOURCE) };

        if enum_handle.is_invalid() {
            return Err(ClusError::WindowsError(WinError::from_win32()));
        }

        let _guard = ClusterEnumGuard(enum_handle);

        let mut index: u32 = 0;
        loop {
            let mut obj_type: u32 = 0;
            let mut size: u32 = 0;

            let result = unsafe {
                ClusterEnum(
                    enum_handle,
                    index,
                    &mut obj_type,
                    PCWSTR(ptr::null_mut()),
                    &mut size,
                )
            };

            if WIN32_ERROR(result) == ERROR_NO_MORE_ITEMS {
                break;
            }

            size += 1;
            let mut buffer: Vec<u16> = vec![0; size as usize];

            let result = unsafe {
                ClusterEnum(
                    enum_handle,
                    index,
                    &mut obj_type,
                    PCWSTR(buffer.as_mut_ptr()),
                    &mut size,
                )
            };

            if result != 0 {
                return Err(ClusError::WindowsError(WinError::from_win32()));
            }

            let resource_name = from_wide(buffer.as_ptr());
            if let Ok(resource) = self.open_resource(&resource_name) {
                resources.push(resource);
            }

            index += 1;
        }

        Ok(resources)
    }

    /// Opens a resource in this cluster by name
    pub fn open_resource(&self, resource_name: &str) -> Result<Resource> {
        Resource::open(self, resource_name)
    }

    /// Enumerate all groups in the cluster.
    pub fn groups(&self) -> Result<Vec<Group>> {
        let mut groups = Vec::new();
        let enum_handle = unsafe { ClusterOpenEnum(self.handle, CLUSTER_ENUM_GROUP) };

        if enum_handle.is_invalid() {
            return Err(ClusError::WindowsError(WinError::from_win32()));
        }

        let _guard = ClusterEnumGuard(enum_handle);

        let mut index: u32 = 0;
        loop {
            let mut obj_type: u32 = 0;
            let mut size: u32 = 0;

            let result = unsafe {
                ClusterEnum(
                    enum_handle,
                    index,
                    &mut obj_type,
                    PCWSTR(ptr::null_mut()),
                    &mut size,
                )
            };

            if WIN32_ERROR(result) == ERROR_NO_MORE_ITEMS {
                break;
            }

            size += 1;
            let mut buffer: Vec<u16> = vec![0; size as usize];

            let result = unsafe {
                ClusterEnum(
                    enum_handle,
                    index,
                    &mut obj_type,
                    PCWSTR(buffer.as_mut_ptr()),
                    &mut size,
                )
            };

            if result != 0 {
                return Err(ClusError::WindowsError(WinError::from_win32()));
            }

            let group_name = from_wide(buffer.as_ptr());
            if let Ok(group) = self.open_group(&group_name) {
                groups.push(group);
            }

            index += 1;
        }

        Ok(groups)
    }

    /// Opens a group in this cluster by name
    pub fn open_group(&self, group_name: &str) -> Result<Group> {
        Group::open(self, group_name)
    }
}

impl Drop for Cluster {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                let _ = CloseCluster(self.handle);
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Run manually on a cluster node
    fn test_cluster_connection() {
        let cluster = Cluster::open(None).expect("Failed to open cluster");
        let name = cluster.name().expect("Failed to get cluster name");
        println!("Connected to cluster: {}", name);

        let nodes = cluster.nodes().expect("Failed to enumerate nodes");
        for node in &nodes {
            println!("Node: {} - {:?}", node.name(), node.state());
        }

        let resources = cluster.resources().expect("Failed to enumerate resources");
        for resource in &resources {
            let (state, owner) = resource.state().expect("Failed to get resource state");
            println!("Resource: {} - {:?} on {:?}", resource.name(), state, owner);
        }

        let groups = cluster.groups().expect("Failed to enumerate groups");
        for group in &groups {
            let (state, owner) = group.state().expect("Failed to get group state");
            println!("Group: {} - {:?} on {:?}", group.name(), state, owner);
        }
    }
}
