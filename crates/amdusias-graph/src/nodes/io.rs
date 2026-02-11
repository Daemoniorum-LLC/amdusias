//! Input and output nodes.

use crate::node::{AudioNode, NodeInfo};
use amdusias_core::AudioBuffer;

/// Input node (receives audio from external source).
#[derive(Debug)]
pub struct InputNode {
    channels: usize,
}

impl InputNode {
    /// Creates a new input node.
    #[must_use]
    pub fn new(channels: usize) -> Self {
        Self { channels }
    }
}

impl AudioNode for InputNode {
    fn info(&self) -> NodeInfo {
        NodeInfo::custom(vec![], vec![self.channels], 0)
    }

    fn process(&mut self, _inputs: &[&AudioBuffer<2>], _outputs: &mut [AudioBuffer<2>], _frames: usize) {
        // Input is filled externally before graph processing
    }

    fn reset(&mut self) {}

    fn name(&self) -> &'static str {
        "Input"
    }
}

/// Output node (sends audio to external destination).
#[derive(Debug)]
pub struct OutputNode {
    channels: usize,
}

impl OutputNode {
    /// Creates a new output node.
    #[must_use]
    pub fn new(channels: usize) -> Self {
        Self { channels }
    }
}

impl AudioNode for OutputNode {
    fn info(&self) -> NodeInfo {
        NodeInfo::custom(vec![self.channels], vec![], 0)
    }

    fn process(&mut self, inputs: &[&AudioBuffer<2>], outputs: &mut [AudioBuffer<2>], frames: usize) {
        // Copy input to output buffer (which is read externally after graph processing)
        if !inputs.is_empty() && !outputs.is_empty() {
            for frame in 0..frames {
                for channel in 0..2.min(self.channels) {
                    outputs[0].set(frame, channel, inputs[0].get(frame, channel));
                }
            }
        }
    }

    fn reset(&mut self) {}

    fn name(&self) -> &'static str {
        "Output"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amdusias_core::SampleRate;

    // =========================================================================
    // Phase 4 TDD: Comprehensive I/O Node tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // InputNode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_input_node_mono() {
        let node = InputNode::new(1);
        let info = node.info();

        assert_eq!(info.input_count, 0);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.output_channels[0], 1);
        assert_eq!(info.latency_samples, 0);
    }

    #[test]
    fn test_input_node_stereo() {
        let node = InputNode::new(2);
        let info = node.info();

        assert_eq!(info.input_count, 0);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.output_channels[0], 2);
    }

    #[test]
    fn test_input_node_multichannel() {
        let node = InputNode::new(6);
        let info = node.info();

        assert_eq!(info.output_channels[0], 6);
    }

    #[test]
    fn test_input_node_name() {
        let node = InputNode::new(2);
        assert_eq!(node.name(), "Input");
    }

    #[test]
    fn test_input_node_process_is_noop() {
        let mut node = InputNode::new(2);

        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];
        outputs[0].fill(0.5);

        let empty: &[&AudioBuffer<2>] = &[];
        node.process(empty, &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.5).abs() < 0.001,
            "InputNode should not modify output: got {}",
            out
        );
    }

    #[test]
    fn test_input_node_reset() {
        let mut node = InputNode::new(2);
        node.reset();
    }

    // -------------------------------------------------------------------------
    // OutputNode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_output_node_mono() {
        let node = OutputNode::new(1);
        let info = node.info();

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 0);
        assert_eq!(info.input_channels[0], 1);
        assert_eq!(info.latency_samples, 0);
    }

    #[test]
    fn test_output_node_stereo() {
        let node = OutputNode::new(2);
        let info = node.info();

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 0);
        assert_eq!(info.input_channels[0], 2);
    }

    #[test]
    fn test_output_node_multichannel() {
        let node = OutputNode::new(6);
        let info = node.info();

        assert_eq!(info.input_channels[0], 6);
    }

    #[test]
    fn test_output_node_name() {
        let node = OutputNode::new(2);
        assert_eq!(node.name(), "Output");
    }

    #[test]
    fn test_output_node_copies_input() {
        let mut node = OutputNode::new(2);

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(0.75);

        node.process(&[&input], &mut outputs, 64);

        for frame in 0..64 {
            for channel in 0..2 {
                let out = outputs[0].get(frame, channel);
                assert!(
                    (out - 0.75).abs() < 0.001,
                    "Output should copy input: got {} at frame {}, channel {}",
                    out,
                    frame,
                    channel
                );
            }
        }
    }

    #[test]
    fn test_output_node_stereo_independence() {
        let mut node = OutputNode::new(2);

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        for frame in 0..64 {
            input.set(frame, 0, 0.3);
            input.set(frame, 1, 0.7);
        }

        node.process(&[&input], &mut outputs, 64);

        let left = outputs[0].get(32, 0);
        let right = outputs[0].get(32, 1);

        assert!((left - 0.3).abs() < 0.001, "Left channel: got {}", left);
        assert!((right - 0.7).abs() < 0.001, "Right channel: got {}", right);
    }

    #[test]
    fn test_output_node_empty_inputs() {
        let mut node = OutputNode::new(2);

        let empty: &[&AudioBuffer<2>] = &[];
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        outputs[0].fill(0.5);

        node.process(empty, &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.5).abs() < 0.001,
            "Empty input should preserve output"
        );
    }

    #[test]
    fn test_output_node_empty_outputs() {
        let mut node = OutputNode::new(2);

        let input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut empty: Vec<AudioBuffer<2>> = vec![];

        node.process(&[&input], &mut empty, 64);
    }

    #[test]
    fn test_output_node_reset() {
        let mut node = OutputNode::new(2);
        node.reset();
    }

    #[test]
    fn test_output_node_mono_limits_channels() {
        let mut node = OutputNode::new(1);

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        for frame in 0..64 {
            input.set(frame, 0, 0.5);
            input.set(frame, 1, 0.9);
        }

        outputs[0].fill(0.0);

        node.process(&[&input], &mut outputs, 64);

        let left = outputs[0].get(32, 0);
        let right = outputs[0].get(32, 1);

        assert!((left - 0.5).abs() < 0.001, "Left should be copied");
        assert!(
            right.abs() < 0.001,
            "Right should be untouched for mono: got {}",
            right
        );
    }

    #[test]
    fn test_output_node_negative_samples() {
        let mut node = OutputNode::new(2);

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(-0.5);

        node.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - (-0.5)).abs() < 0.001,
            "Negative samples should be copied: got {}",
            out
        );
    }

    // -------------------------------------------------------------------------
    // Combined I/O tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_input_output_symmetric() {
        let input_node = InputNode::new(2);
        let output_node = OutputNode::new(2);

        let input_info = input_node.info();
        let output_info = output_node.info();

        assert_eq!(input_info.input_count, 0);
        assert_eq!(input_info.output_count, 1);
        assert_eq!(output_info.input_count, 1);
        assert_eq!(output_info.output_count, 0);

        assert_eq!(input_info.output_channels[0], output_info.input_channels[0]);
    }

    #[test]
    fn test_various_channel_counts() {
        let channel_counts = [1, 2, 4, 6, 8];

        for &channels in &channel_counts {
            let input = InputNode::new(channels);
            let output = OutputNode::new(channels);

            assert_eq!(
                input.info().output_channels[0],
                channels,
                "Input channels for {}",
                channels
            );
            assert_eq!(
                output.info().input_channels[0],
                channels,
                "Output channels for {}",
                channels
            );
        }
    }
}
