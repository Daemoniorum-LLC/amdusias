//! # amdusias-graph
//!
//! Audio graph system for routing and processing.
//!
//! This crate provides a node-based audio processing graph with:
//!
//! - **Automatic latency compensation** (PDC - Plugin Delay Compensation)
//! - **Topological sorting** for correct processing order
//! - **Lock-free graph updates** from non-audio threads
//! - **Flexible routing** (any node to any node)
//!
//! ## Example
//!
//! ```rust,ignore
//! use amdusias_graph::{AudioGraph, nodes::GainNode};
//!
//! let mut graph = AudioGraph::new(48000.0, 512);
//!
//! // Add nodes
//! let input = graph.add_input_node(2);
//! let gain = graph.add_node(GainNode::new(0.5));
//! let output = graph.add_output_node(2);
//!
//! // Connect nodes
//! graph.connect(input, 0, gain, 0)?;
//! graph.connect(gain, 0, output, 0)?;
//!
//! // Compile for processing
//! graph.compile()?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod connection;
pub mod error;
pub mod graph;
pub mod node;
pub mod nodes;
pub mod processor;

pub use connection::Connection;
pub use error::{Error, Result};
pub use graph::AudioGraph;
pub use node::{AudioNode, NodeId, NodeInfo};
pub use processor::GraphProcessor;
