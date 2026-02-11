//! Graph processor for audio thread execution.

use crate::{Connection, NodeId};
use amdusias_core::AudioBuffer;
use std::collections::HashMap;

/// Compiled graph processor for the audio thread.
///
/// This struct contains only the data needed for processing,
/// without any graph modification capabilities.
pub struct GraphProcessor {
    /// Processing order (topologically sorted).
    processing_order: Vec<NodeId>,
    /// Connections for routing.
    connections: Vec<Connection>,
    /// Buffer storage for intermediate results.
    buffers: HashMap<(NodeId, usize), AudioBuffer<2>>,
    /// Buffer size.
    buffer_size: usize,
}

impl GraphProcessor {
    /// Creates a new graph processor.
    pub(crate) fn new(
        processing_order: Vec<NodeId>,
        connections: Vec<Connection>,
        buffer_size: usize,
    ) -> Self {
        Self {
            processing_order,
            connections,
            buffers: HashMap::new(),
            buffer_size,
        }
    }

    /// Returns the processing order.
    #[must_use]
    pub fn processing_order(&self) -> &[NodeId] {
        &self.processing_order
    }

    /// Returns the connections.
    #[must_use]
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Returns the buffer size.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }

    /// Gets incoming connections for a node.
    pub fn inputs_for(&self, node: NodeId) -> impl Iterator<Item = &Connection> {
        self.connections.iter().filter(move |c| c.dest_node == node)
    }

    /// Gets outgoing connections from a node.
    pub fn outputs_from(&self, node: NodeId) -> impl Iterator<Item = &Connection> {
        self.connections
            .iter()
            .filter(move |c| c.source_node == node)
    }
}

/// Context passed to nodes during processing.
pub struct ProcessContext<'a> {
    /// Sample rate.
    pub sample_rate: f32,
    /// Buffer size.
    pub buffer_size: usize,
    /// Current transport position in samples.
    pub transport_position: u64,
    /// Whether transport is playing.
    pub is_playing: bool,
    /// Tempo in BPM (if available).
    pub tempo: Option<f32>,
    /// Time signature (numerator, denominator).
    pub time_signature: Option<(u8, u8)>,
    /// Processor reference.
    processor: &'a GraphProcessor,
}

impl<'a> ProcessContext<'a> {
    /// Creates a new process context.
    #[must_use]
    pub fn new(processor: &'a GraphProcessor, sample_rate: f32) -> Self {
        Self {
            sample_rate,
            buffer_size: processor.buffer_size,
            transport_position: 0,
            is_playing: false,
            tempo: None,
            time_signature: None,
            processor,
        }
    }

    /// Sets the transport position.
    pub fn set_transport(&mut self, position: u64, is_playing: bool) {
        self.transport_position = position;
        self.is_playing = is_playing;
    }

    /// Sets tempo and time signature.
    pub fn set_tempo(&mut self, tempo: f32, time_sig: (u8, u8)) {
        self.tempo = Some(tempo);
        self.time_signature = Some(time_sig);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::AudioGraph;
    use crate::nodes::{GainNode, InputNode, MixerNode, OutputNode};

    // =========================================================================
    // Phase 4 TDD: GraphProcessor tests
    // =========================================================================

    #[test]
    fn test_processor_processing_order() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let order = processor.processing_order();

        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_processor_connections() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(GainNode::new(1.0));
        let b = graph.add_node(GainNode::new(1.0));
        let c = graph.add_node(GainNode::new(1.0));

        graph.connect(a, 0, b, 0).unwrap();
        graph.connect(b, 0, c, 0).unwrap();
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let connections = processor.connections();

        assert_eq!(connections.len(), 2);
    }

    #[test]
    fn test_processor_buffer_size() {
        let mut graph = AudioGraph::new(48000.0, 256);
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        assert_eq!(processor.buffer_size(), 256);
    }

    #[test]
    fn test_processor_inputs_for_node() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let a = graph.add_node(InputNode::new(2));
        let b = graph.add_node(InputNode::new(2));
        let mixer = graph.add_node(MixerNode::new(2));

        graph.connect(a, 0, mixer, 0).unwrap();
        graph.connect(b, 0, mixer, 1).unwrap();
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let mixer_inputs: Vec<_> = processor.inputs_for(mixer).collect();

        assert_eq!(mixer_inputs.len(), 2);
        assert!(mixer_inputs.iter().any(|c| c.source_node == a));
        assert!(mixer_inputs.iter().any(|c| c.source_node == b));
    }

    #[test]
    fn test_processor_outputs_from_node() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let input = graph.add_node(InputNode::new(2));
        let gain1 = graph.add_node(GainNode::new(0.5));
        let gain2 = graph.add_node(GainNode::new(0.5));

        graph.connect(input, 0, gain1, 0).unwrap();
        graph.connect(input, 0, gain2, 0).unwrap();
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let input_outputs: Vec<_> = processor.outputs_from(input).collect();

        assert_eq!(input_outputs.len(), 2);
        assert!(input_outputs.iter().any(|c| c.dest_node == gain1));
        assert!(input_outputs.iter().any(|c| c.dest_node == gain2));
    }

    #[test]
    fn test_processor_node_with_no_inputs() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let input = graph.add_node(InputNode::new(2));
        let output = graph.add_node(OutputNode::new(2));

        graph.connect(input, 0, output, 0).unwrap();
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();

        // Input node has no incoming connections
        let input_inputs: Vec<_> = processor.inputs_for(input).collect();
        assert_eq!(input_inputs.len(), 0);
    }

    #[test]
    fn test_processor_node_with_no_outputs() {
        let mut graph = AudioGraph::new(48000.0, 512);

        let input = graph.add_node(InputNode::new(2));
        let output = graph.add_node(OutputNode::new(2));

        graph.connect(input, 0, output, 0).unwrap();
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();

        // Output node has no outgoing connections
        let output_outputs: Vec<_> = processor.outputs_from(output).collect();
        assert_eq!(output_outputs.len(), 0);
    }

    // =========================================================================
    // ProcessContext tests
    // =========================================================================

    #[test]
    fn test_process_context_creation() {
        let mut graph = AudioGraph::new(48000.0, 512);
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let ctx = ProcessContext::new(&processor, 48000.0);

        assert!((ctx.sample_rate - 48000.0).abs() < 0.01);
        assert_eq!(ctx.buffer_size, 512);
        assert_eq!(ctx.transport_position, 0);
        assert!(!ctx.is_playing);
        assert!(ctx.tempo.is_none());
        assert!(ctx.time_signature.is_none());
    }

    #[test]
    fn test_process_context_transport() {
        let mut graph = AudioGraph::new(48000.0, 512);
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let mut ctx = ProcessContext::new(&processor, 48000.0);

        ctx.set_transport(44100, true); // 1 second at 44.1kHz

        assert_eq!(ctx.transport_position, 44100);
        assert!(ctx.is_playing);
    }

    #[test]
    fn test_process_context_tempo() {
        let mut graph = AudioGraph::new(48000.0, 512);
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let mut ctx = ProcessContext::new(&processor, 48000.0);

        ctx.set_tempo(120.0, (4, 4));

        assert_eq!(ctx.tempo, Some(120.0));
        assert_eq!(ctx.time_signature, Some((4, 4)));
    }

    #[test]
    fn test_process_context_various_sample_rates() {
        let sample_rates = [44100.0, 48000.0, 88200.0, 96000.0, 192000.0];

        for &sr in &sample_rates {
            let mut graph = AudioGraph::new(sr, 512);
            graph.compile().unwrap();

            let processor = graph.create_processor().unwrap();
            let ctx = ProcessContext::new(&processor, sr);

            assert!((ctx.sample_rate - sr).abs() < 0.01);
        }
    }

    #[test]
    fn test_process_context_various_buffer_sizes() {
        let buffer_sizes = [64, 128, 256, 512, 1024, 2048];

        for &size in &buffer_sizes {
            let mut graph = AudioGraph::new(48000.0, size);
            graph.compile().unwrap();

            let processor = graph.create_processor().unwrap();
            let ctx = ProcessContext::new(&processor, 48000.0);

            assert_eq!(ctx.buffer_size, size);
        }
    }

    #[test]
    fn test_process_context_odd_time_signatures() {
        let mut graph = AudioGraph::new(48000.0, 512);
        graph.compile().unwrap();

        let processor = graph.create_processor().unwrap();
        let mut ctx = ProcessContext::new(&processor, 48000.0);

        // Test odd time signatures
        let signatures = [(3, 4), (5, 4), (6, 8), (7, 8), (12, 8)];

        for &sig in &signatures {
            ctx.set_tempo(120.0, sig);
            assert_eq!(ctx.time_signature, Some(sig));
        }
    }
}
