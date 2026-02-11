//! Audio node traits and types.

use amdusias_core::AudioBuffer;

/// Unique identifier for a node in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(crate) slotmap::DefaultKey);

impl NodeId {
    /// Creates a node ID from a raw key (for testing).
    #[cfg(test)]
    pub fn from_raw(key: slotmap::DefaultKey) -> Self {
        Self(key)
    }
}

/// Information about a node's ports.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Number of input ports.
    pub input_count: usize,
    /// Number of output ports.
    pub output_count: usize,
    /// Channels per input port.
    pub input_channels: Vec<usize>,
    /// Channels per output port.
    pub output_channels: Vec<usize>,
    /// Latency introduced by this node in samples.
    pub latency_samples: usize,
}

impl NodeInfo {
    /// Creates info for a simple mono-in/mono-out node.
    #[must_use]
    pub fn mono() -> Self {
        Self {
            input_count: 1,
            output_count: 1,
            input_channels: vec![1],
            output_channels: vec![1],
            latency_samples: 0,
        }
    }

    /// Creates info for a simple stereo-in/stereo-out node.
    #[must_use]
    pub fn stereo() -> Self {
        Self {
            input_count: 1,
            output_count: 1,
            input_channels: vec![2],
            output_channels: vec![2],
            latency_samples: 0,
        }
    }

    /// Creates info with custom port configuration.
    #[must_use]
    pub fn custom(
        input_channels: Vec<usize>,
        output_channels: Vec<usize>,
        latency_samples: usize,
    ) -> Self {
        Self {
            input_count: input_channels.len(),
            output_count: output_channels.len(),
            input_channels,
            output_channels,
            latency_samples,
        }
    }
}

/// Trait for audio processing nodes.
pub trait AudioNode: Send {
    /// Returns information about this node's ports.
    fn info(&self) -> NodeInfo;

    /// Processes audio data.
    ///
    /// # Arguments
    ///
    /// - `inputs`: Input buffers (one per input port).
    /// - `outputs`: Output buffers to fill (one per output port).
    /// - `frames`: Number of frames to process.
    fn process(&mut self, inputs: &[&AudioBuffer<2>], outputs: &mut [AudioBuffer<2>], frames: usize);

    /// Resets the node state.
    fn reset(&mut self);

    /// Called when the sample rate changes.
    fn set_sample_rate(&mut self, _sample_rate: f32) {}

    /// Returns the node's name for debugging.
    fn name(&self) -> &'static str {
        "AudioNode"
    }
}

/// A boxed audio node.
pub type BoxedNode = Box<dyn AudioNode>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_info() {
        let info = NodeInfo::stereo();
        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels[0], 2);
    }

    // =========================================================================
    // Phase 4 TDD: Comprehensive NodeInfo tests
    // =========================================================================

    #[test]
    fn test_node_info_mono() {
        let info = NodeInfo::mono();

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels.len(), 1);
        assert_eq!(info.output_channels.len(), 1);
        assert_eq!(info.input_channels[0], 1);
        assert_eq!(info.output_channels[0], 1);
        assert_eq!(info.latency_samples, 0);
    }

    #[test]
    fn test_node_info_stereo() {
        let info = NodeInfo::stereo();

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels.len(), 1);
        assert_eq!(info.output_channels.len(), 1);
        assert_eq!(info.input_channels[0], 2);
        assert_eq!(info.output_channels[0], 2);
        assert_eq!(info.latency_samples, 0);
    }

    #[test]
    fn test_node_info_custom_single_port() {
        let info = NodeInfo::custom(vec![2], vec![2], 128);

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels[0], 2);
        assert_eq!(info.output_channels[0], 2);
        assert_eq!(info.latency_samples, 128);
    }

    #[test]
    fn test_node_info_custom_multi_port() {
        // Mixer-style: 4 stereo inputs, 1 stereo output
        let info = NodeInfo::custom(vec![2, 2, 2, 2], vec![2], 0);

        assert_eq!(info.input_count, 4);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels.len(), 4);
        assert_eq!(info.output_channels.len(), 1);
    }

    #[test]
    fn test_node_info_custom_no_inputs() {
        // Generator node: no inputs, 1 stereo output
        let info = NodeInfo::custom(vec![], vec![2], 0);

        assert_eq!(info.input_count, 0);
        assert_eq!(info.output_count, 1);
        assert!(info.input_channels.is_empty());
        assert_eq!(info.output_channels[0], 2);
    }

    #[test]
    fn test_node_info_custom_no_outputs() {
        // Analyzer/sink node: 1 stereo input, no outputs
        let info = NodeInfo::custom(vec![2], vec![], 0);

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 0);
        assert_eq!(info.input_channels[0], 2);
        assert!(info.output_channels.is_empty());
    }

    #[test]
    fn test_node_info_custom_with_latency() {
        // Node with lookahead/latency
        let info = NodeInfo::custom(vec![2], vec![2], 512);

        assert_eq!(info.latency_samples, 512);
    }

    #[test]
    fn test_node_info_custom_mixed_channels() {
        // Node with different channel counts per port
        // 2 stereo inputs + 1 mono input -> 1 5.1 surround output
        let info = NodeInfo::custom(vec![2, 2, 1], vec![6], 0);

        assert_eq!(info.input_count, 3);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels[0], 2);
        assert_eq!(info.input_channels[1], 2);
        assert_eq!(info.input_channels[2], 1);
        assert_eq!(info.output_channels[0], 6);
    }

    #[test]
    fn test_node_info_custom_splitter() {
        // Splitter: 1 stereo input -> 4 stereo outputs
        let info = NodeInfo::custom(vec![2], vec![2, 2, 2, 2], 0);

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 4);
        assert_eq!(info.input_channels[0], 2);
        for &ch in &info.output_channels {
            assert_eq!(ch, 2);
        }
    }

    #[test]
    fn test_node_info_clone() {
        let info = NodeInfo::custom(vec![2, 2], vec![2], 256);
        let cloned = info.clone();

        assert_eq!(cloned.input_count, info.input_count);
        assert_eq!(cloned.output_count, info.output_count);
        assert_eq!(cloned.input_channels, info.input_channels);
        assert_eq!(cloned.output_channels, info.output_channels);
        assert_eq!(cloned.latency_samples, info.latency_samples);
    }

    #[test]
    fn test_node_info_debug() {
        let info = NodeInfo::stereo();
        let debug_str = format!("{:?}", info);

        assert!(debug_str.contains("NodeInfo"));
        assert!(debug_str.contains("input_count"));
        assert!(debug_str.contains("output_count"));
    }
}
