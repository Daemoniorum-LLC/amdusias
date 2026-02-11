//! # amdusias-dsp
//!
//! Digital Signal Processing primitives for the Amdusias audio engine.
//!
//! This crate provides high-performance DSP building blocks:
//!
//! - **Filters**: Biquad, state-variable, FIR, allpass
//! - **Dynamics**: Compressor, limiter, gate, expander
//! - **Delay**: Basic delay, multi-tap, modulated
//! - **Reverb**: Algorithmic (Schroeder, Dattorro), convolution
//! - **Modulation**: Chorus, flanger, phaser
//! - **Analysis**: FFT, peak detection, RMS
//!
//! All processors implement the [`Processor`] trait for uniform handling.
//!
//! ## Example
//!
//! ```rust
//! use amdusias_dsp::{BiquadFilter, FilterType, Processor};
//!
//! // Create a lowpass filter at 1kHz
//! let mut filter = BiquadFilter::new(FilterType::Lowpass, 1000.0, 0.707, 48000.0);
//!
//! // Process a block of samples
//! let mut samples = [0.5, 0.3, -0.2, 0.1];
//! filter.process_block(&mut samples);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod biquad;
pub mod compressor;
pub mod delay;
pub mod envelope;
pub mod limiter;
pub mod reverb;
pub mod traits;

pub use biquad::{BiquadFilter, FilterType};
pub use compressor::Compressor;
pub use delay::DelayLine;
pub use envelope::{EnvelopeDetector, EnvelopeMode};
pub use limiter::Limiter;
pub use reverb::Reverb;
pub use traits::Processor;

/// Common sample type.
pub type Sample = f32;

/// Converts decibels to linear gain.
#[inline]
#[must_use]
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Converts linear gain to decibels.
#[inline]
#[must_use]
pub fn linear_to_db(linear: f32) -> f32 {
    20.0 * linear.abs().max(1e-10).log10()
}

/// Clamps a sample to the valid range [-1.0, 1.0].
#[inline]
#[must_use]
pub fn clamp_sample(sample: f32) -> f32 {
    sample.clamp(-1.0, 1.0)
}

/// Linear interpolation between two values.
#[inline]
#[must_use]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_conversions() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 1e-6);
        assert!((db_to_linear(-6.0) - 0.501).abs() < 0.01);
        assert!((linear_to_db(1.0) - 0.0).abs() < 1e-6);
    }
}
