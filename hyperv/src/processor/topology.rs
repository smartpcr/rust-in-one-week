//! NUMA topology configuration for Hyper-V VMs.

use std::fmt;

/// NUMA node configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumaNode {
    /// Node ID (0-based).
    pub id: u32,
    /// Number of virtual processors on this node.
    pub processor_count: u32,
    /// Memory in MB assigned to this node.
    pub memory_mb: u64,
}

impl NumaNode {
    /// Create a new NUMA node.
    pub fn new(id: u32, processor_count: u32, memory_mb: u64) -> Self {
        Self {
            id,
            processor_count,
            memory_mb,
        }
    }
}

impl fmt::Display for NumaNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node {}: {} vCPUs, {} MB",
            self.id, self.processor_count, self.memory_mb
        )
    }
}

/// NUMA topology for a VM.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NumaTopology {
    /// NUMA nodes.
    pub nodes: Vec<NumaNode>,
    /// Maximum processors per NUMA node.
    pub max_processors_per_node: Option<u32>,
    /// Maximum NUMA nodes per socket.
    pub max_nodes_per_socket: Option<u32>,
    /// Whether NUMA spanning is enabled.
    pub numa_spanning_enabled: bool,
}

impl NumaTopology {
    /// Create an empty topology.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a simple topology with one node.
    pub fn single_node(processor_count: u32, memory_mb: u64) -> Self {
        Self {
            nodes: vec![NumaNode::new(0, processor_count, memory_mb)],
            max_processors_per_node: None,
            max_nodes_per_socket: None,
            numa_spanning_enabled: true,
        }
    }

    /// Create a symmetric topology with equal resources per node.
    pub fn symmetric(node_count: u32, processors_per_node: u32, memory_per_node_mb: u64) -> Self {
        let nodes = (0..node_count)
            .map(|id| NumaNode::new(id, processors_per_node, memory_per_node_mb))
            .collect();

        Self {
            nodes,
            max_processors_per_node: Some(processors_per_node),
            max_nodes_per_socket: None,
            numa_spanning_enabled: true,
        }
    }

    /// Add a NUMA node.
    pub fn add_node(&mut self, node: NumaNode) {
        self.nodes.push(node);
    }

    /// Get total processor count across all nodes.
    pub fn total_processors(&self) -> u32 {
        self.nodes.iter().map(|n| n.processor_count).sum()
    }

    /// Get total memory across all nodes.
    pub fn total_memory_mb(&self) -> u64 {
        self.nodes.iter().map(|n| n.memory_mb).sum()
    }

    /// Get number of NUMA nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if topology is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Set maximum processors per NUMA node.
    pub fn with_max_processors_per_node(mut self, max: u32) -> Self {
        self.max_processors_per_node = Some(max);
        self
    }

    /// Set maximum NUMA nodes per socket.
    pub fn with_max_nodes_per_socket(mut self, max: u32) -> Self {
        self.max_nodes_per_socket = Some(max);
        self
    }

    /// Enable or disable NUMA spanning.
    pub fn with_numa_spanning(mut self, enabled: bool) -> Self {
        self.numa_spanning_enabled = enabled;
        self
    }
}

impl fmt::Display for NumaTopology {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} NUMA nodes, {} total vCPUs, {} MB total memory",
            self.node_count(),
            self.total_processors(),
            self.total_memory_mb()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numa_node() {
        let node = NumaNode::new(0, 4, 8192);
        assert_eq!(node.id, 0);
        assert_eq!(node.processor_count, 4);
        assert_eq!(node.memory_mb, 8192);
    }

    #[test]
    fn test_numa_node_display() {
        let node = NumaNode::new(1, 8, 16384);
        let display = format!("{}", node);
        assert!(display.contains("Node 1"));
        assert!(display.contains("8 vCPUs"));
        assert!(display.contains("16384 MB"));
    }

    #[test]
    fn test_numa_topology_single_node() {
        let topo = NumaTopology::single_node(4, 8192);
        assert_eq!(topo.node_count(), 1);
        assert_eq!(topo.total_processors(), 4);
        assert_eq!(topo.total_memory_mb(), 8192);
    }

    #[test]
    fn test_numa_topology_symmetric() {
        let topo = NumaTopology::symmetric(2, 4, 8192);
        assert_eq!(topo.node_count(), 2);
        assert_eq!(topo.total_processors(), 8);
        assert_eq!(topo.total_memory_mb(), 16384);
        assert_eq!(topo.max_processors_per_node, Some(4));
    }

    #[test]
    fn test_numa_topology_add_node() {
        let mut topo = NumaTopology::new();
        assert!(topo.is_empty());

        topo.add_node(NumaNode::new(0, 4, 4096));
        topo.add_node(NumaNode::new(1, 2, 2048));

        assert_eq!(topo.node_count(), 2);
        assert_eq!(topo.total_processors(), 6);
        assert_eq!(topo.total_memory_mb(), 6144);
    }

    #[test]
    fn test_numa_topology_builder_methods() {
        let topo = NumaTopology::single_node(4, 8192)
            .with_max_processors_per_node(8)
            .with_max_nodes_per_socket(2)
            .with_numa_spanning(false);

        assert_eq!(topo.max_processors_per_node, Some(8));
        assert_eq!(topo.max_nodes_per_socket, Some(2));
        assert!(!topo.numa_spanning_enabled);
    }

    #[test]
    fn test_numa_topology_display() {
        let topo = NumaTopology::symmetric(2, 4, 8192);
        let display = format!("{}", topo);
        assert!(display.contains("2 NUMA nodes"));
        assert!(display.contains("8 total vCPUs"));
    }
}
