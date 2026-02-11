//! Common traits for DSP processors.

use crate::Sample;

/// Trait for audio processors that can process samples.
pub trait Processor: Send {
    /// Processes a single sample and returns the output.
    fn process_sample(&mut self, input: Sample) -> Sample;

    /// Processes a block of samples in-place.
    fn process_block(&mut self, samples: &mut [Sample]) {
        for sample in samples.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }

    /// Resets the processor state (clears delay lines, etc.).
    fn reset(&mut self);

    /// Returns the latency introduced by this processor in samples.
    fn latency_samples(&self) -> usize {
        0
    }
}

/// Trait for processors with stereo input/output.
pub trait StereoProcessor: Send {
    /// Processes a stereo sample pair.
    fn process_stereo(&mut self, left: Sample, right: Sample) -> (Sample, Sample);

    /// Processes interleaved stereo samples in-place.
    fn process_block_stereo(&mut self, samples: &mut [Sample]) {
        for chunk in samples.chunks_exact_mut(2) {
            let (l, r) = self.process_stereo(chunk[0], chunk[1]);
            chunk[0] = l;
            chunk[1] = r;
        }
    }

    /// Resets the processor state.
    fn reset(&mut self);
}

/// Trait for processors with parameters.
pub trait Parameterized {
    /// Parameter identifier type.
    type ParamId;

    /// Gets a parameter value.
    fn get_param(&self, id: Self::ParamId) -> f32;

    /// Sets a parameter value.
    fn set_param(&mut self, id: Self::ParamId, value: f32);

    /// Gets the parameter range.
    fn param_range(&self, id: Self::ParamId) -> (f32, f32);
}

/// Smoothed parameter for click-free automation.
pub struct SmoothedParam {
    current: f32,
    target: f32,
    coeff: f32,
}

impl SmoothedParam {
    /// Creates a new smoothed parameter.
    ///
    /// # Arguments
    ///
    /// - `initial`: Initial value.
    /// - `smooth_time_ms`: Smoothing time in milliseconds.
    /// - `sample_rate`: Sample rate in Hz.
    #[must_use]
    pub fn new(initial: f32, smooth_time_ms: f32, sample_rate: f32) -> Self {
        let samples = smooth_time_ms * sample_rate / 1000.0;
        let coeff = (-1.0 / samples).exp();

        Self {
            current: initial,
            target: initial,
            coeff,
        }
    }

    /// Sets the target value.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Gets the next smoothed value.
    #[must_use]
    pub fn next(&mut self) -> f32 {
        self.current = self.target + self.coeff * (self.current - self.target);
        self.current
    }

    /// Returns true if the value has reached the target.
    #[must_use]
    pub fn is_settled(&self) -> bool {
        (self.current - self.target).abs() < 1e-6
    }

    /// Immediately sets both current and target.
    pub fn set_immediate(&mut self, value: f32) {
        self.current = value;
        self.target = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoothed_param() {
        // Use 1ms smoothing at 48kHz = 48 samples time constant
        // Need ~5 time constants (240 samples) to reach 99%
        let mut param = SmoothedParam::new(0.0, 1.0, 48000.0);
        param.set_target(1.0);

        // Run for 500 samples (~10 time constants)
        for _ in 0..500 {
            param.next();
        }

        // Should be very close to target
        assert!(
            (param.next() - 1.0).abs() < 0.01,
            "Expected ~1.0, got {}",
            param.next()
        );
    }

    #[test]
    fn test_smoothed_param_immediate() {
        let mut param = SmoothedParam::new(0.0, 10.0, 48000.0);
        param.set_immediate(0.5);
        assert!((param.next() - 0.5).abs() < 0.001);
    }
}
