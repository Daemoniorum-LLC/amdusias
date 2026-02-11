//! Audio buffer types with SIMD alignment and zero-copy semantics.

use crate::{error::Result, format::SampleRate, ChannelCount, Error, FrameCount, Sample};
use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};

/// SIMD alignment requirement (32 bytes for AVX2).
pub const SIMD_ALIGNMENT: usize = 32;

/// Audio buffer with compile-time channel count.
///
/// The buffer stores samples in **interleaved** format for cache efficiency
/// during typical audio processing (sample-by-sample iteration).
///
/// For SIMD processing, use [`AudioBuffer::as_planar`] to get a planar view.
///
/// # Type Parameter
///
/// - `CHANNELS`: The number of audio channels (e.g., 2 for stereo).
///
/// # Memory Layout
///
/// Samples are stored interleaved: `[L0, R0, L1, R1, L2, R2, ...]`
///
/// The buffer is aligned to 32 bytes for AVX2 SIMD operations.
#[repr(C, align(32))]
pub struct AudioBuffer<const CHANNELS: usize> {
    /// Interleaved sample data.
    samples: Box<[Sample]>,
    /// Number of frames (samples per channel).
    frames: FrameCount,
    /// Sample rate for this buffer.
    sample_rate: SampleRate,
}

impl<const CHANNELS: usize> AudioBuffer<CHANNELS> {
    /// Creates a new audio buffer with the specified frame count.
    ///
    /// # Arguments
    ///
    /// - `frames`: Number of frames (samples per channel).
    /// - `sample_rate`: The sample rate for this buffer.
    ///
    /// # Panics
    ///
    /// Panics if `CHANNELS` is 0.
    #[must_use]
    pub fn new(frames: FrameCount, sample_rate: SampleRate) -> Self {
        assert!(CHANNELS > 0, "channel count must be > 0");

        let total_samples = frames * CHANNELS;
        let samples = alloc::vec![0.0; total_samples].into_boxed_slice();

        Self {
            samples,
            frames,
            sample_rate,
        }
    }

    /// Returns the number of frames in this buffer.
    #[inline]
    #[must_use]
    pub const fn frames(&self) -> FrameCount {
        self.frames
    }

    /// Returns the number of channels.
    #[inline]
    #[must_use]
    pub const fn channels(&self) -> ChannelCount {
        CHANNELS
    }

    /// Returns the total number of samples (frames * channels).
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.frames * CHANNELS
    }

    /// Returns true if the buffer has no samples.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.frames == 0
    }

    /// Returns the sample rate.
    #[inline]
    #[must_use]
    pub const fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Returns a reference to the raw sample data (interleaved).
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Sample] {
        &self.samples
    }

    /// Returns a mutable reference to the raw sample data (interleaved).
    #[inline]
    #[must_use]
    pub fn as_slice_mut(&mut self) -> &mut [Sample] {
        &mut self.samples
    }

    /// Gets a sample at the specified frame and channel.
    ///
    /// # Panics
    ///
    /// Panics if `frame >= self.frames()` or `channel >= CHANNELS`.
    #[inline]
    #[must_use]
    pub fn get(&self, frame: usize, channel: usize) -> Sample {
        debug_assert!(frame < self.frames, "frame out of bounds");
        debug_assert!(channel < CHANNELS, "channel out of bounds");
        self.samples[frame * CHANNELS + channel]
    }

    /// Sets a sample at the specified frame and channel.
    ///
    /// # Panics
    ///
    /// Panics if `frame >= self.frames()` or `channel >= CHANNELS`.
    #[inline]
    pub fn set(&mut self, frame: usize, channel: usize, value: Sample) {
        debug_assert!(frame < self.frames, "frame out of bounds");
        debug_assert!(channel < CHANNELS, "channel out of bounds");
        self.samples[frame * CHANNELS + channel] = value;
    }

    /// Fills the entire buffer with a constant value.
    #[inline]
    pub fn fill(&mut self, value: Sample) {
        self.samples.fill(value);
    }

    /// Clears the buffer (fills with silence).
    #[inline]
    pub fn clear(&mut self) {
        self.fill(0.0);
    }

    /// Applies a gain (volume) multiplier to all samples.
    ///
    /// This operation is SIMD-optimized when the `simd` feature is enabled.
    pub fn apply_gain(&mut self, gain: Sample) {
        #[cfg(feature = "simd")]
        {
            crate::simd::apply_gain_simd(&mut self.samples, gain);
        }

        #[cfg(not(feature = "simd"))]
        {
            for sample in self.samples.iter_mut() {
                *sample *= gain;
            }
        }
    }

    /// Copies samples from another buffer of the same format.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer sizes don't match.
    pub fn copy_from(&mut self, other: &Self) -> Result<()> {
        if self.frames != other.frames {
            return Err(Error::BufferSizeMismatch {
                expected: self.frames,
                actual: other.frames,
            });
        }

        self.samples.copy_from_slice(&other.samples);
        Ok(())
    }

    /// Adds samples from another buffer (mixing).
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer sizes don't match.
    pub fn mix_from(&mut self, other: &Self) -> Result<()> {
        if self.frames != other.frames {
            return Err(Error::BufferSizeMismatch {
                expected: self.frames,
                actual: other.frames,
            });
        }

        #[cfg(feature = "simd")]
        {
            crate::simd::mix_buffers_simd(&mut self.samples, &other.samples);
        }

        #[cfg(not(feature = "simd"))]
        {
            for (dst, src) in self.samples.iter_mut().zip(other.samples.iter()) {
                *dst += *src;
            }
        }

        Ok(())
    }

    /// Returns an iterator over frames, yielding a slice of channels for each frame.
    #[inline]
    pub fn frames_iter(&self) -> impl Iterator<Item = &[Sample]> {
        self.samples.chunks_exact(CHANNELS)
    }

    /// Returns a mutable iterator over frames.
    #[inline]
    pub fn frames_iter_mut(&mut self) -> impl Iterator<Item = &mut [Sample]> {
        self.samples.chunks_exact_mut(CHANNELS)
    }
}

impl<const CHANNELS: usize> Deref for AudioBuffer<CHANNELS> {
    type Target = [Sample];

    fn deref(&self) -> &Self::Target {
        &self.samples
    }
}

impl<const CHANNELS: usize> DerefMut for AudioBuffer<CHANNELS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.samples
    }
}

/// Dynamically-sized audio buffer for runtime-determined channel counts.
#[repr(C, align(32))]
pub struct DynamicBuffer {
    /// Interleaved sample data.
    samples: Box<[Sample]>,
    /// Number of frames.
    frames: FrameCount,
    /// Number of channels.
    channels: ChannelCount,
    /// Sample rate.
    sample_rate: SampleRate,
}

impl DynamicBuffer {
    /// Creates a new dynamic buffer.
    #[must_use]
    pub fn new(frames: FrameCount, channels: ChannelCount, sample_rate: SampleRate) -> Self {
        assert!(channels > 0, "channel count must be > 0");

        let total_samples = frames * channels;
        let samples = alloc::vec![0.0; total_samples].into_boxed_slice();

        Self {
            samples,
            frames,
            channels,
            sample_rate,
        }
    }

    /// Returns the number of frames.
    #[inline]
    #[must_use]
    pub const fn frames(&self) -> FrameCount {
        self.frames
    }

    /// Returns the number of channels.
    #[inline]
    #[must_use]
    pub const fn channels(&self) -> ChannelCount {
        self.channels
    }

    /// Returns the sample rate.
    #[inline]
    #[must_use]
    pub const fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    /// Returns a reference to the raw sample data.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[Sample] {
        &self.samples
    }

    /// Returns a mutable reference to the raw sample data.
    #[inline]
    #[must_use]
    pub fn as_slice_mut(&mut self) -> &mut [Sample] {
        &mut self.samples
    }

    /// Clears the buffer.
    #[inline]
    pub fn clear(&mut self) {
        self.samples.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = AudioBuffer::<2>::new(512, SampleRate::Hz48000);
        assert_eq!(buffer.frames(), 512);
        assert_eq!(buffer.channels(), 2);
        assert_eq!(buffer.len(), 1024);
    }

    #[test]
    fn test_buffer_get_set() {
        let mut buffer = AudioBuffer::<2>::new(4, SampleRate::Hz48000);
        buffer.set(0, 0, 1.0); // Left channel, frame 0
        buffer.set(0, 1, 0.5); // Right channel, frame 0

        assert_eq!(buffer.get(0, 0), 1.0);
        assert_eq!(buffer.get(0, 1), 0.5);
    }

    #[test]
    fn test_apply_gain() {
        let mut buffer = AudioBuffer::<2>::new(4, SampleRate::Hz48000);
        buffer.fill(1.0);
        buffer.apply_gain(0.5);

        for sample in buffer.as_slice() {
            assert!((sample - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_buffer_struct_alignment() {
        // AudioBuffer struct should be 32-byte aligned for AVX2
        assert!(
            core::mem::align_of::<AudioBuffer<2>>() >= SIMD_ALIGNMENT,
            "AudioBuffer alignment {} is less than required {}",
            core::mem::align_of::<AudioBuffer<2>>(),
            SIMD_ALIGNMENT
        );
    }

    #[test]
    fn test_dynamic_buffer_alignment() {
        // DynamicBuffer should also be 32-byte aligned
        assert!(
            core::mem::align_of::<DynamicBuffer>() >= SIMD_ALIGNMENT,
            "DynamicBuffer alignment {} is less than required {}",
            core::mem::align_of::<DynamicBuffer>(),
            SIMD_ALIGNMENT
        );
    }

    #[test]
    fn test_buffer_interleaved_layout() {
        let mut buffer = AudioBuffer::<2>::new(4, SampleRate::Hz48000);

        // Set specific values
        buffer.set(0, 0, 1.0); // L0
        buffer.set(0, 1, 2.0); // R0
        buffer.set(1, 0, 3.0); // L1
        buffer.set(1, 1, 4.0); // R1

        // Verify interleaved layout: [L0, R0, L1, R1, ...]
        let slice = buffer.as_slice();
        assert_eq!(slice[0], 1.0, "L0 at index 0");
        assert_eq!(slice[1], 2.0, "R0 at index 1");
        assert_eq!(slice[2], 3.0, "L1 at index 2");
        assert_eq!(slice[3], 4.0, "R1 at index 3");
    }

    #[test]
    fn test_buffer_clear() {
        let mut buffer = AudioBuffer::<2>::new(256, SampleRate::Hz48000);
        buffer.fill(1.0);
        buffer.clear();

        for sample in buffer.as_slice() {
            assert_eq!(*sample, 0.0);
        }
    }

    #[test]
    fn test_buffer_copy_from() {
        let mut dst = AudioBuffer::<2>::new(4, SampleRate::Hz48000);
        let mut src = AudioBuffer::<2>::new(4, SampleRate::Hz48000);

        src.fill(0.75);
        dst.copy_from(&src).unwrap();

        for sample in dst.as_slice() {
            assert_eq!(*sample, 0.75);
        }
    }

    #[test]
    fn test_buffer_copy_from_size_mismatch() {
        let mut dst = AudioBuffer::<2>::new(4, SampleRate::Hz48000);
        let src = AudioBuffer::<2>::new(8, SampleRate::Hz48000);

        assert!(dst.copy_from(&src).is_err());
    }

    #[test]
    fn test_buffer_mix_from() {
        let mut dst = AudioBuffer::<2>::new(4, SampleRate::Hz48000);
        let mut src = AudioBuffer::<2>::new(4, SampleRate::Hz48000);

        dst.fill(1.0);
        src.fill(0.5);
        dst.mix_from(&src).unwrap();

        for sample in dst.as_slice() {
            assert!((sample - 1.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_buffer_frames_iterator() {
        let mut buffer = AudioBuffer::<2>::new(4, SampleRate::Hz48000);
        buffer.set(0, 0, 1.0);
        buffer.set(0, 1, 2.0);
        buffer.set(1, 0, 3.0);
        buffer.set(1, 1, 4.0);

        let frames: Vec<&[Sample]> = buffer.frames_iter().collect();
        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0], &[1.0, 2.0]);
        assert_eq!(frames[1], &[3.0, 4.0]);
    }

    #[test]
    fn test_buffer_empty() {
        let buffer = AudioBuffer::<2>::new(0, SampleRate::Hz48000);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_mono_buffer() {
        let mut buffer = AudioBuffer::<1>::new(256, SampleRate::Hz48000);
        buffer.set(0, 0, 0.5);
        assert_eq!(buffer.get(0, 0), 0.5);
        assert_eq!(buffer.channels(), 1);
    }

    #[test]
    fn test_multichannel_buffer() {
        let buffer = AudioBuffer::<8>::new(128, SampleRate::Hz48000);
        assert_eq!(buffer.channels(), 8);
        assert_eq!(buffer.len(), 128 * 8);
    }
}
