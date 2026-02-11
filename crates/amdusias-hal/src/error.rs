//! Error types for the hardware abstraction layer.

use thiserror::Error;

/// Result type for HAL operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur in audio hardware operations.
#[derive(Debug, Error)]
pub enum Error {
    /// The requested device was not found.
    #[error("device not found: {0}")]
    DeviceNotFound(String),

    /// The device is already in use by another application.
    #[error("device busy: {0}")]
    DeviceBusy(String),

    /// The requested configuration is not supported by the device.
    #[error("unsupported configuration: {0}")]
    UnsupportedConfig(String),

    /// The requested sample rate is not supported.
    #[error("unsupported sample rate: {0} Hz")]
    UnsupportedSampleRate(u32),

    /// The requested buffer size is not supported.
    #[error("unsupported buffer size: {0} frames")]
    UnsupportedBufferSize(usize),

    /// The stream is not in the expected state.
    #[error("invalid stream state: expected {expected}, got {actual}")]
    InvalidStreamState {
        /// Expected state.
        expected: &'static str,
        /// Actual state.
        actual: &'static str,
    },

    /// An error occurred during stream initialization.
    #[error("stream initialization failed: {0}")]
    StreamInitError(String),

    /// An error occurred during audio I/O.
    #[error("I/O error: {0}")]
    IoError(String),

    /// Buffer overrun (output couldn't keep up).
    #[error("buffer overrun: audio callback took too long")]
    Overrun,

    /// Buffer underrun (input buffer was empty).
    #[error("buffer underrun: no audio data available")]
    Underrun,

    /// Platform-specific error.
    #[error("platform error ({code}): {message}")]
    PlatformError {
        /// Platform-specific error code.
        code: i32,
        /// Error message.
        message: String,
    },

    /// The audio backend is not available on this system.
    #[error("backend not available: {0}")]
    BackendNotAvailable(String),
}

#[cfg(target_os = "linux")]
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Phase 3 TDD: Comprehensive error tests
    // =========================================================================

    #[test]
    fn test_device_not_found_error() {
        let err = Error::DeviceNotFound("hw:0,0".to_string());
        let msg = format!("{}", err);

        assert!(msg.contains("device not found"));
        assert!(msg.contains("hw:0,0"));
    }

    #[test]
    fn test_device_busy_error() {
        let err = Error::DeviceBusy("USB Audio Interface".to_string());
        let msg = format!("{}", err);

        assert!(msg.contains("device busy"));
        assert!(msg.contains("USB Audio Interface"));
    }

    #[test]
    fn test_unsupported_config_error() {
        let err = Error::UnsupportedConfig("8 channels not supported".to_string());
        let msg = format!("{}", err);

        assert!(msg.contains("unsupported configuration"));
        assert!(msg.contains("8 channels"));
    }

    #[test]
    fn test_unsupported_sample_rate_error() {
        let err = Error::UnsupportedSampleRate(384000);
        let msg = format!("{}", err);

        assert!(msg.contains("unsupported sample rate"));
        assert!(msg.contains("384000"));
        assert!(msg.contains("Hz"));
    }

    #[test]
    fn test_unsupported_buffer_size_error() {
        let err = Error::UnsupportedBufferSize(16);
        let msg = format!("{}", err);

        assert!(msg.contains("unsupported buffer size"));
        assert!(msg.contains("16"));
        assert!(msg.contains("frames"));
    }

    #[test]
    fn test_invalid_stream_state_error() {
        let err = Error::InvalidStreamState {
            expected: "running",
            actual: "stopped",
        };
        let msg = format!("{}", err);

        assert!(msg.contains("invalid stream state"));
        assert!(msg.contains("running"));
        assert!(msg.contains("stopped"));
    }

    #[test]
    fn test_stream_init_error() {
        let err = Error::StreamInitError("failed to open PCM device".to_string());
        let msg = format!("{}", err);

        assert!(msg.contains("stream initialization failed"));
        assert!(msg.contains("failed to open PCM device"));
    }

    #[test]
    fn test_io_error() {
        let err = Error::IoError("connection reset".to_string());
        let msg = format!("{}", err);

        assert!(msg.contains("I/O error"));
        assert!(msg.contains("connection reset"));
    }

    #[test]
    fn test_overrun_error() {
        let err = Error::Overrun;
        let msg = format!("{}", err);

        assert!(msg.contains("buffer overrun"));
        assert!(msg.contains("callback took too long"));
    }

    #[test]
    fn test_underrun_error() {
        let err = Error::Underrun;
        let msg = format!("{}", err);

        assert!(msg.contains("buffer underrun"));
        assert!(msg.contains("no audio data"));
    }

    #[test]
    fn test_platform_error() {
        let err = Error::PlatformError {
            code: -22,
            message: "Invalid argument".to_string(),
        };
        let msg = format!("{}", err);

        assert!(msg.contains("platform error"));
        assert!(msg.contains("-22"));
        assert!(msg.contains("Invalid argument"));
    }

    #[test]
    fn test_backend_not_available_error() {
        let err = Error::BackendNotAvailable("WASAPI".to_string());
        let msg = format!("{}", err);

        assert!(msg.contains("backend not available"));
        assert!(msg.contains("WASAPI"));
    }

    // -------------------------------------------------------------------------
    // Error debug tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_error_debug() {
        let err = Error::DeviceNotFound("test".to_string());
        let debug = format!("{:?}", err);

        assert!(debug.contains("DeviceNotFound"));
        assert!(debug.contains("test"));
    }

    // -------------------------------------------------------------------------
    // Result type tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_result_ok() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_err() {
        let result: Result<i32> = Err(Error::Overrun);
        assert!(result.is_err());
    }

    #[test]
    fn test_result_map() {
        let result: Result<i32> = Ok(21);
        let doubled = result.map(|x| x * 2);
        assert_eq!(doubled.unwrap(), 42);
    }

    #[test]
    fn test_result_map_err() {
        let result: Result<i32> = Err(Error::Underrun);
        let mapped = result.map_err(|_| Error::Overrun);
        assert!(matches!(mapped, Err(Error::Overrun)));
    }

    // -------------------------------------------------------------------------
    // io::Error conversion tests (Linux only)
    // -------------------------------------------------------------------------

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::*;
        use std::io;

        #[test]
        fn test_from_io_error() {
            let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
            let hal_err: Error = io_err.into();

            match hal_err {
                Error::IoError(msg) => {
                    assert!(msg.contains("file not found"));
                }
                _ => panic!("Expected IoError variant"),
            }
        }

        #[test]
        fn test_from_io_error_permission() {
            let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
            let hal_err: Error = io_err.into();

            match hal_err {
                Error::IoError(msg) => {
                    assert!(msg.contains("access denied"));
                }
                _ => panic!("Expected IoError variant"),
            }
        }
    }

    // -------------------------------------------------------------------------
    // Error matching tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_error_match_device_not_found() {
        let err = Error::DeviceNotFound("test".to_string());

        match err {
            Error::DeviceNotFound(name) => assert_eq!(name, "test"),
            _ => panic!("Expected DeviceNotFound"),
        }
    }

    #[test]
    fn test_error_match_invalid_stream_state() {
        let err = Error::InvalidStreamState {
            expected: "running",
            actual: "paused",
        };

        match err {
            Error::InvalidStreamState { expected, actual } => {
                assert_eq!(expected, "running");
                assert_eq!(actual, "paused");
            }
            _ => panic!("Expected InvalidStreamState"),
        }
    }

    #[test]
    fn test_error_match_platform_error() {
        let err = Error::PlatformError {
            code: -1,
            message: "Unknown error".to_string(),
        };

        match err {
            Error::PlatformError { code, message } => {
                assert_eq!(code, -1);
                assert_eq!(message, "Unknown error");
            }
            _ => panic!("Expected PlatformError"),
        }
    }

    // -------------------------------------------------------------------------
    // Common error scenarios
    // -------------------------------------------------------------------------

    #[test]
    fn test_common_alsa_errors() {
        // Simulate common ALSA error codes
        let errors = [
            Error::PlatformError {
                code: -19,
                message: "No such device".to_string(),
            },
            Error::PlatformError {
                code: -16,
                message: "Device or resource busy".to_string(),
            },
            Error::PlatformError {
                code: -32,
                message: "Broken pipe (stream stopped)".to_string(),
            },
        ];

        for err in errors {
            let msg = format!("{}", err);
            assert!(msg.contains("platform error"));
        }
    }

    #[test]
    fn test_common_sample_rate_errors() {
        let unsupported_rates = [22050, 176400, 352800, 768000];

        for rate in unsupported_rates {
            let err = Error::UnsupportedSampleRate(rate);
            let msg = format!("{}", err);
            assert!(msg.contains(&rate.to_string()));
        }
    }

    #[test]
    fn test_common_buffer_size_errors() {
        let unsupported_sizes = [8, 16, 8192, 16384];

        for size in unsupported_sizes {
            let err = Error::UnsupportedBufferSize(size);
            let msg = format!("{}", err);
            assert!(msg.contains(&size.to_string()));
        }
    }
}
