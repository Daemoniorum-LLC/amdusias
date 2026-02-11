//! Main audio graph implementation.

use crate::{
    connection::Connection,
    error::{Error, Result},
    node::{AudioNode, BoxedNode, NodeId, NodeInfo},
    processor::GraphProcessor,
};
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};

/// The main audio graph structure.
pub struct AudioGraph {
    /// All nodes in the graph.
    nodes: SlotMap<slotmap::DefaultKey, NodeEntry>,
    /// All connections.
    connections: Vec<Connection>,
    /// Sample rate.
    sample_rate: f32,
    /// Buffer size.
    buffer_size: usize,
    /// Whether the graph needs recompilation.
    dirty: bool,
    /// Compiled processing order.
    processing_order: Vec<NodeId>,
    /// Latency compensation delays per node.
    latency_compensation: HashMap<NodeId, usize>,
}

/// Entry for a node in the graph.
struct NodeEntry {
    /// The node itself.
    node: BoxedNode,
    /// Cached node info.
    info: NodeInfo,
}

impl AudioGraph {
    /// Creates a new audio graph.
    #[must_use]
    pub fn new(sample_rate: f32, buffer_size: usize) -> Self {
        Self {
            nodes: SlotMap::new(),
            connections: Vec::new(),
            sample_rate,
            buffer_size,
            dirty: true,
            processing_order: Vec::new(),
            latency_compensation: HashMap::new(),
        }
    }

    /// Returns the sample rate.
    #[must_use]
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Returns the buffer size.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: impl AudioNode + 'static) -> NodeId {
        let info = node.info();
        let key = self.nodes.insert(NodeEntry {
            node: Box::new(node),
            info,
        });
        self.dirty = true;
        NodeId(key)
    }

    /// Removes a node from the graph.
    ///
    /// Also removes all connections to/from this node.
    pub fn remove_node(&mut self, node_id: NodeId) -> Result<()> {
        if self.nodes.remove(node_id.0).is_none() {
            return Err(Error::NodeNotFound(node_id));
        }

        // Remove all connections involving this node
        self.connections.retain(|c| {
            c.source_node != node_id && c.dest_node != node_id
        });

        self.dirty = true;
        Ok(())
    }

    /// Gets a reference to a node.
    pub fn get_node(&self, node_id: NodeId) -> Result<&dyn AudioNode> {
        match self.nodes.get(node_id.0) {
            Some(entry) => Ok(entry.node.as_ref()),
            None => Err(Error::NodeNotFound(node_id)),
        }
    }

    /// Gets a mutable reference to a node.
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Result<&mut dyn AudioNode> {
        match self.nodes.get_mut(node_id.0) {
            Some(entry) => Ok(entry.node.as_mut()),
            None => Err(Error::NodeNotFound(node_id)),
        }
    }

    /// Connects two nodes.
    pub fn connect(
        &mut self,
        source_node: NodeId,
        source_port: usize,
        dest_node: NodeId,
        dest_port: usize,
    ) -> Result<()> {
        // Validate source node and port
        let source_info = self
            .nodes
            .get(source_node.0)
            .ok_or(Error::NodeNotFound(source_node))?
            .info
            .clone();

        if source_port >= source_info.output_count {
            return Err(Error::PortNotFound {
                node: source_node,
                port: source_port,
                max: source_info.output_count.saturating_sub(1),
            });
        }

        // Validate dest node and port
        let dest_info = self
            .nodes
            .get(dest_node.0)
            .ok_or(Error::NodeNotFound(dest_node))?
            .info
            .clone();

        if dest_port >= dest_info.input_count {
            return Err(Error::PortNotFound {
                node: dest_node,
                port: dest_port,
                max: dest_info.input_count.saturating_sub(1),
            });
        }

        let connection = Connection::new(source_node, source_port, dest_node, dest_port);

        // Check for duplicate
        if self.connections.contains(&connection) {
            return Err(Error::DuplicateConnection);
        }

        // Check for cycle (simple check: source can't be dest's descendant)
        if self.would_create_cycle(source_node, dest_node) {
            return Err(Error::CycleDetected);
        }

        self.connections.push(connection);
        self.dirty = true;
        Ok(())
    }

    /// Disconnects two nodes.
    pub fn disconnect(
        &mut self,
        source_node: NodeId,
        source_port: usize,
        dest_node: NodeId,
        dest_port: usize,
    ) -> Result<()> {
        let connection = Connection::new(source_node, source_port, dest_node, dest_port);

        let idx = self
            .connections
            .iter()
            .position(|c| *c == connection)
            .ok_or(Error::NodeNotFound(source_node))?;

        self.connections.remove(idx);
        self.dirty = true;
        Ok(())
    }

    /// Checks if adding a connection would create a cycle.
    fn would_create_cycle(&self, source: NodeId, dest: NodeId) -> bool {
        // If source is reachable from dest, adding dest->source would create a cycle
        let mut visited = HashSet::new();
        let mut stack = vec![dest];

        while let Some(node) = stack.pop() {
            if node == source {
                return true;
            }

            if visited.insert(node) {
                // Add all nodes that this node connects to
                for conn in &self.connections {
                    if conn.source_node == node {
                        stack.push(conn.dest_node);
                    }
                }
            }
        }

        false
    }

    /// Compiles the graph for processing.
    ///
    /// This performs:
    /// 1. Topological sorting to determine processing order
    /// 2. Latency analysis for PDC (Plugin Delay Compensation)
    pub fn compile(&mut self) -> Result<()> {
        // Topological sort using Kahn's algorithm
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        // Initialize
        for key in self.nodes.keys() {
            let node_id = NodeId(key);
            in_degree.insert(node_id, 0);
            adjacency.insert(node_id, Vec::new());
        }

        // Build adjacency and in-degree
        for conn in &self.connections {
            adjacency
                .get_mut(&conn.source_node)
                .unwrap()
                .push(conn.dest_node);
            *in_degree.get_mut(&conn.dest_node).unwrap() += 1;
        }

        // Find all nodes with no incoming edges
        let mut queue: Vec<NodeId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();

        while let Some(node) = queue.pop() {
            order.push(node);

            for &neighbor in adjacency.get(&node).unwrap() {
                let deg = in_degree.get_mut(&neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(neighbor);
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(Error::CycleDetected);
        }

        self.processing_order = order;

        // Calculate latency compensation (simplified)
        self.calculate_latency_compensation();

        self.dirty = false;
        Ok(())
    }

    /// Calculates latency compensation for each node.
    fn calculate_latency_compensation(&mut self) {
        self.latency_compensation.clear();

        // For now, simple implementation: no compensation
        // Full implementation would trace paths and add delays
        for key in self.nodes.keys() {
            self.latency_compensation.insert(NodeId(key), 0);
        }
    }

    /// Returns whether the graph needs recompilation.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Creates a processor for this graph.
    pub fn create_processor(&self) -> Result<GraphProcessor> {
        if self.dirty {
            return Err(Error::NotCompiled);
        }

        Ok(GraphProcessor::new(
            self.processing_order.clone(),
            self.connections.clone(),
            self.buffer_size,
        ))
    }

    /// Returns the number of nodes in the graph.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of connections in the graph.
    #[must_use]
    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::{GainNode, InputNode, MixerNode, OutputNode};

    #[test]
    fn test_add_remove_node() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let node = graph.add_node(GainNode::new(1.0));
        assert_eq!(graph.node_count(), 1);

        graph.remove_node(node).unwrap();
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();

        // This would create a cycle
        let result = graph.connect(c, 0, a, 0);
        assert!(matches!(result, Err(Error::CycleDetected)));
    }

    // =========================================================================
    // Phase 4 TDD: Comprehensive audio graph tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // Topological Sorting Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_topological_sort_linear_chain() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));
        let d = graph.add_node(GainNode::new(1.0));

        // Linear chain: a -> b -> c -> d
        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        graph.connect(c, 0, d, 0).unwrap();

        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let order = processor.processing_order();

        // Verify ordering constraints
        let pos_a = order.iter().position(|&n| n == a).unwrap();
        let pos_b = order.iter().position(|&n| n == b).unwrap();
        let pos_c = order.iter().position(|&n| n == c).unwrap();
        let pos_d = order.iter().position(|&n| n == d).unwrap();

        assert!(pos_a < pos_b, "a must come before b");
        assert!(pos_b < pos_c, "b must come before c");
        assert!(pos_c < pos_d, "c must come before d");
    }

    #[test]
    fn test_topological_sort_diamond() {
        let mut graph = AudioGraph::new(48000.0, 512);

        //     b
        //    / \
        // a      d
        //    \ /
        //     c
        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));
        let d = graph.add_node(MixerNode::new(2));

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(a, 0, c, 0).unwrap();
        graph.connect(b, 0, d, 0).unwrap();
        graph.connect(c, 0, d, 1).unwrap();

        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let order = processor.processing_order();

        let pos_a = order.iter().position(|&n| n == a).unwrap();
        let pos_b = order.iter().position(|&n| n == b).unwrap();
        let pos_c = order.iter().position(|&n| n == c).unwrap();
        let pos_d = order.iter().position(|&n| n == d).unwrap();

        assert!(pos_a < pos_b, "a must come before b");
        assert!(pos_a < pos_c, "a must come before c");
        assert!(pos_b < pos_d, "b must come before d");
        assert!(pos_c < pos_d, "c must come before d");
    }

    #[test]
    fn test_topological_sort_parallel_chains() {
        let mut graph = AudioGraph::new(48000.0, 512);

        // Two independent chains:
        // a1 -> b1 -> c1
        // a2 -> b2 -> c2
        let a1 = graph.add_node(GainNode::new(1.0));
        let b1 = graph.add_node(GainNode::new(1.0));
        let c1 = graph.add_node(GainNode::new(1.0));
        let a2 = graph.add_node(GainNode::new(1.0));
        let b2 = graph.add_node(GainNode::new(1.0));
        let c2 = graph.add_node(GainNode::new(1.0));

        graph.connect(a1, 0, b1, 0).unwrap();
        graph.connect(b1, 0, c1, 0).unwrap();
        graph.connect(a2, 0, b2, 0).unwrap();
        graph.connect(b2, 0, c2, 0).unwrap();

        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let order = processor.processing_order();

        // Each chain should maintain internal order
        let pos_a1 = order.iter().position(|&n| n == a1).unwrap();
        let pos_b1 = order.iter().position(|&n| n == b1).unwrap();
        let pos_c1 = order.iter().position(|&n| n == c1).unwrap();
        let pos_a2 = order.iter().position(|&n| n == a2).unwrap();
        let pos_b2 = order.iter().position(|&n| n == b2).unwrap();
        let pos_c2 = order.iter().position(|&n| n == c2).unwrap();

        assert!(pos_a1 < pos_b1 && pos_b1 < pos_c1, "chain 1 order");
        assert!(pos_a2 < pos_b2 && pos_b2 < pos_c2, "chain 2 order");
    }

    #[test]
    fn test_topological_sort_complex_graph() {
        let mut graph = AudioGraph::new(48000.0, 512);

        // Complex graph:
        //   a -> b -> d
        //   |    |    |
        //   v    v    v
        //   c -> e -> f
        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));
        let d = graph.add_node(GainNode::new(1.0));
        let e = graph.add_node(MixerNode::new(2));
        let f = graph.add_node(MixerNode::new(2));

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(a, 0, c, 0).unwrap();
        graph.connect(b, 0, d, 0).unwrap();
        graph.connect(b, 0, e, 0).unwrap();
        graph.connect(c, 0, e, 1).unwrap();
        graph.connect(d, 0, f, 0).unwrap();
        graph.connect(e, 0, f, 1).unwrap();

        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let order = processor.processing_order();

        // Verify all ordering constraints
        let positions: HashMap<NodeId, usize> = order
            .iter()
            .enumerate()
            .map(|(i, &n)| (n, i))
            .collect();

        assert!(positions[&a] < positions[&b]);
        assert!(positions[&a] < positions[&c]);
        assert!(positions[&b] < positions[&d]);
        assert!(positions[&b] < positions[&e]);
        assert!(positions[&c] < positions[&e]);
        assert!(positions[&d] < positions[&f]);
        assert!(positions[&e] < positions[&f]);
    }

    #[test]
    fn test_topological_sort_disconnected_nodes() {
        let mut graph = AudioGraph::new(48000.0, 512);

        // Three disconnected nodes
        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));

        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let order = processor.processing_order();

        // All nodes should be in the order
        assert_eq!(order.len(), 3);
        assert!(order.contains(&a));
        assert!(order.contains(&b));
        assert!(order.contains(&c));
    }

    // -------------------------------------------------------------------------
    // Cycle Detection Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cycle_detection_self_loop() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));

        // Self-loop: a -> a
        let result = graph.connect(a, 0, a, 0);
        assert!(
            matches!(result, Err(Error::CycleDetected)),
            "Self-loop should be rejected"
        );
    }

    #[test]
    fn test_cycle_detection_two_node_cycle() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();

        // Would create: a -> b -> a
        let result = graph.connect(b, 0, a, 0);
        assert!(
            matches!(result, Err(Error::CycleDetected)),
            "Two-node cycle should be rejected"
        );
    }

    #[test]
    fn test_cycle_detection_indirect_cycle() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));
        let d = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        graph.connect(c, 0, d, 0).unwrap();

        // Would create cycle through 4 nodes
        let result = graph.connect(d, 0, a, 0);
        assert!(
            matches!(result, Err(Error::CycleDetected)),
            "Indirect cycle should be rejected"
        );
    }

    #[test]
    fn test_valid_diamond_is_not_cycle() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));
        let d = graph.add_node(MixerNode::new(2));

        // Diamond pattern is valid (not a cycle)
        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(a, 0, c, 0).unwrap();
        graph.connect(b, 0, d, 0).unwrap();
        graph.connect(c, 0, d, 1).unwrap();

        // Should compile successfully
        assert!(graph.compile().is_ok());
    }

    // -------------------------------------------------------------------------
    // Connection Management Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_duplicate_connection_rejected() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();

        let result = graph.connect(a, 0, b, 0);
        assert!(
            matches!(result, Err(Error::DuplicateConnection)),
            "Duplicate connection should be rejected"
        );
    }

    #[test]
    fn test_invalid_source_port_rejected() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));

        // GainNode has only 1 output (port 0)
        let result = graph.connect(a, 99, b, 0);
        assert!(
            matches!(result, Err(Error::PortNotFound { .. })),
            "Invalid source port should be rejected"
        );
    }

    #[test]
    fn test_invalid_dest_port_rejected() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));

        // GainNode has only 1 input (port 0)
        let result = graph.connect(a, 0, b, 99);
        assert!(
            matches!(result, Err(Error::PortNotFound { .. })),
            "Invalid dest port should be rejected"
        );
    }

    #[test]
    fn test_connect_to_nonexistent_node() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));

        // Remove b, then try to connect to it
        graph.remove_node(b).unwrap();

        let result = graph.connect(a, 0, b, 0);
        assert!(
            matches!(result, Err(Error::NodeNotFound(_))),
            "Connection to removed node should fail"
        );
    }

    #[test]
    fn test_disconnect() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();
        assert_eq!(graph.connection_count(), 1);

        graph.disconnect(a, 0, b, 0).unwrap();
        assert_eq!(graph.connection_count(), 0);
    }

    #[test]
    fn test_remove_node_cleans_connections() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        assert_eq!(graph.connection_count(), 2);

        // Remove middle node - should remove both connections
        graph.remove_node(b).unwrap();
        assert_eq!(graph.connection_count(), 0);
        assert_eq!(graph.node_count(), 2);
    }

    // -------------------------------------------------------------------------
    // Dirty Flag and Compilation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_graph_dirty_on_add_node() {
        let mut graph = AudioGraph::new(48000.0, 512);

        assert!(graph.is_dirty(), "New graph should be dirty");

        graph.compile().unwrap();
        assert!(!graph.is_dirty(), "Compiled graph should not be dirty");

        graph.add_node(GainNode::new(1.0));
        assert!(graph.is_dirty(), "Graph should be dirty after adding node");
    }

    #[test]
    fn test_graph_dirty_on_remove_node() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        graph.compile().unwrap();
        assert!(!graph.is_dirty());

        graph.remove_node(a).unwrap();
        assert!(graph.is_dirty(), "Graph should be dirty after removing node");
    }

    #[test]
    fn test_graph_dirty_on_connect() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        graph.compile().unwrap();

        graph.connect(a, 0, b, 0).unwrap();
        assert!(graph.is_dirty(), "Graph should be dirty after connecting");
    }

    #[test]
    fn test_graph_dirty_on_disconnect() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        graph.connect(a, 0, b, 0).unwrap();
        graph.compile().unwrap();

        graph.disconnect(a, 0, b, 0).unwrap();
        assert!(
            graph.is_dirty(),
            "Graph should be dirty after disconnecting"
        );
    }

    #[test]
    fn test_processor_requires_compilation() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        graph.connect(a, 0, b, 0).unwrap();

        // Without compile
        let result = graph.create_processor();
        assert!(
            matches!(result, Err(Error::NotCompiled)),
            "Processor creation should fail without compilation"
        );

        // After compile
        graph.compile().unwrap();
        assert!(
            graph.create_processor().is_ok(),
            "Processor creation should succeed after compilation"
        );
    }

    // -------------------------------------------------------------------------
    // Node Access Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_node() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let id = graph.add_node(GainNode::new(0.5));

        let node = graph.get_node(id).unwrap();
        assert_eq!(node.name(), "Gain");
    }

    #[test]
    fn test_get_node_mut() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let id = graph.add_node(GainNode::new(0.5));

        let _node = graph.get_node_mut(id).unwrap();
        // Node is mutable - can modify parameters
    }

    #[test]
    fn test_get_removed_node_fails() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let id = graph.add_node(GainNode::new(0.5));
        graph.remove_node(id).unwrap();

        assert!(matches!(graph.get_node(id), Err(Error::NodeNotFound(_))));
    }

    // -------------------------------------------------------------------------
    // Graph Properties Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sample_rate() {
        let graph = AudioGraph::new(44100.0, 256);
        assert!((graph.sample_rate() - 44100.0).abs() < 0.01);
    }

    #[test]
    fn test_buffer_size() {
        let graph = AudioGraph::new(48000.0, 512);
        assert_eq!(graph.buffer_size(), 512);
    }

    #[test]
    fn test_node_count() {
        let mut graph = AudioGraph::new(48000.0, 512);

        assert_eq!(graph.node_count(), 0);

        let a = graph.add_node(GainNode::new(1.0));
        assert_eq!(graph.node_count(), 1);

        graph.add_node(GainNode::new(1.0));
        assert_eq!(graph.node_count(), 2);

        graph.remove_node(a).unwrap();
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_connection_count() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));

        assert_eq!(graph.connection_count(), 0);

        graph.connect(a, 0, b, 0).unwrap();
        assert_eq!(graph.connection_count(), 1);

        graph.connect(b, 0, c, 0).unwrap();
        assert_eq!(graph.connection_count(), 2);

        graph.disconnect(a, 0, b, 0).unwrap();
        assert_eq!(graph.connection_count(), 1);
    }

    // -------------------------------------------------------------------------
    // I/O Node Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_input_output_pipeline() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let input = graph.add_node(InputNode::new(2));
        let gain = graph.add_node(GainNode::new(0.5));
        let output = graph.add_node(OutputNode::new(2));

        graph.connect(input, 0, gain, 0).unwrap();
        graph.connect(gain, 0, output, 0).unwrap();

        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let order = processor.processing_order();

        // Input should come first, output last
        let pos_input = order.iter().position(|&n| n == input).unwrap();
        let pos_gain = order.iter().position(|&n| n == gain).unwrap();
        let pos_output = order.iter().position(|&n| n == output).unwrap();

        assert!(pos_input < pos_gain);
        assert!(pos_gain < pos_output);
    }

    // -------------------------------------------------------------------------
    // Mixer Node Graph Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mixer_multi_input_graph() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let input1 = graph.add_node(InputNode::new(2));
        let input2 = graph.add_node(InputNode::new(2));
        let input3 = graph.add_node(InputNode::new(2));
        let mixer = graph.add_node(MixerNode::new(3));
        let output = graph.add_node(OutputNode::new(2));

        graph.connect(input1, 0, mixer, 0).unwrap();
        graph.connect(input2, 0, mixer, 1).unwrap();
        graph.connect(input3, 0, mixer, 2).unwrap();
        graph.connect(mixer, 0, output, 0).unwrap();

        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();

        // Verify mixer receives from all inputs
        let mixer_inputs: Vec<_> = processor.inputs_for(mixer).collect();
        assert_eq!(mixer_inputs.len(), 3);
    }

    // -------------------------------------------------------------------------
    // Empty Graph Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_graph_compiles() {
        let mut graph = AudioGraph::new(48000.0, 512);
        assert!(graph.compile().is_ok());
    }

    #[test]
    fn test_empty_graph_processor() {
        let mut graph = AudioGraph::new(48000.0, 512);
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        assert_eq!(processor.processing_order().len(), 0);
        assert_eq!(processor.connections().len(), 0);
    }
}
