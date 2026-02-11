//! Mixer node implementation.

use crate::node::{AudioNode, NodeInfo};
use amdusias_core::AudioBuffer;

/// Multi-input mixer node.
#[derive(Debug)]
pub struct MixerNode {
    /// Number of input channels.
    input_count: usize,
    /// Per-input gains.
    gains: Vec<f32>,
}

impl MixerNode {
    /// Creates a new mixer with the specified number of inputs.
    #[must_use]
    pub fn new(input_count: usize) -> Self {
        Self {
            input_count,
            gains: vec![1.0; input_count],
        }
    }

    /// Sets the gain for a specific input.
    pub fn set_input_gain(&mut self, input: usize, gain: f32) {
        if input < self.gains.len() {
            self.gains[input] = gain;
        }
    }
}

impl AudioNode for MixerNode {
    fn info(&self) -> NodeInfo {
        NodeInfo::custom(
            vec![2; self.input_count], // Each input is stereo
            vec![2],                    // One stereo output
            0,
        )
    }

    fn process(&mut self, inputs: &[&AudioBuffer<2>], outputs: &mut [AudioBuffer<2>], frames: usize) {
        if outputs.is_empty() {
            return;
        }

        let output = &mut outputs[0];
        output.clear();

        for (idx, &input) in inputs.iter().enumerate() {
            let gain = self.gains.get(idx).copied().unwrap_or(1.0);

            for frame in 0..frames {
                for channel in 0..2 {
                    let current = output.get(frame, channel);
                    let new = current + input.get(frame, channel) * gain;
                    output.set(frame, channel, new);
                }
            }
        }
    }

    fn reset(&mut self) {}

    fn name(&self) -> &'static str {
        "Mixer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amdusias_core::SampleRate;

    // =========================================================================
    // Phase 4 TDD: Comprehensive MixerNode tests
    // =========================================================================

    #[test]
    fn test_mixer_single_input() {
        let mut mixer = MixerNode::new(1);

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(0.5);

        mixer.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.5).abs() < 0.001,
            "Single input should pass through: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_two_inputs_sum() {
        let mut mixer = MixerNode::new(2);

        let mut input1 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut input2 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input1.fill(0.3);
        input2.fill(0.2);

        mixer.process(&[&input1, &input2], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.5).abs() < 0.001,
            "Two inputs should sum: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_four_inputs() {
        let mut mixer = MixerNode::new(4);

        let mut inputs: Vec<AudioBuffer<2>> = (0..4)
            .map(|_| {
                let mut buf = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
                buf.fill(0.1);
                buf
            })
            .collect();

        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        let input_refs: Vec<&AudioBuffer<2>> = inputs.iter().collect();
        mixer.process(&input_refs, &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.4).abs() < 0.001,
            "Four inputs should sum: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_per_input_gain() {
        let mut mixer = MixerNode::new(2);
        mixer.set_input_gain(0, 0.5);
        mixer.set_input_gain(1, 2.0);

        let mut input1 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut input2 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input1.fill(1.0);
        input2.fill(0.5);

        mixer.process(&[&input1, &input2], &mut outputs, 64);

        // Expected: 1.0 * 0.5 + 0.5 * 2.0 = 1.5
        let out = outputs[0].get(32, 0);
        assert!(
            (out - 1.5).abs() < 0.001,
            "Per-input gain should apply: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_zero_gain_mutes() {
        let mut mixer = MixerNode::new(2);
        mixer.set_input_gain(0, 0.0);
        mixer.set_input_gain(1, 1.0);

        let mut input1 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut input2 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input1.fill(1.0);
        input2.fill(0.3);

        mixer.process(&[&input1, &input2], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.3).abs() < 0.001,
            "Muted input should not contribute: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_empty_inputs() {
        let mut mixer = MixerNode::new(2);
        let empty_inputs: &[&AudioBuffer<2>] = &[];
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        outputs[0].fill(1.0);

        mixer.process(empty_inputs, &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(out.abs() < 0.001, "Empty inputs should produce silence");
    }

    #[test]
    fn test_mixer_empty_outputs() {
        let mut mixer = MixerNode::new(2);
        let input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut empty_outputs: Vec<AudioBuffer<2>> = vec![];

        mixer.process(&[&input], &mut empty_outputs, 64);
    }

    #[test]
    fn test_mixer_stereo_independence() {
        let mut mixer = MixerNode::new(2);

        let mut input1 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut input2 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        for frame in 0..64 {
            input1.set(frame, 0, 1.0);
            input1.set(frame, 1, 0.0);
        }

        for frame in 0..64 {
            input2.set(frame, 0, 0.0);
            input2.set(frame, 1, 1.0);
        }

        mixer.process(&[&input1, &input2], &mut outputs, 64);

        let left = outputs[0].get(32, 0);
        let right = outputs[0].get(32, 1);

        assert!((left - 1.0).abs() < 0.001, "Left channel: got {}", left);
        assert!((right - 1.0).abs() < 0.001, "Right channel: got {}", right);
    }

    #[test]
    fn test_mixer_clears_output() {
        let mut mixer = MixerNode::new(2);

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        outputs[0].fill(999.0);
        input.fill(0.5);

        mixer.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.5).abs() < 0.001,
            "Mixer should clear output first: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_node_info() {
        let mixer = MixerNode::new(4);
        let info = mixer.info();

        assert_eq!(info.input_count, 4);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels.len(), 4);
        assert_eq!(info.output_channels.len(), 1);

        for &ch in &info.input_channels {
            assert_eq!(ch, 2);
        }
        assert_eq!(info.output_channels[0], 2);
        assert_eq!(info.latency_samples, 0);
    }

    #[test]
    fn test_mixer_node_name() {
        let mixer = MixerNode::new(2);
        assert_eq!(mixer.name(), "Mixer");
    }

    #[test]
    fn test_mixer_set_gain_out_of_bounds() {
        let mut mixer = MixerNode::new(2);

        mixer.set_input_gain(99, 0.5);

        let mut input1 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut input2 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input1.fill(0.5);
        input2.fill(0.5);

        mixer.process(&[&input1, &input2], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!((out - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_mixer_reset() {
        let mut mixer = MixerNode::new(2);
        mixer.set_input_gain(0, 0.5);
        mixer.reset();
    }

    #[test]
    fn test_mixer_negative_samples() {
        let mut mixer = MixerNode::new(2);

        let mut input1 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut input2 = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input1.fill(-0.5);
        input2.fill(0.3);

        mixer.process(&[&input1, &input2], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - (-0.2)).abs() < 0.001,
            "Negative samples should mix: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_fewer_inputs_than_configured() {
        let mut mixer = MixerNode::new(4);

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(0.5);

        mixer.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.5).abs() < 0.001,
            "Should work with fewer inputs: got {}",
            out
        );
    }

    #[test]
    fn test_mixer_more_inputs_than_configured() {
        let mut mixer = MixerNode::new(2);

        let inputs: Vec<AudioBuffer<2>> = (0..4)
            .map(|_| {
                let mut buf = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
                buf.fill(0.1);
                buf
            })
            .collect();

        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        let input_refs: Vec<&AudioBuffer<2>> = inputs.iter().collect();
        mixer.process(&input_refs, &mut outputs, 64);

        let out = outputs[0].get(32, 0);
        assert!(
            (out - 0.4).abs() < 0.001,
            "Extra inputs should use default gain: got {}",
            out
        );
    }
}
