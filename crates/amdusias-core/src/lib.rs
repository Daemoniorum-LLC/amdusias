//! # amdusias-core
//!
//! Core real-time audio primitives for the Amdusias audio engine.
//!
//! This crate provides the foundational building blocks for professional audio:
//!
//! - **Lock-free data structures** for audio thread communication
//! - **SIMD-optimized audio buffers** with zero-copy semantics
//! - **Sample-accurate scheduling** for events and automation
//! - **Real-time thread utilities** for priority elevation
//!
//! ## Design Principles
//!
//! 1. **No allocations in the audio thread** - all memory is pre-allocated
//! 2. **No locks in the audio thread** - only lock-free primitives
//! 3. **No syscalls in the audio thread** - no I/O, no mutexes
//! 4. **SIMD by default** - vectorized processing where beneficial
//!
//! ## Example
//!
//! ```rust
//! use amdusias_core::{AudioBuffer, SampleRate, SpscQueue};
//!
//! // Create a stereo buffer at 48kHz
//! let mut buffer = AudioBuffer::<2>::new(512, SampleRate::Hz48000);
//!
//! // Process samples
//! buffer.fill(0.0);
//! buffer.apply_gain(0.5);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

pub mod buffer;
pub mod error;
pub mod format;
pub mod queue;
pub mod schedule;
pub mod simd;

pub use buffer::AudioBuffer;
pub use error::{Error, Result};
pub use format::{ChannelLayout, SampleRate};
pub use queue::SpscQueue;
pub use schedule::{SamplePosition, Scheduler};

/// Frame count type (number of samples per channel).
pub type FrameCount = usize;

/// Sample type (32-bit float, industry standard).
pub type Sample = f32;

/// Channel count type.
pub type ChannelCount = usize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_rate_to_hz() {
        assert_eq!(SampleRate::Hz48000.as_hz(), 48000);
        assert_eq!(SampleRate::Hz44100.as_hz(), 44100);
    }
}
