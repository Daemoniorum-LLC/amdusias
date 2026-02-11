//! Error types for amdusias-core.

use thiserror::Error;

/// Result type alias for amdusias-core operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur in core audio operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Buffer size mismatch between input and output.
    #[error("buffer size mismatch: expected {expected}, got {actual}")]
    BufferSizeMismatch {
        /// Expected buffer size.
        expected: usize,
        /// Actual buffer size.
        actual: usize,
    },

    /// Sample rate mismatch between components.
    #[error("sample rate mismatch: expected {expected}Hz, got {actual}Hz")]
    SampleRateMismatch {
        /// Expected sample rate in Hz.
        expected: u32,
        /// Actual sample rate in Hz.
        actual: u32,
    },

    /// Channel count mismatch.
    #[error("channel count mismatch: expected {expected}, got {actual}")]
    ChannelMismatch {
        /// Expected channel count.
        expected: usize,
        /// Actual channel count.
        actual: usize,
    },

    /// Queue is full, cannot push more items.
    #[error("queue is full")]
    QueueFull,

    /// Queue is empty, cannot pop items.
    #[error("queue is empty")]
    QueueEmpty,

    /// Invalid buffer alignment for SIMD operations.
    #[error("buffer not aligned to {required} bytes")]
    AlignmentError {
        /// Required alignment in bytes.
        required: usize,
    },

    /// Scheduler event is in the past.
    #[error("cannot schedule event in the past: {position} < {current}")]
    EventInPast {
        /// Requested position.
        position: u64,
        /// Current position.
        current: u64,
    },
}
