//! Audio format definitions: sample rates, channel layouts.

use core::fmt;

/// Supported sample rates.
///
/// Professional audio typically uses 44.1kHz (CD quality) or 48kHz (video/broadcast).
/// Higher rates (88.2kHz, 96kHz, 192kHz) are used for high-resolution audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum SampleRate {
    /// 44,100 Hz - CD quality, music production standard.
    Hz44100 = 44100,
    /// 48,000 Hz - Video/broadcast standard, recommended default.
    Hz48000 = 48000,
    /// 88,200 Hz - 2x CD rate, high-resolution audio.
    Hz88200 = 88200,
    /// 96,000 Hz - 2x broadcast rate, high-resolution audio.
    Hz96000 = 96000,
    /// 176,400 Hz - 4x CD rate, ultra high-resolution.
    Hz176400 = 176_400,
    /// 192,000 Hz - 4x broadcast rate, ultra high-resolution.
    Hz192000 = 192_000,
}

impl SampleRate {
    /// Returns the sample rate in Hz.
    #[inline]
    #[must_use]
    pub const fn as_hz(self) -> u32 {
        self as u32
    }

    /// Returns the sample rate as f32 for calculations.
    #[inline]
    #[must_use]
    pub const fn as_f32(self) -> f32 {
        self as u32 as f32
    }

    /// Returns the sample rate as f64 for high-precision calculations.
    #[inline]
    #[must_use]
    pub const fn as_f64(self) -> f64 {
        self as u32 as f64
    }

    /// Calculates the number of samples for a given duration in seconds.
    #[inline]
    #[must_use]
    pub fn samples_for_duration(self, seconds: f64) -> usize {
        (seconds * self.as_f64()) as usize
    }

    /// Calculates the duration in seconds for a given number of samples.
    #[inline]
    #[must_use]
    pub fn duration_for_samples(self, samples: usize) -> f64 {
        samples as f64 / self.as_f64()
    }

    /// Attempts to create a SampleRate from a raw Hz value.
    #[must_use]
    pub const fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            44100 => Some(Self::Hz44100),
            48000 => Some(Self::Hz48000),
            88200 => Some(Self::Hz88200),
            96000 => Some(Self::Hz96000),
            176_400 => Some(Self::Hz176400),
            192_000 => Some(Self::Hz192000),
            _ => None,
        }
    }
}

impl Default for SampleRate {
    fn default() -> Self {
        Self::Hz48000
    }
}

impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Hz", self.as_hz())
    }
}

/// Channel layout configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelLayout {
    /// Mono (1 channel).
    Mono,
    /// Stereo (2 channels: left, right).
    Stereo,
    /// LCR (3 channels: left, center, right).
    Lcr,
    /// Quad (4 channels: front left, front right, rear left, rear right).
    Quad,
    /// 5.1 surround (6 channels: L, R, C, LFE, Ls, Rs).
    Surround51,
    /// 7.1 surround (8 channels: L, R, C, LFE, Ls, Rs, Lb, Rb).
    Surround71,
    /// Custom channel count.
    Custom(usize),
}

impl ChannelLayout {
    /// Returns the number of channels in this layout.
    #[inline]
    #[must_use]
    pub const fn channel_count(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Lcr => 3,
            Self::Quad => 4,
            Self::Surround51 => 6,
            Self::Surround71 => 8,
            Self::Custom(n) => *n,
        }
    }
}

impl Default for ChannelLayout {
    fn default() -> Self {
        Self::Stereo
    }
}

impl From<usize> for ChannelLayout {
    fn from(channels: usize) -> Self {
        match channels {
            1 => Self::Mono,
            2 => Self::Stereo,
            3 => Self::Lcr,
            4 => Self::Quad,
            6 => Self::Surround51,
            8 => Self::Surround71,
            n => Self::Custom(n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_rate_conversions() {
        let rate = SampleRate::Hz48000;
        assert_eq!(rate.as_hz(), 48000);
        assert_eq!(rate.samples_for_duration(1.0), 48000);
        assert!((rate.duration_for_samples(48000) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_channel_layout() {
        assert_eq!(ChannelLayout::Mono.channel_count(), 1);
        assert_eq!(ChannelLayout::Stereo.channel_count(), 2);
        assert_eq!(ChannelLayout::Surround51.channel_count(), 6);
    }
}
