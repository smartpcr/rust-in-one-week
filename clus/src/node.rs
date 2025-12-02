//! Cluster node operations

use crate::cluster::Cluster;
use crate::error::{ClusError, Result};
use crate::utils::to_wide;
use windows::core::{Error as WinError, PCWSTR};
use windows::Win32::Networking::Clustering::{
    CloseClusterNode, ClusterNodeDown, ClusterNodeJoining, ClusterNodePaused, ClusterNodeUp,
    GetClusterNodeState, OpenClusterNode, PauseClusterNode, ResumeClusterNode,
    CLUSTER_NODE_STATE, HNODE,
};

/// Represents a node in a Windows Failover Cluster
pub struct Node {
    pub(crate) handle: HNODE,
    name: String,
}

/// Node state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Up,
    Down,
    Paused,
    Joining,
    Unknown(u32),
}

impl From<CLUSTER_NODE_STATE> for NodeState {
    fn from(state: CLUSTER_NODE_STATE) -> Self {
        match state {
            s if s == ClusterNodeUp => NodeState::Up,
            s if s == ClusterNodeDown => NodeState::Down,
            s if s == ClusterNodePaused => NodeState::Paused,
            s if s == ClusterNodeJoining => NodeState::Joining,
            s => NodeState::Unknown(s.0 as u32),
        }
    }
}

impl Node {
    /// Opens a node by name within a cluster
    pub fn open(cluster: &Cluster, node_name: &str) -> Result<Self> {
        let wide_name = to_wide(node_name);

        let handle = unsafe { OpenClusterNode(cluster.handle(), PCWSTR(wide_name.as_ptr())) };

        if handle.0 == 0 {
            return Err(ClusError::NotFound(node_name.to_string()));
        }

        Ok(Node {
            handle,
            name: node_name.to_string(),
        })
    }

    /// Returns the node name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the raw node handle
    pub fn handle(&self) -> HNODE {
        self.handle
    }

    /// Gets the current state of the node
    pub fn state(&self) -> NodeState {
        let state = unsafe { GetClusterNodeState(self.handle) };
        NodeState::from(state)
    }

    /// Pauses the node (prevents resources from failing over to this node)
    pub fn pause(&self) -> Result<()> {
        let result = unsafe { PauseClusterNode(self.handle) };
        if result != 0 {
            return Err(ClusError::WindowsError(WinError::from_thread()));
        }
        Ok(())
    }

    /// Resumes a paused node
    pub fn resume(&self) -> Result<()> {
        let result = unsafe { ResumeClusterNode(self.handle) };
        if result != 0 {
            return Err(ClusError::WindowsError(WinError::from_thread()));
        }
        Ok(())
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if !self.handle.0 == 0 {
            unsafe {
                let _ = CloseClusterNode(self.handle);
            }
        }
    }
}
