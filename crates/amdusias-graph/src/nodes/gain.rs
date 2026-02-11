//! Gain node implementation.

use crate::node::{AudioNode, NodeInfo};
use amdusias_core::AudioBuffer;

/// Simple gain (volume) node.
#[derive(Debug, Clone)]
pub struct GainNode {
    /// Gain value (linear, not dB).
    gain: f32,
    /// Target gain for smoothing.
    target_gain: f32,
    /// Smoothing coefficient.
    smooth_coeff: f32,
}

impl GainNode {
    /// Creates a new gain node.
    #[must_use]
    pub fn new(gain: f32) -> Self {
        Self {
            gain,
            target_gain: gain,
            smooth_coeff: 0.999,
        }
    }

    /// Sets the gain value (linear).
    pub fn set_gain(&mut self, gain: f32) {
        self.target_gain = gain;
    }

    /// Sets the gain value in decibels.
    pub fn set_gain_db(&mut self, gain_db: f32) {
        self.target_gain = 10.0_f32.powf(gain_db / 20.0);
    }

    /// Returns the current gain value.
    #[must_use]
    pub fn gain(&self) -> f32 {
        self.gain
    }
}

impl AudioNode for GainNode {
    fn info(&self) -> NodeInfo {
        NodeInfo::stereo()
    }

    fn process(&mut self, inputs: &[&AudioBuffer<2>], outputs: &mut [AudioBuffer<2>], frames: usize) {
        if inputs.is_empty() || outputs.is_empty() {
            return;
        }

        let input = inputs[0];
        let output = &mut outputs[0];

        for frame in 0..frames {
            // Smooth gain changes
            self.gain = self.target_gain + self.smooth_coeff * (self.gain - self.target_gain);

            for channel in 0..2 {
                let sample = input.get(frame, channel);
                output.set(frame, channel, sample * self.gain);
            }
        }
    }

    fn reset(&mut self) {
        self.gain = self.target_gain;
    }

    fn name(&self) -> &'static str {
        "Gain"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amdusias_core::SampleRate;

    #[test]
    fn test_gain_node() {
        let mut node = GainNode::new(0.5);

        let mut input_mut = AudioBuffer::<2>::new(4, SampleRate::Hz48000);
        input_mut.fill(1.0);

        let mut outputs = vec![AudioBuffer::<2>::new(4, SampleRate::Hz48000)];
        node.process(&[&input_mut], &mut outputs, 4);
    }

    // =========================================================================
    // Phase 4 TDD: Comprehensive GainNode tests
    // =========================================================================

    #[test]
    fn test_gain_unity() {
        let mut node = GainNode::new(1.0);
        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(0.5);

        node.process(&[&input], &mut outputs, 64);

        // Unity gain should pass signal through unchanged
        for frame in 0..64 {
            for channel in 0..2 {
                let out = outputs[0].get(frame, channel);
                assert!(
                    (out - 0.5).abs() < 0.01,
                    "Unity gain should pass through: got {} at frame {}, channel {}",
                    out,
                    frame,
                    channel
                );
            }
        }
    }

    #[test]
    fn test_gain_half() {
        let mut node = GainNode::new(0.5);
        node.reset();

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(1.0);

        node.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(63, 0);
        assert!(
            (out - 0.5).abs() < 0.05,
            "Half gain should halve signal: got {}",
            out
        );
    }

    #[test]
    fn test_gain_zero() {
        let mut node = GainNode::new(0.0);
        node.reset();

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(1.0);

        node.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(63, 0);
        assert!(out.abs() < 0.001, "Zero gain should silence: got {}", out);
    }

    #[test]
    fn test_gain_boost() {
        let mut node = GainNode::new(2.0);
        node.reset();

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(0.3);

        node.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(63, 0);
        assert!(
            (out - 0.6).abs() < 0.05,
            "2x gain should double: got {}",
            out
        );
    }

    #[test]
    fn test_gain_db_conversion() {
        let mut node = GainNode::new(1.0);

        // -6dB ≈ 0.5 linear
        node.set_gain_db(-6.0);
        node.reset();

        let expected = 10.0_f32.powf(-6.0 / 20.0);
        assert!(
            (node.gain() - expected).abs() < 0.01,
            "-6dB should be ~0.5: got {}",
            node.gain()
        );

        // 0dB = 1.0 linear
        node.set_gain_db(0.0);
        node.reset();
        assert!(
            (node.gain() - 1.0).abs() < 0.001,
            "0dB should be 1.0: got {}",
            node.gain()
        );

        // +6dB ≈ 2.0 linear
        node.set_gain_db(6.0);
        node.reset();
        let expected_boost = 10.0_f32.powf(6.0 / 20.0);
        assert!(
            (node.gain() - expected_boost).abs() < 0.01,
            "+6dB should be ~2.0: got {}",
            node.gain()
        );
    }

    #[test]
    fn test_gain_smoothing() {
        let mut node = GainNode::new(1.0);

        let mut input = AudioBuffer::<2>::new(1, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(1, SampleRate::Hz48000)];
        input.fill(1.0);

        // Process a few samples at unity
        for _ in 0..10 {
            node.process(&[&input], &mut outputs, 1);
        }
        assert!((node.gain() - 1.0).abs() < 0.001);

        // Now change to 0.5 - should smooth gradually
        node.set_gain(0.5);

        node.process(&[&input], &mut outputs, 1);
        let first_out = outputs[0].get(0, 0);

        // Should be between old and new gain due to smoothing
        assert!(
            first_out > 0.5 && first_out < 1.0,
            "Smoothing should interpolate: got {}",
            first_out
        );
    }

    #[test]
    fn test_gain_reset() {
        let mut node = GainNode::new(1.0);

        node.set_gain(0.5);
        assert!((node.gain() - 1.0).abs() < 0.001);

        node.reset();
        assert!(
            (node.gain() - 0.5).abs() < 0.001,
            "Reset should jump to target: got {}",
            node.gain()
        );
    }

    #[test]
    fn test_gain_node_info() {
        let node = GainNode::new(1.0);
        let info = node.info();

        assert_eq!(info.input_count, 1);
        assert_eq!(info.output_count, 1);
        assert_eq!(info.input_channels[0], 2);
        assert_eq!(info.output_channels[0], 2);
        assert_eq!(info.latency_samples, 0);
    }

    #[test]
    fn test_gain_node_name() {
        let node = GainNode::new(1.0);
        assert_eq!(node.name(), "Gain");
    }

    #[test]
    fn test_gain_node_clone() {
        let node = GainNode::new(0.7);
        let cloned = node.clone();

        assert!((cloned.gain() - node.gain()).abs() < 0.001);
    }

    #[test]
    fn test_gain_empty_inputs() {
        let mut node = GainNode::new(1.0);
        let empty_inputs: &[&AudioBuffer<2>] = &[];
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        node.process(empty_inputs, &mut outputs, 64);
    }

    #[test]
    fn test_gain_empty_outputs() {
        let mut node = GainNode::new(1.0);
        let input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut empty_outputs: Vec<AudioBuffer<2>> = vec![];

        node.process(&[&input], &mut empty_outputs, 64);
    }

    #[test]
    fn test_gain_stereo_independence() {
        let mut node = GainNode::new(0.5);
        node.reset();

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        for frame in 0..64 {
            input.set(frame, 0, 1.0);
            input.set(frame, 1, 0.5);
        }

        node.process(&[&input], &mut outputs, 64);

        let left = outputs[0].get(63, 0);
        let right = outputs[0].get(63, 1);

        assert!(
            (left - 0.5).abs() < 0.05,
            "Left should be ~0.5: got {}",
            left
        );
        assert!(
            (right - 0.25).abs() < 0.03,
            "Right should be ~0.25: got {}",
            right
        );
    }

    #[test]
    fn test_gain_negative_samples() {
        let mut node = GainNode::new(0.5);
        node.reset();

        let mut input = AudioBuffer::<2>::new(64, SampleRate::Hz48000);
        let mut outputs = vec![AudioBuffer::<2>::new(64, SampleRate::Hz48000)];

        input.fill(-1.0);

        node.process(&[&input], &mut outputs, 64);

        let out = outputs[0].get(63, 0);
        assert!(
            (out - (-0.5)).abs() < 0.05,
            "Negative sample should be scaled: got {}",
            out
        );
    }
}
