//! Audio stream types and state management.

use crate::{config::StreamConfig, Result};

/// State of an audio stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    /// Stream is stopped (not processing audio).
    Stopped,
    /// Stream is running (processing audio).
    Running,
    /// Stream is paused (can resume without re-initialization).
    Paused,
    /// Stream encountered an error.
    Error,
}

impl StreamState {
    /// Returns true if the stream is currently processing audio.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns the state name as a string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Error => "error",
        }
    }
}

/// Information passed to the audio callback.
#[derive(Debug, Clone)]
pub struct CallbackInfo {
    /// Current stream time in samples since start.
    pub stream_time_samples: u64,
    /// Current stream time in seconds since start.
    pub stream_time_secs: f64,
    /// Number of frames in this callback.
    pub frames: usize,
    /// Sample rate.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: usize,
}

impl CallbackInfo {
    /// Returns the callback duration in seconds.
    #[must_use]
    pub fn duration_secs(&self) -> f64 {
        self.frames as f64 / self.sample_rate as f64
    }
}

/// Trait for audio streams (both input and output).
pub trait AudioStream: Send {
    /// Returns the stream configuration.
    fn config(&self) -> &StreamConfig;

    /// Returns the current stream state.
    fn state(&self) -> StreamState;

    /// Starts the audio stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the stream cannot be started.
    fn start(&mut self) -> Result<()>;

    /// Stops the audio stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the stream cannot be stopped.
    fn stop(&mut self) -> Result<()>;

    /// Pauses the audio stream (if supported).
    ///
    /// # Errors
    ///
    /// Returns an error if pausing is not supported or fails.
    fn pause(&mut self) -> Result<()> {
        self.stop()
    }

    /// Resumes a paused stream.
    ///
    /// # Errors
    ///
    /// Returns an error if the stream cannot be resumed.
    fn resume(&mut self) -> Result<()> {
        self.start()
    }

    /// Returns the estimated output latency in samples.
    fn latency_samples(&self) -> usize;

    /// Returns the estimated output latency in seconds.
    fn latency_secs(&self) -> f64 {
        self.latency_samples() as f64 / self.config().sample_rate as f64
    }
}

/// Callback timing statistics.
#[derive(Debug, Clone, Default)]
pub struct CallbackStats {
    /// Number of callbacks processed.
    pub callback_count: u64,
    /// Total time spent in callbacks (nanoseconds).
    pub total_time_ns: u64,
    /// Maximum callback duration (nanoseconds).
    pub max_time_ns: u64,
    /// Number of overruns (callback took too long).
    pub overruns: u64,
    /// Number of underruns (buffer was empty).
    pub underruns: u64,
}

impl CallbackStats {
    /// Returns the average callback duration in microseconds.
    #[must_use]
    pub fn avg_time_us(&self) -> f64 {
        if self.callback_count == 0 {
            0.0
        } else {
            (self.total_time_ns as f64 / self.callback_count as f64) / 1000.0
        }
    }

    /// Returns the maximum callback duration in microseconds.
    #[must_use]
    pub fn max_time_us(&self) -> f64 {
        self.max_time_ns as f64 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Phase 3 TDD: Comprehensive stream tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // StreamState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_stream_state_is_active_running() {
        assert!(StreamState::Running.is_active());
    }

    #[test]
    fn test_stream_state_is_active_stopped() {
        assert!(!StreamState::Stopped.is_active());
    }

    #[test]
    fn test_stream_state_is_active_paused() {
        assert!(!StreamState::Paused.is_active());
    }

    #[test]
    fn test_stream_state_is_active_error() {
        assert!(!StreamState::Error.is_active());
    }

    #[test]
    fn test_stream_state_as_str() {
        assert_eq!(StreamState::Stopped.as_str(), "stopped");
        assert_eq!(StreamState::Running.as_str(), "running");
        assert_eq!(StreamState::Paused.as_str(), "paused");
        assert_eq!(StreamState::Error.as_str(), "error");
    }

    #[test]
    fn test_stream_state_eq() {
        assert_eq!(StreamState::Running, StreamState::Running);
        assert_ne!(StreamState::Running, StreamState::Stopped);
    }

    #[test]
    fn test_stream_state_clone() {
        let state = StreamState::Paused;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_stream_state_copy() {
        let state = StreamState::Running;
        let copied: StreamState = state; // Copy, not move
        assert_eq!(state, copied);
    }

    #[test]
    fn test_stream_state_debug() {
        assert!(format!("{:?}", StreamState::Running).contains("Running"));
        assert!(format!("{:?}", StreamState::Stopped).contains("Stopped"));
        assert!(format!("{:?}", StreamState::Paused).contains("Paused"));
        assert!(format!("{:?}", StreamState::Error).contains("Error"));
    }

    // -------------------------------------------------------------------------
    // CallbackInfo tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_callback_info_duration_secs() {
        let info = CallbackInfo {
            stream_time_samples: 0,
            stream_time_secs: 0.0,
            frames: 480,
            sample_rate: 48000,
            channels: 2,
        };

        // 480 frames at 48kHz = 10ms = 0.01s
        let duration = info.duration_secs();
        assert!((duration - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_callback_info_duration_various_rates() {
        let test_cases = [
            (441, 44100, 0.01),   // 10ms at 44.1kHz
            (480, 48000, 0.01),   // 10ms at 48kHz
            (960, 96000, 0.01),   // 10ms at 96kHz
            (128, 48000, 0.00267), // ~2.67ms
            (256, 48000, 0.00533), // ~5.33ms
            (512, 48000, 0.01067), // ~10.67ms
        ];

        for (frames, rate, expected) in test_cases {
            let info = CallbackInfo {
                stream_time_samples: 0,
                stream_time_secs: 0.0,
                frames,
                sample_rate: rate,
                channels: 2,
            };

            let duration = info.duration_secs();
            assert!(
                (duration - expected).abs() < 0.0001,
                "frames={}, rate={}: expected {}, got {}",
                frames,
                rate,
                expected,
                duration
            );
        }
    }

    #[test]
    fn test_callback_info_clone() {
        let info = CallbackInfo {
            stream_time_samples: 48000,
            stream_time_secs: 1.0,
            frames: 512,
            sample_rate: 48000,
            channels: 2,
        };

        let cloned = info.clone();

        assert_eq!(cloned.stream_time_samples, info.stream_time_samples);
        assert!((cloned.stream_time_secs - info.stream_time_secs).abs() < 0.001);
        assert_eq!(cloned.frames, info.frames);
        assert_eq!(cloned.sample_rate, info.sample_rate);
        assert_eq!(cloned.channels, info.channels);
    }

    #[test]
    fn test_callback_info_debug() {
        let info = CallbackInfo {
            stream_time_samples: 96000,
            stream_time_secs: 2.0,
            frames: 256,
            sample_rate: 48000,
            channels: 2,
        };

        let debug = format!("{:?}", info);
        assert!(debug.contains("CallbackInfo"));
        assert!(debug.contains("96000")); // stream_time_samples
        assert!(debug.contains("256"));   // frames
    }

    #[test]
    fn test_callback_info_stream_time_progression() {
        // Simulate a sequence of callbacks
        let frames = 512;
        let sample_rate = 48000;
        let duration_per_callback = frames as f64 / sample_rate as f64;

        for i in 0..10 {
            let info = CallbackInfo {
                stream_time_samples: (i * frames) as u64,
                stream_time_secs: i as f64 * duration_per_callback,
                frames,
                sample_rate,
                channels: 2,
            };

            let expected_time = i as f64 * duration_per_callback;
            assert!(
                (info.stream_time_secs - expected_time).abs() < 0.0001,
                "Callback {}: expected {}s, got {}s",
                i,
                expected_time,
                info.stream_time_secs
            );
        }
    }

    // -------------------------------------------------------------------------
    // CallbackStats tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_callback_stats_default() {
        let stats = CallbackStats::default();

        assert_eq!(stats.callback_count, 0);
        assert_eq!(stats.total_time_ns, 0);
        assert_eq!(stats.max_time_ns, 0);
        assert_eq!(stats.overruns, 0);
        assert_eq!(stats.underruns, 0);
    }

    #[test]
    fn test_callback_stats_avg_time_zero_callbacks() {
        let stats = CallbackStats::default();
        assert!((stats.avg_time_us() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_callback_stats_avg_time() {
        let stats = CallbackStats {
            callback_count: 100,
            total_time_ns: 10_000_000, // 10ms total = 100μs average
            max_time_ns: 200_000,       // 200μs max
            overruns: 0,
            underruns: 0,
        };

        // Average: 10_000_000ns / 100 = 100_000ns = 100μs
        let avg = stats.avg_time_us();
        assert!((avg - 100.0).abs() < 0.1, "Expected 100μs, got {}", avg);
    }

    #[test]
    fn test_callback_stats_max_time() {
        let stats = CallbackStats {
            callback_count: 100,
            total_time_ns: 10_000_000,
            max_time_ns: 500_000, // 500μs
            overruns: 0,
            underruns: 0,
        };

        let max = stats.max_time_us();
        assert!((max - 500.0).abs() < 0.1, "Expected 500μs, got {}", max);
    }

    #[test]
    fn test_callback_stats_clone() {
        let stats = CallbackStats {
            callback_count: 50,
            total_time_ns: 5_000_000,
            max_time_ns: 150_000,
            overruns: 2,
            underruns: 1,
        };

        let cloned = stats.clone();

        assert_eq!(cloned.callback_count, stats.callback_count);
        assert_eq!(cloned.total_time_ns, stats.total_time_ns);
        assert_eq!(cloned.max_time_ns, stats.max_time_ns);
        assert_eq!(cloned.overruns, stats.overruns);
        assert_eq!(cloned.underruns, stats.underruns);
    }

    #[test]
    fn test_callback_stats_debug() {
        let stats = CallbackStats {
            callback_count: 100,
            total_time_ns: 10_000_000,
            max_time_ns: 200_000,
            overruns: 5,
            underruns: 3,
        };

        let debug = format!("{:?}", stats);
        assert!(debug.contains("CallbackStats"));
        assert!(debug.contains("100")); // callback_count
        assert!(debug.contains("5"));   // overruns
    }

    #[test]
    fn test_callback_stats_overruns_underruns() {
        let stats = CallbackStats {
            callback_count: 1000,
            total_time_ns: 100_000_000,
            max_time_ns: 15_000_000, // 15ms - would cause overrun at 10ms callback
            overruns: 10,
            underruns: 5,
        };

        assert_eq!(stats.overruns, 10);
        assert_eq!(stats.underruns, 5);
    }

    // -------------------------------------------------------------------------
    // Latency and timing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_callback_timing_budget() {
        // At 48kHz with 256 sample buffer, callback budget is ~5.33ms
        let info = CallbackInfo {
            stream_time_samples: 0,
            stream_time_secs: 0.0,
            frames: 256,
            sample_rate: 48000,
            channels: 2,
        };

        let budget_ms = info.duration_secs() * 1000.0;
        assert!(
            budget_ms > 5.0 && budget_ms < 6.0,
            "Expected ~5.33ms, got {}ms",
            budget_ms
        );
    }

    #[test]
    fn test_low_latency_callback_budget() {
        // At 96kHz with 64 sample buffer, callback budget is ~0.67ms
        let info = CallbackInfo {
            stream_time_samples: 0,
            stream_time_secs: 0.0,
            frames: 64,
            sample_rate: 96000,
            channels: 2,
        };

        let budget_ms = info.duration_secs() * 1000.0;
        assert!(
            budget_ms < 1.0,
            "Low latency should be <1ms, got {}ms",
            budget_ms
        );
    }

    #[test]
    fn test_callback_stats_performance_monitoring() {
        // Simulate good performance
        let good_stats = CallbackStats {
            callback_count: 10000,
            total_time_ns: 50_000_000_000, // 50ms total for 10000 callbacks
            max_time_ns: 8_000_000,         // 8ms max (under 10ms budget)
            overruns: 0,
            underruns: 0,
        };

        // Average should be 5ms (good)
        assert!(good_stats.avg_time_us() < 6000.0);
        assert_eq!(good_stats.overruns, 0);

        // Simulate poor performance
        let poor_stats = CallbackStats {
            callback_count: 10000,
            total_time_ns: 100_000_000_000, // 100ms total (10μs average)
            max_time_ns: 15_000_000,         // 15ms max (over budget)
            overruns: 50,
            underruns: 10,
        };

        assert!(poor_stats.overruns > 0, "Should have overruns");
    }
}
