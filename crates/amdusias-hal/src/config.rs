//! Stream configuration types.

/// Configuration for an audio stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamConfig {
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Buffer size in frames (samples per channel).
    pub buffer_size: usize,
    /// Number of channels.
    pub channels: usize,
    /// Whether to use exclusive mode (if available).
    pub exclusive: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 512,
            channels: 2,
            exclusive: true,
        }
    }
}

impl StreamConfig {
    /// Creates a new configuration with the specified parameters.
    #[must_use]
    pub const fn new(sample_rate: u32, buffer_size: usize, channels: usize) -> Self {
        Self {
            sample_rate,
            buffer_size,
            channels,
            exclusive: true,
        }
    }

    /// Returns the buffer duration in seconds.
    #[must_use]
    pub fn buffer_duration_secs(&self) -> f64 {
        self.buffer_size as f64 / self.sample_rate as f64
    }

    /// Returns the buffer duration in milliseconds.
    #[must_use]
    pub fn buffer_duration_ms(&self) -> f64 {
        self.buffer_duration_secs() * 1000.0
    }

    /// Returns the total number of samples per buffer (frames * channels).
    #[must_use]
    pub const fn total_samples(&self) -> usize {
        self.buffer_size * self.channels
    }

    /// Sets exclusive mode.
    #[must_use]
    pub const fn with_exclusive(mut self, exclusive: bool) -> Self {
        self.exclusive = exclusive;
        self
    }
}

/// Supported buffer sizes for a device.
#[derive(Debug, Clone)]
pub struct BufferSizeRange {
    /// Minimum buffer size in frames.
    pub min: usize,
    /// Maximum buffer size in frames.
    pub max: usize,
    /// Preferred buffer size (may be 0 if no preference).
    pub preferred: usize,
}

impl BufferSizeRange {
    /// Checks if a buffer size is within the supported range.
    #[must_use]
    pub const fn contains(&self, size: usize) -> bool {
        size >= self.min && size <= self.max
    }

    /// Clamps a buffer size to the supported range.
    #[must_use]
    pub fn clamp(&self, size: usize) -> usize {
        size.clamp(self.min, self.max)
    }
}

/// Supported sample rates for a device.
#[derive(Debug, Clone)]
pub enum SampleRateRange {
    /// Discrete set of supported sample rates.
    Discrete(Vec<u32>),
    /// Continuous range of supported sample rates.
    Range { min: u32, max: u32 },
}

impl SampleRateRange {
    /// Checks if a sample rate is supported.
    #[must_use]
    pub fn contains(&self, rate: u32) -> bool {
        match self {
            Self::Discrete(rates) => rates.contains(&rate),
            Self::Range { min, max } => rate >= *min && rate <= *max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_duration() {
        let config = StreamConfig::new(48000, 256, 2);
        let duration_ms = config.buffer_duration_ms();
        assert!((duration_ms - 5.333).abs() < 0.01);
    }

    #[test]
    fn test_total_samples() {
        let config = StreamConfig::new(48000, 512, 2);
        assert_eq!(config.total_samples(), 1024);
    }

    // =========================================================================
    // Phase 3 TDD: Comprehensive StreamConfig tests
    // =========================================================================

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();

        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.channels, 2);
        assert!(config.exclusive);
    }

    #[test]
    fn test_stream_config_new() {
        let config = StreamConfig::new(44100, 256, 1);

        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.buffer_size, 256);
        assert_eq!(config.channels, 1);
        assert!(config.exclusive); // Default is true
    }

    #[test]
    fn test_stream_config_with_exclusive() {
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(false);

        assert!(!config.exclusive);

        let config_exclusive = StreamConfig::new(48000, 512, 2).with_exclusive(true);
        assert!(config_exclusive.exclusive);
    }

    #[test]
    fn test_buffer_duration_secs() {
        let config = StreamConfig::new(48000, 480, 2);
        let duration = config.buffer_duration_secs();

        // 480 samples at 48kHz = 10ms = 0.01s
        assert!((duration - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_buffer_duration_ms_various_rates() {
        // Test common sample rates
        let configs = [
            (44100, 441, 10.0),   // 441 samples @ 44.1kHz = 10ms
            (48000, 480, 10.0),   // 480 samples @ 48kHz = 10ms
            (96000, 960, 10.0),   // 960 samples @ 96kHz = 10ms
            (192000, 1920, 10.0), // 1920 samples @ 192kHz = 10ms
        ];

        for (rate, buffer, expected_ms) in configs {
            let config = StreamConfig::new(rate, buffer, 2);
            let duration = config.buffer_duration_ms();
            assert!(
                (duration - expected_ms).abs() < 0.01,
                "Rate {}: expected {}ms, got {}ms",
                rate,
                expected_ms,
                duration
            );
        }
    }

    #[test]
    fn test_total_samples_mono() {
        let config = StreamConfig::new(48000, 256, 1);
        assert_eq!(config.total_samples(), 256);
    }

    #[test]
    fn test_total_samples_stereo() {
        let config = StreamConfig::new(48000, 256, 2);
        assert_eq!(config.total_samples(), 512);
    }

    #[test]
    fn test_total_samples_multichannel() {
        let config = StreamConfig::new(48000, 256, 8);
        assert_eq!(config.total_samples(), 2048);
    }

    #[test]
    fn test_stream_config_clone() {
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(false);
        let cloned = config.clone();

        assert_eq!(cloned.sample_rate, config.sample_rate);
        assert_eq!(cloned.buffer_size, config.buffer_size);
        assert_eq!(cloned.channels, config.channels);
        assert_eq!(cloned.exclusive, config.exclusive);
    }

    #[test]
    fn test_stream_config_eq() {
        let config1 = StreamConfig::new(48000, 512, 2);
        let config2 = StreamConfig::new(48000, 512, 2);
        let config3 = StreamConfig::new(44100, 512, 2);

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_stream_config_debug() {
        let config = StreamConfig::new(48000, 512, 2);
        let debug_str = format!("{:?}", config);

        assert!(debug_str.contains("StreamConfig"));
        assert!(debug_str.contains("48000"));
        assert!(debug_str.contains("512"));
    }

    // -------------------------------------------------------------------------
    // BufferSizeRange tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_buffer_size_range_contains() {
        let range = BufferSizeRange {
            min: 64,
            max: 4096,
            preferred: 512,
        };

        assert!(range.contains(64));   // Min
        assert!(range.contains(512));  // Preferred
        assert!(range.contains(4096)); // Max
        assert!(range.contains(256));  // Middle

        assert!(!range.contains(32));   // Below min
        assert!(!range.contains(8192)); // Above max
    }

    #[test]
    fn test_buffer_size_range_clamp() {
        let range = BufferSizeRange {
            min: 64,
            max: 4096,
            preferred: 512,
        };

        assert_eq!(range.clamp(32), 64);    // Below min -> min
        assert_eq!(range.clamp(8192), 4096); // Above max -> max
        assert_eq!(range.clamp(256), 256);   // In range -> unchanged
        assert_eq!(range.clamp(64), 64);     // At min -> unchanged
        assert_eq!(range.clamp(4096), 4096); // At max -> unchanged
    }

    #[test]
    fn test_buffer_size_range_clone() {
        let range = BufferSizeRange {
            min: 128,
            max: 2048,
            preferred: 256,
        };
        let cloned = range.clone();

        assert_eq!(cloned.min, range.min);
        assert_eq!(cloned.max, range.max);
        assert_eq!(cloned.preferred, range.preferred);
    }

    #[test]
    fn test_buffer_size_range_debug() {
        let range = BufferSizeRange {
            min: 64,
            max: 4096,
            preferred: 512,
        };
        let debug_str = format!("{:?}", range);

        assert!(debug_str.contains("BufferSizeRange"));
        assert!(debug_str.contains("64"));
        assert!(debug_str.contains("4096"));
    }

    // -------------------------------------------------------------------------
    // SampleRateRange tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sample_rate_range_discrete() {
        let range = SampleRateRange::Discrete(vec![44100, 48000, 96000]);

        assert!(range.contains(44100));
        assert!(range.contains(48000));
        assert!(range.contains(96000));

        assert!(!range.contains(22050));
        assert!(!range.contains(192000));
    }

    #[test]
    fn test_sample_rate_range_continuous() {
        let range = SampleRateRange::Range {
            min: 8000,
            max: 192000,
        };

        assert!(range.contains(8000));   // Min
        assert!(range.contains(192000)); // Max
        assert!(range.contains(48000));  // Middle
        assert!(range.contains(44100));  // Common rate

        assert!(!range.contains(4000));   // Below min
        assert!(!range.contains(384000)); // Above max
    }

    #[test]
    fn test_sample_rate_range_discrete_empty() {
        let range = SampleRateRange::Discrete(vec![]);

        assert!(!range.contains(48000));
    }

    #[test]
    fn test_sample_rate_range_clone() {
        let range = SampleRateRange::Discrete(vec![44100, 48000]);
        let cloned = range.clone();

        assert!(cloned.contains(44100));
        assert!(cloned.contains(48000));
    }

    #[test]
    fn test_sample_rate_range_debug() {
        let discrete = SampleRateRange::Discrete(vec![48000]);
        let continuous = SampleRateRange::Range { min: 8000, max: 192000 };

        let discrete_str = format!("{:?}", discrete);
        let continuous_str = format!("{:?}", continuous);

        assert!(discrete_str.contains("Discrete"));
        assert!(continuous_str.contains("Range"));
    }

    // -------------------------------------------------------------------------
    // Latency calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_low_latency_config() {
        // 64 samples @ 48kHz = 1.33ms - professional low latency
        let config = StreamConfig::new(48000, 64, 2);
        let latency_ms = config.buffer_duration_ms();

        assert!(latency_ms < 2.0, "Low latency config: {}ms", latency_ms);
    }

    #[test]
    fn test_ultra_low_latency_config() {
        // 32 samples @ 96kHz = 0.33ms - ultra low latency
        let config = StreamConfig::new(96000, 32, 2);
        let latency_ms = config.buffer_duration_ms();

        assert!(latency_ms < 0.5, "Ultra low latency config: {}ms", latency_ms);
    }

    #[test]
    fn test_high_sample_rate_latency() {
        // Higher sample rates mean lower latency for same buffer size
        let config_48k = StreamConfig::new(48000, 256, 2);
        let config_96k = StreamConfig::new(96000, 256, 2);

        let latency_48k = config_48k.buffer_duration_ms();
        let latency_96k = config_96k.buffer_duration_ms();

        assert!(latency_96k < latency_48k);
        assert!((latency_48k / latency_96k - 2.0).abs() < 0.01); // Should be exactly 2x
    }
}
