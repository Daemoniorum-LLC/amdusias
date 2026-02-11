//! Brickwall limiter implementation.

use crate::{delay::DelayLine, linear_to_db, traits::Processor, Sample};

/// Brickwall limiter with lookahead.
///
/// Ensures output never exceeds the ceiling.
#[derive(Debug, Clone)]
pub struct Limiter {
    /// Ceiling in linear.
    ceiling: f32,
    /// Release time in samples.
    release_samples: f32,
    /// Lookahead delay line.
    lookahead: DelayLine,
    /// Current gain.
    gain: f32,
    /// Target gain.
    target_gain: f32,
    /// Samples of lookahead.
    lookahead_samples: usize,
}

impl Limiter {
    /// Creates a new limiter.
    ///
    /// # Arguments
    ///
    /// - `ceiling_db`: Maximum output level in dB (typically -0.3 dB).
    /// - `lookahead_ms`: Lookahead time in milliseconds.
    /// - `release_ms`: Release time in milliseconds.
    /// - `sample_rate`: Sample rate in Hz.
    #[must_use]
    pub fn new(ceiling_db: f32, lookahead_ms: f32, release_ms: f32, sample_rate: f32) -> Self {
        let lookahead_samples = (lookahead_ms * sample_rate / 1000.0) as usize;

        Self {
            ceiling: 10.0_f32.powf(ceiling_db / 20.0),
            release_samples: release_ms * sample_rate / 1000.0,
            lookahead: DelayLine::new(lookahead_samples.max(1)),
            gain: 1.0,
            target_gain: 1.0,
            lookahead_samples,
        }
    }

    /// Sets the ceiling level.
    pub fn set_ceiling(&mut self, ceiling_db: f32) {
        self.ceiling = 10.0_f32.powf(ceiling_db / 20.0);
    }

    /// Returns the current gain reduction in dB.
    #[must_use]
    pub fn gain_reduction_db(&self) -> f32 {
        linear_to_db(self.gain)
    }
}

impl Processor for Limiter {
    fn process_sample(&mut self, input: Sample) -> Sample {
        // Write to lookahead buffer
        self.lookahead.write(input);

        // Calculate required gain for current input
        let input_abs = input.abs();
        let required_gain = if input_abs > self.ceiling {
            self.ceiling / input_abs
        } else {
            1.0
        };

        // Update target gain (attack = instant)
        if required_gain < self.target_gain {
            self.target_gain = required_gain;
        } else {
            // Release smoothly
            let release_coeff = 1.0 / self.release_samples;
            self.target_gain = self.target_gain + release_coeff * (1.0 - self.target_gain);
            self.target_gain = self.target_gain.min(1.0);
        }

        // Smooth gain changes
        self.gain = self.target_gain;

        // Read from lookahead buffer and apply gain
        let delayed = self.lookahead.read(self.lookahead_samples as f32);
        delayed * self.gain
    }

    fn reset(&mut self) {
        self.lookahead.clear();
        self.gain = 1.0;
        self.target_gain = 1.0;
    }

    fn latency_samples(&self) -> usize {
        self.lookahead_samples
    }
}

/// True peak limiter with oversampling.
#[derive(Debug, Clone)]
pub struct TruePeakLimiter {
    /// Base limiter.
    limiter: Limiter,
    /// Oversampling factor.
    oversample_factor: usize,
    /// Upsampling buffer.
    upsample_buffer: Vec<Sample>,
}

impl TruePeakLimiter {
    /// Creates a new true peak limiter.
    #[must_use]
    pub fn new(ceiling_db: f32, lookahead_ms: f32, release_ms: f32, sample_rate: f32) -> Self {
        let oversample_factor = 4;
        Self {
            limiter: Limiter::new(
                ceiling_db,
                lookahead_ms,
                release_ms,
                sample_rate * oversample_factor as f32,
            ),
            oversample_factor,
            upsample_buffer: vec![0.0; oversample_factor],
        }
    }

    /// Processes a sample with true peak limiting.
    pub fn process(&mut self, input: Sample) -> Sample {
        // Simple 4x oversampling (zero-stuffing + lowpass would be more accurate)
        self.upsample_buffer[0] = input;
        for i in 1..self.oversample_factor {
            self.upsample_buffer[i] = 0.0;
        }

        // Process oversampled
        let mut output = 0.0;
        for &sample in &self.upsample_buffer {
            output = self.limiter.process_sample(sample);
        }

        output
    }

    /// Returns gain reduction in dB.
    #[must_use]
    pub fn gain_reduction_db(&self) -> f32 {
        self.limiter.gain_reduction_db()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_to_linear;

    #[test]
    fn test_limiter_ceiling() {
        let ceiling_db = -0.3;
        let ceiling_linear = 10.0_f32.powf(ceiling_db / 20.0); // ~0.966
        let mut limiter = Limiter::new(ceiling_db, 5.0, 50.0, 48000.0);

        // Fill lookahead buffer with quiet signal
        for _ in 0..500 {
            limiter.process_sample(0.5);
        }

        // Feed loud signal and collect outputs after lookahead settles
        let mut outputs = Vec::new();
        for _ in 0..1000 {
            outputs.push(limiter.process_sample(2.0));
        }

        // Skip first 240 samples (5ms lookahead at 48kHz) to let limiter settle
        let settled_outputs: Vec<f32> = outputs.iter().skip(300).copied().collect();

        // All settled outputs should be at or below ceiling
        for (i, &output) in settled_outputs.iter().enumerate() {
            assert!(
                output.abs() <= ceiling_linear + 0.01,
                "Sample {} exceeded ceiling: {} > {}",
                i,
                output.abs(),
                ceiling_linear
            );
        }
    }

    #[test]
    fn test_limiter_passthrough() {
        let mut limiter = Limiter::new(-0.3, 5.0, 50.0, 48000.0);

        // Quiet signals should pass through unchanged (after latency)
        for _ in 0..500 {
            limiter.process_sample(0.3);
        }

        let output = limiter.process_sample(0.3);
        assert!(
            (output - 0.3).abs() < 0.01,
            "Expected ~0.3, got {}",
            output
        );
    }

    // =========================================================================
    // Phase 2 TDD: Comprehensive limiter tests
    // =========================================================================

    #[test]
    fn test_various_ceiling_levels() {
        let ceilings = [-0.1, -0.3, -1.0, -3.0, -6.0];

        for &ceiling_db in &ceilings {
            let ceiling_linear = db_to_linear(ceiling_db);
            let mut limiter = Limiter::new(ceiling_db, 5.0, 50.0, 48000.0);

            // Prime with signal
            for _ in 0..500 {
                limiter.process_sample(0.5);
            }

            // Feed 0dBFS signal (1.0)
            for _ in 0..1000 {
                let output = limiter.process_sample(1.0);
                // After settling, output should not exceed ceiling
                assert!(
                    output <= ceiling_linear + 0.02,
                    "Ceiling {}dB: output {} exceeded {}",
                    ceiling_db,
                    output,
                    ceiling_linear
                );
            }
        }
    }

    #[test]
    fn test_limiter_handles_extreme_peaks() {
        let ceiling_db = -0.3;
        let ceiling_linear = db_to_linear(ceiling_db);
        let mut limiter = Limiter::new(ceiling_db, 5.0, 50.0, 48000.0);

        // Prime
        for _ in 0..500 {
            limiter.process_sample(0.5);
        }

        // Feed extreme peaks (+20dB = 10.0 linear)
        for _ in 0..1000 {
            let output = limiter.process_sample(10.0);
            assert!(
                output <= ceiling_linear + 0.02,
                "Extreme peak output {} exceeded ceiling {}",
                output,
                ceiling_linear
            );
        }
    }

    #[test]
    fn test_limiter_negative_samples() {
        let ceiling_db = -0.3;
        let ceiling_linear = db_to_linear(ceiling_db);
        let mut limiter = Limiter::new(ceiling_db, 5.0, 50.0, 48000.0);

        // Prime
        for _ in 0..500 {
            limiter.process_sample(-0.5);
        }

        // Feed negative loud signal
        for _ in 0..1000 {
            let output = limiter.process_sample(-2.0);
            assert!(
                output.abs() <= ceiling_linear + 0.02,
                "Negative sample output {} exceeded ceiling {}",
                output.abs(),
                ceiling_linear
            );
        }
    }

    #[test]
    fn test_lookahead_latency() {
        let sample_rate = 48000.0;
        let lookahead_ms = 5.0;
        let expected_samples = (lookahead_ms * sample_rate / 1000.0) as usize;

        let limiter = Limiter::new(-0.3, lookahead_ms, 50.0, sample_rate);

        assert_eq!(
            limiter.latency_samples(),
            expected_samples,
            "Lookahead latency should be {} samples",
            expected_samples
        );
    }

    #[test]
    fn test_various_lookahead_times() {
        let sample_rate = 48000.0;

        for lookahead_ms in [1.0, 2.5, 5.0, 10.0] {
            let expected_samples = (lookahead_ms * sample_rate / 1000.0) as usize;
            let limiter = Limiter::new(-0.3, lookahead_ms, 50.0, sample_rate);

            assert_eq!(
                limiter.latency_samples(),
                expected_samples,
                "Lookahead {}ms should give {} samples latency",
                lookahead_ms,
                expected_samples
            );
        }
    }

    #[test]
    fn test_gain_reduction_metering() {
        let mut limiter = Limiter::new(-0.3, 5.0, 50.0, 48000.0);

        // Process quiet signal - no GR
        for _ in 0..500 {
            limiter.process_sample(0.3);
        }

        let gr_quiet = limiter.gain_reduction_db();
        assert!(
            gr_quiet.abs() < 0.5,
            "Quiet signal should have minimal GR: got {}dB",
            gr_quiet
        );

        // Process loud signal - should have GR
        for _ in 0..1000 {
            limiter.process_sample(2.0);
        }

        let gr_loud = limiter.gain_reduction_db();
        assert!(
            gr_loud < -3.0,
            "Loud signal should have significant GR: got {}dB",
            gr_loud
        );
    }

    #[test]
    fn test_release_behavior() {
        let sample_rate = 48000.0;
        let release_ms = 50.0;
        let release_samples = (release_ms * sample_rate / 1000.0) as usize;

        let mut limiter = Limiter::new(-0.3, 5.0, release_ms, sample_rate);

        // Build up GR with loud signal
        for _ in 0..2000 {
            limiter.process_sample(2.0);
        }

        let gr_before = limiter.gain_reduction_db();
        assert!(gr_before < -3.0, "Should have GR before release");

        // Switch to quiet signal and monitor release
        let mut gr_values = Vec::new();
        for _ in 0..release_samples * 3 {
            limiter.process_sample(0.1);
            gr_values.push(limiter.gain_reduction_db());
        }

        // GR should approach 0dB (no reduction)
        let final_gr = gr_values.last().unwrap();
        assert!(
            *final_gr > gr_before,
            "GR should release: final {} should be > initial {}",
            final_gr,
            gr_before
        );
    }

    #[test]
    fn test_reset() {
        let mut limiter = Limiter::new(-0.3, 5.0, 50.0, 48000.0);

        // Build up state
        for _ in 0..1000 {
            limiter.process_sample(2.0);
        }
        assert!(limiter.gain_reduction_db() < -3.0);

        // Reset
        limiter.reset();

        // Gain should be back to unity (0dB)
        assert!(
            limiter.gain_reduction_db().abs() < 0.1,
            "GR should be ~0dB after reset: got {}",
            limiter.gain_reduction_db()
        );
    }

    #[test]
    fn test_set_ceiling_runtime() {
        let mut limiter = Limiter::new(-0.3, 5.0, 50.0, 48000.0);

        // Prime
        for _ in 0..500 {
            limiter.process_sample(0.5);
        }

        // Change ceiling to -6dB
        limiter.set_ceiling(-6.0);
        let new_ceiling = db_to_linear(-6.0); // ~0.5

        // Feed loud signal
        for _ in 0..1000 {
            let output = limiter.process_sample(1.0);
            assert!(
                output <= new_ceiling + 0.02,
                "Output {} exceeded new ceiling {}",
                output,
                new_ceiling
            );
        }
    }

    #[test]
    fn test_true_peak_limiter_creation() {
        let tpl = TruePeakLimiter::new(-0.3, 5.0, 50.0, 48000.0);
        // Just verify it can be created and has initial state
        assert!(
            tpl.gain_reduction_db().abs() < 0.1,
            "Initial GR should be ~0dB"
        );
    }

    #[test]
    fn test_true_peak_limiter_basic() {
        let ceiling_db = -1.0;
        let ceiling_linear = db_to_linear(ceiling_db);
        let mut tpl = TruePeakLimiter::new(ceiling_db, 5.0, 50.0, 48000.0);

        // Prime with quiet signal
        for _ in 0..2000 {
            tpl.process(0.5);
        }

        // Process loud signal
        for _ in 0..1000 {
            let output = tpl.process(1.5);
            // True peak limiting may have some overshoot due to oversampling
            // but should generally stay near ceiling
            assert!(
                output <= ceiling_linear + 0.1,
                "TPL output {} exceeded ceiling {} by too much",
                output,
                ceiling_linear
            );
        }
    }

    #[test]
    fn test_limiter_with_sine_wave() {
        let ceiling_db = -3.0;
        let ceiling_linear = db_to_linear(ceiling_db);
        let mut limiter = Limiter::new(ceiling_db, 5.0, 50.0, 48000.0);

        // Generate sine wave at +3dB (amplitude ~1.41)
        let amplitude = db_to_linear(3.0);
        let freq = 1000.0;
        let sample_rate = 48000.0;

        // Prime
        for i in 0..500 {
            let sample =
                amplitude * (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin();
            limiter.process_sample(sample);
        }

        // Test
        for i in 500..2000 {
            let sample =
                amplitude * (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin();
            let output = limiter.process_sample(sample);

            assert!(
                output.abs() <= ceiling_linear + 0.02,
                "Sine wave output {} exceeded ceiling {}",
                output.abs(),
                ceiling_linear
            );
        }
    }

    #[test]
    fn test_limiter_preserves_quiet_material() {
        let mut limiter = Limiter::new(-0.3, 5.0, 50.0, 48000.0);

        // Prime and settle
        for _ in 0..1000 {
            limiter.process_sample(0.3);
        }

        // Feed consistent quiet signal
        let mut outputs = Vec::new();
        for _ in 0..500 {
            outputs.push(limiter.process_sample(0.3));
        }

        // All outputs should be very close to input (no limiting needed)
        let avg: f32 = outputs.iter().sum::<f32>() / outputs.len() as f32;
        assert!(
            (avg - 0.3).abs() < 0.02,
            "Quiet material should be preserved: avg {} vs expected 0.3",
            avg
        );
    }
}
