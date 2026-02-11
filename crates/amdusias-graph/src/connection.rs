//! Connection types for the audio graph.

use crate::NodeId;

/// A connection between two nodes in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connection {
    /// Source node.
    pub source_node: NodeId,
    /// Source port index.
    pub source_port: usize,
    /// Destination node.
    pub dest_node: NodeId,
    /// Destination port index.
    pub dest_port: usize,
}

impl Connection {
    /// Creates a new connection.
    #[must_use]
    pub const fn new(
        source_node: NodeId,
        source_port: usize,
        dest_node: NodeId,
        dest_port: usize,
    ) -> Self {
        Self {
            source_node,
            source_port,
            dest_node,
            dest_port,
        }
    }
}

/// Builder for creating connections.
pub struct ConnectionBuilder {
    source_node: NodeId,
    source_port: usize,
}

impl ConnectionBuilder {
    /// Creates a new connection builder.
    #[must_use]
    pub const fn from(node: NodeId, port: usize) -> Self {
        Self {
            source_node: node,
            source_port: port,
        }
    }

    /// Completes the connection to a destination.
    #[must_use]
    pub const fn to(self, node: NodeId, port: usize) -> Connection {
        Connection::new(self.source_node, self.source_port, node, port)
    }
}
