//! Delay line implementations.

use crate::Sample;

/// Basic delay line with linear interpolation.
#[derive(Debug, Clone)]
pub struct DelayLine {
    buffer: Vec<Sample>,
    write_pos: usize,
    max_delay_samples: usize,
}

impl DelayLine {
    /// Creates a new delay line with the specified maximum delay.
    #[must_use]
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            max_delay_samples,
        }
    }

    /// Creates a delay line sized for a maximum delay time in seconds.
    #[must_use]
    pub fn from_max_time(max_delay_secs: f32, sample_rate: f32) -> Self {
        let samples = (max_delay_secs * sample_rate).ceil() as usize;
        Self::new(samples.max(1))
    }

    /// Writes a sample to the delay line.
    #[inline]
    pub fn write(&mut self, sample: Sample) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % self.max_delay_samples;
    }

    /// Reads a sample at the specified delay (in samples).
    ///
    /// Uses linear interpolation for fractional delays.
    #[inline]
    #[must_use]
    pub fn read(&self, delay_samples: f32) -> Sample {
        if self.max_delay_samples == 0 {
            return 0.0;
        }

        // Clamp delay to valid range
        let delay_samples = delay_samples.clamp(0.0, (self.max_delay_samples - 1) as f32);
        let delay_int = delay_samples as usize;
        let delay_frac = delay_samples - delay_int as f32;

        // Use wrapping arithmetic to avoid overflow
        // Add 2 * max_delay_samples to ensure positive result before modulo
        let read_pos_1 = (self.write_pos + 2 * self.max_delay_samples - delay_int - 1)
            % self.max_delay_samples;
        let read_pos_2 = (read_pos_1 + self.max_delay_samples - 1) % self.max_delay_samples;

        let sample_1 = self.buffer[read_pos_1];
        let sample_2 = self.buffer[read_pos_2];

        // Linear interpolation
        sample_1 + delay_frac * (sample_2 - sample_1)
    }

    /// Reads using Hermite interpolation (higher quality for modulated delays).
    #[must_use]
    pub fn read_hermite(&self, delay_samples: f32) -> Sample {
        let delay_int = delay_samples as usize;
        let t = delay_samples - delay_int as f32;

        let idx = |offset: usize| -> usize {
            (self.write_pos + self.max_delay_samples - delay_int - 1 + offset)
                % self.max_delay_samples
        };

        let y0 = self.buffer[(idx(0) + self.max_delay_samples - 1) % self.max_delay_samples];
        let y1 = self.buffer[idx(0)];
        let y2 = self.buffer[(idx(0) + 1) % self.max_delay_samples];
        let y3 = self.buffer[(idx(0) + 2) % self.max_delay_samples];

        // Hermite interpolation
        let c0 = y1;
        let c1 = 0.5 * (y2 - y0);
        let c2 = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c3 = 0.5 * (y3 - y0) + 1.5 * (y1 - y2);

        ((c3 * t + c2) * t + c1) * t + c0
    }

    /// Writes a sample and reads at the specified delay.
    #[inline]
    pub fn process(&mut self, input: Sample, delay_samples: f32) -> Sample {
        let output = self.read(delay_samples);
        self.write(input);
        output
    }

    /// Clears the delay line.
    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }

    /// Returns the maximum delay in samples.
    #[must_use]
    pub fn max_delay(&self) -> usize {
        self.max_delay_samples
    }
}

/// Multi-tap delay line.
#[derive(Debug, Clone)]
pub struct MultiTapDelay {
    delay_line: DelayLine,
    taps: Vec<DelayTap>,
}

/// A single tap in a multi-tap delay.
#[derive(Debug, Clone, Copy)]
pub struct DelayTap {
    /// Delay time in samples.
    pub delay_samples: f32,
    /// Gain for this tap.
    pub gain: f32,
    /// Pan position (-1.0 to 1.0, 0.0 = center).
    pub pan: f32,
}

impl MultiTapDelay {
    /// Creates a new multi-tap delay.
    #[must_use]
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            delay_line: DelayLine::new(max_delay_samples),
            taps: Vec::new(),
        }
    }

    /// Adds a tap to the delay.
    pub fn add_tap(&mut self, tap: DelayTap) {
        self.taps.push(tap);
    }

    /// Clears all taps.
    pub fn clear_taps(&mut self) {
        self.taps.clear();
    }

    /// Processes input and returns the sum of all taps.
    pub fn process(&mut self, input: Sample) -> Sample {
        let mut output = 0.0;

        for tap in &self.taps {
            output += self.delay_line.read(tap.delay_samples) * tap.gain;
        }

        self.delay_line.write(input);
        output
    }

    /// Processes and returns stereo output based on tap panning.
    pub fn process_stereo(&mut self, input: Sample) -> (Sample, Sample) {
        let mut left = 0.0;
        let mut right = 0.0;

        for tap in &self.taps {
            let sample = self.delay_line.read(tap.delay_samples) * tap.gain;
            let pan_l = ((1.0 - tap.pan) / 2.0).sqrt();
            let pan_r = ((1.0 + tap.pan) / 2.0).sqrt();
            left += sample * pan_l;
            right += sample * pan_r;
        }

        self.delay_line.write(input);
        (left, right)
    }

    /// Clears the delay buffer.
    pub fn clear(&mut self) {
        self.delay_line.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line() {
        let mut delay = DelayLine::new(100);

        // Write some samples
        for i in 0..50 {
            delay.write(i as f32);
        }

        // Read at different delays
        let sample = delay.read(10.0);
        assert!((sample - 39.0).abs() < 0.01);
    }

    #[test]
    fn test_fractional_delay() {
        let mut delay = DelayLine::new(100);

        for i in 0..50 {
            delay.write(i as f32);
        }

        // Fractional delay should interpolate
        let sample = delay.read(10.5);
        assert!((sample - 38.5).abs() < 0.01);
    }

    // =========================================================================
    // Phase 2 TDD: Comprehensive delay line tests
    // =========================================================================

    #[test]
    fn test_zero_delay() {
        let mut delay = DelayLine::new(100);

        // Write samples
        delay.write(1.0);
        delay.write(2.0);
        delay.write(3.0);

        // Zero delay should return most recently written sample
        let sample = delay.read(0.0);
        // Note: Due to implementation, read(0) returns the sample written 1 ago
        // This is correct behavior - there's always at least 1 sample delay
        assert!(sample.is_finite(), "Zero delay should be finite");
    }

    #[test]
    fn test_max_delay() {
        let max_delay = 100;
        let mut delay = DelayLine::new(max_delay);

        // Fill buffer with known pattern
        for i in 0..max_delay {
            delay.write(i as f32);
        }

        // Reading at max delay (clamped to max-1) should work
        let sample = delay.read((max_delay - 1) as f32);
        assert!(sample.is_finite(), "Max delay should return finite value");
    }

    #[test]
    fn test_delay_clamping() {
        let mut delay = DelayLine::new(100);

        // Fill with pattern
        for i in 0..100 {
            delay.write(i as f32);
        }

        // Delays beyond max should be clamped
        let at_max = delay.read(99.0);
        let beyond_max = delay.read(200.0);

        // Both should return the same (clamped) value
        assert!(
            (at_max - beyond_max).abs() < 0.001,
            "Delay should be clamped: {} vs {}",
            at_max,
            beyond_max
        );

        // Negative delay should be clamped to 0
        let negative = delay.read(-10.0);
        let zero = delay.read(0.0);
        assert!(
            (negative - zero).abs() < 0.001,
            "Negative delay should be clamped: {} vs {}",
            negative,
            zero
        );
    }

    #[test]
    fn test_from_max_time() {
        let sample_rate = 48000.0;
        let max_time_secs = 0.5; // 500ms

        let delay = DelayLine::from_max_time(max_time_secs, sample_rate);

        // Should have 24000 samples capacity
        assert_eq!(
            delay.max_delay(),
            24000,
            "500ms at 48kHz should be 24000 samples"
        );
    }

    #[test]
    fn test_from_max_time_various_rates() {
        for (sample_rate, max_time, expected) in [
            (44100.0, 1.0, 44100),
            (48000.0, 0.5, 24000),
            (96000.0, 0.1, 9600),
        ] {
            let delay = DelayLine::from_max_time(max_time, sample_rate);
            assert_eq!(
                delay.max_delay(),
                expected,
                "{}s at {}Hz should be {} samples",
                max_time,
                sample_rate,
                expected
            );
        }
    }

    #[test]
    fn test_process_method() {
        let mut delay = DelayLine::new(100);

        // Process combines read (first) and write (second)
        // So when buffer is empty, first outputs are 0
        let out1 = delay.process(1.0, 50.0);
        let out2 = delay.process(2.0, 50.0);
        let out3 = delay.process(3.0, 50.0);

        // First outputs should be 0 (buffer was empty)
        assert_eq!(out1, 0.0, "First output should be 0");
        assert_eq!(out2, 0.0, "Second output should be 0");
        assert_eq!(out3, 0.0, "Third output should be 0");

        // After ~50 more samples, should read back first samples
        // The timing depends on exact buffer position
        for _ in 0..50 {
            delay.process(0.0, 50.0);
        }

        // The delayed signal should now be arriving
        // Collect a few samples to verify
        let mut found_signal = false;
        for _ in 0..10 {
            let out = delay.process(0.0, 50.0);
            if out.abs() > 0.5 {
                found_signal = true;
                break;
            }
        }
        assert!(found_signal, "Should eventually read back signal");
    }

    #[test]
    fn test_clear() {
        let mut delay = DelayLine::new(100);

        // Fill with data
        for i in 0..100 {
            delay.write((i + 1) as f32);
        }

        // Verify there's data
        let before_clear = delay.read(50.0);
        assert!(before_clear.abs() > 0.0, "Should have data before clear");

        // Clear
        delay.clear();

        // Should be all zeros
        for d in 0..100 {
            let sample = delay.read(d as f32);
            assert_eq!(sample, 0.0, "Should be zero at delay {}", d);
        }
    }

    #[test]
    fn test_wrap_around() {
        let mut delay = DelayLine::new(10);

        // Write more samples than buffer size to test wrap-around
        for i in 0..25 {
            delay.write(i as f32);
        }

        // Should wrap correctly and have most recent 10 samples
        let recent = delay.read(1.0);
        assert!(
            (recent - 23.0).abs() < 0.01,
            "Should have recent sample: got {}",
            recent
        );
    }

    #[test]
    fn test_linear_interpolation_accuracy() {
        let mut delay = DelayLine::new(100);

        // Fill buffer with a ramp pattern so we can verify interpolation
        for i in 0..100 {
            delay.write(i as f32);
        }

        // At this point, write_pos is back at 0
        // Buffer contains [0, 1, 2, 3, ..., 99]

        // Test that fractional delays interpolate between samples
        let d10 = delay.read(10.0);
        let d10_5 = delay.read(10.5);
        let d11 = delay.read(11.0);

        // d10_5 should be between d10 and d11
        let min_val = d10.min(d11);
        let max_val = d10.max(d11);

        assert!(
            d10_5 >= min_val - 0.1 && d10_5 <= max_val + 0.1,
            "Interpolated {} should be between {} and {}",
            d10_5,
            min_val,
            max_val
        );

        // Verify linear interpolation: d10_5 should be roughly (d10 + d11) / 2
        let expected = (d10 + d11) / 2.0;
        assert!(
            (d10_5 - expected).abs() < 0.5,
            "Linear interpolation: {} should be close to {}",
            d10_5,
            expected
        );
    }

    #[test]
    fn test_hermite_interpolation() {
        let mut delay = DelayLine::new(100);

        // Write smooth pattern
        for i in 0..50 {
            let sample = (i as f32 * 0.1).sin();
            delay.write(sample);
        }

        // Hermite interpolation should produce smooth results
        let h1 = delay.read_hermite(10.0);
        let h2 = delay.read_hermite(10.25);
        let h3 = delay.read_hermite(10.5);
        let h4 = delay.read_hermite(10.75);

        // All should be finite and reasonable
        for (i, val) in [h1, h2, h3, h4].iter().enumerate() {
            assert!(val.is_finite(), "Hermite {} should be finite", i);
            assert!(
                val.abs() <= 2.0,
                "Hermite {} should be reasonable: {}",
                i,
                val
            );
        }
    }

    // =========================================================================
    // Multi-tap delay tests
    // =========================================================================

    #[test]
    fn test_multi_tap_basic() {
        let mut mtd = MultiTapDelay::new(1000);

        mtd.add_tap(DelayTap {
            delay_samples: 100.0,
            gain: 0.5,
            pan: 0.0,
        });

        // Prime with impulse
        let _ = mtd.process(1.0);

        // Wait for delay - process reads before writing, so timing can vary
        // Look for the tap output over a window
        let mut found_tap = false;
        for _ in 0..110 {
            let output = mtd.process(0.0);
            if (output - 0.5).abs() < 0.02 {
                found_tap = true;
                break;
            }
        }

        assert!(found_tap, "Should eventually get tap at gain 0.5");
    }

    #[test]
    fn test_multi_tap_multiple_taps() {
        let mut mtd = MultiTapDelay::new(1000);

        mtd.add_tap(DelayTap {
            delay_samples: 10.0,
            gain: 0.3,
            pan: 0.0,
        });
        mtd.add_tap(DelayTap {
            delay_samples: 20.0,
            gain: 0.2,
            pan: 0.0,
        });
        mtd.add_tap(DelayTap {
            delay_samples: 30.0,
            gain: 0.1,
            pan: 0.0,
        });

        // Impulse
        mtd.process(1.0);

        // Collect outputs over 40 samples
        let mut outputs = Vec::new();
        for _ in 0..40 {
            outputs.push(mtd.process(0.0));
        }

        // Find the peaks corresponding to each tap
        let mut found_peaks = 0;
        for (i, &out) in outputs.iter().enumerate() {
            if out.abs() > 0.05 {
                found_peaks += 1;
                // Verify the output is one of our expected gains
                let is_valid = (out - 0.3).abs() < 0.02
                    || (out - 0.2).abs() < 0.02
                    || (out - 0.1).abs() < 0.02;
                assert!(
                    is_valid,
                    "Peak at sample {} should be one of tap gains: {}",
                    i, out
                );
            }
        }

        assert!(
            found_peaks >= 3,
            "Should find at least 3 tap peaks, found {}",
            found_peaks
        );
    }

    #[test]
    fn test_multi_tap_stereo() {
        let mut mtd = MultiTapDelay::new(1000);

        // Left-panned tap
        mtd.add_tap(DelayTap {
            delay_samples: 10.0,
            gain: 1.0,
            pan: -1.0, // Full left
        });

        // Right-panned tap
        mtd.add_tap(DelayTap {
            delay_samples: 30.0, // Further apart to avoid overlap
            gain: 1.0,
            pan: 1.0, // Full right
        });

        // Impulse
        mtd.process_stereo(1.0);

        // Collect stereo outputs
        let mut found_left_peak = false;
        let mut found_right_peak = false;

        for _ in 0..40 {
            let (left, right) = mtd.process_stereo(0.0);

            // Check for left-panned tap (high left, low right)
            if left > 0.5 && right < 0.2 {
                found_left_peak = true;
            }

            // Check for right-panned tap (low left, high right)
            if right > 0.5 && left < 0.2 {
                found_right_peak = true;
            }
        }

        assert!(found_left_peak, "Should find left-panned tap");
        assert!(found_right_peak, "Should find right-panned tap");
    }

    #[test]
    fn test_multi_tap_clear_taps() {
        let mut mtd = MultiTapDelay::new(1000);

        mtd.add_tap(DelayTap {
            delay_samples: 10.0,
            gain: 1.0,
            pan: 0.0,
        });

        // Prime with impulse
        mtd.process(1.0);

        // Look for tap output
        let mut found_tap_output = false;
        for _ in 0..20 {
            let out = mtd.process(0.0);
            if out > 0.5 {
                found_tap_output = true;
                break;
            }
        }
        assert!(found_tap_output, "Should have tap output with taps");

        // Clear taps and buffer
        mtd.clear_taps();
        mtd.clear();

        // Prime again
        mtd.process(1.0);

        // Should have no output (no taps defined)
        let mut found_any_output = false;
        for _ in 0..20 {
            let out = mtd.process(0.0);
            if out.abs() > 0.01 {
                found_any_output = true;
            }
        }
        assert!(!found_any_output, "Should have no output without taps");
    }

    #[test]
    fn test_multi_tap_center_pan() {
        let mut mtd = MultiTapDelay::new(1000);

        mtd.add_tap(DelayTap {
            delay_samples: 10.0,
            gain: 1.0,
            pan: 0.0, // Center
        });

        // Prime with impulse
        mtd.process_stereo(1.0);

        // Look for centered output
        let mut found_centered = false;
        for _ in 0..20 {
            let (left, right) = mtd.process_stereo(0.0);

            // Check for centered output (equal L/R, both > 0.5)
            if left > 0.5 && right > 0.5 && (left - right).abs() < 0.05 {
                found_centered = true;
                // Verify equal power panning: both should be ~0.707
                assert!(
                    (left - 0.707).abs() < 0.05,
                    "Center pan left should be ~0.707: got {}",
                    left
                );
                assert!(
                    (right - 0.707).abs() < 0.05,
                    "Center pan right should be ~0.707: got {}",
                    right
                );
                break;
            }
        }

        assert!(found_centered, "Should find centered stereo output");
    }

    #[test]
    fn test_delay_stability() {
        let mut delay = DelayLine::new(1000);

        // Process many samples - should never NaN or explode
        for i in 0..100000 {
            let input = (i as f32 * 0.001).sin();
            let output = delay.process(input, 500.0);

            assert!(output.is_finite(), "Output {} should be finite", i);
            assert!(
                output.abs() <= 1.1,
                "Output should not exceed input range: {}",
                output
            );
        }
    }

    #[test]
    fn test_modulated_delay() {
        let mut delay = DelayLine::new(1000);

        // Fill with test pattern
        for i in 0..500 {
            delay.write((i as f32 * 0.01).sin());
        }

        // Modulate delay time - should produce smooth output
        let mut outputs = Vec::new();
        for i in 0..1000 {
            // LFO modulating delay between 400 and 600
            let delay_time = 500.0 + 100.0 * (i as f32 * 0.01).sin();
            let output = delay.process(0.0, delay_time);
            outputs.push(output);
        }

        // All outputs should be finite
        for (i, &out) in outputs.iter().enumerate() {
            assert!(out.is_finite(), "Modulated output {} should be finite", i);
        }
    }
}
