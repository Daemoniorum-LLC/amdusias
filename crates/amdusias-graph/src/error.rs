//! Error types for the audio graph.

use crate::NodeId;
use thiserror::Error;

/// Result type for graph operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur in audio graph operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Node not found in graph.
    #[error("node not found: {0:?}")]
    NodeNotFound(NodeId),

    /// Port index out of bounds.
    #[error("port {port} not found on node {node:?} (max: {max})")]
    PortNotFound {
        /// The node ID.
        node: NodeId,
        /// The requested port index.
        port: usize,
        /// Maximum valid port index.
        max: usize,
    },

    /// Connection would create a cycle.
    #[error("connection would create a cycle")]
    CycleDetected,

    /// Connection already exists.
    #[error("connection already exists")]
    DuplicateConnection,

    /// Graph is not compiled.
    #[error("graph must be compiled before processing")]
    NotCompiled,

    /// Buffer size mismatch.
    #[error("buffer size mismatch: expected {expected}, got {actual}")]
    BufferSizeMismatch {
        /// Expected buffer size.
        expected: usize,
        /// Actual buffer size.
        actual: usize,
    },

    /// Channel count mismatch.
    #[error("channel count mismatch at connection")]
    ChannelMismatch,
}
