//! # amdusias-hal
//!
//! Hardware Abstraction Layer for the Amdusias audio engine.
//!
//! This crate provides platform-agnostic interfaces to audio hardware,
//! with native implementations for:
//!
//! - **Linux**: ALSA (direct), PipeWire
//! - **Windows**: WASAPI (exclusive mode for low latency)
//! - **macOS**: CoreAudio (AudioUnit)
//!
//! ## Design Philosophy
//!
//! Unlike `cpal` or other audio libraries, `amdusias-hal` is designed for
//! **professional audio applications** with these priorities:
//!
//! 1. **Minimal latency** - Exclusive mode, direct hardware access
//! 2. **No allocations in callbacks** - All buffers pre-allocated
//! 3. **Full hardware control** - Buffer sizes, sample rates, device selection
//! 4. **No hidden threads** - You control the audio thread
//!
//! ## Example
//!
//! ```rust,ignore
//! use amdusias_hal::{AudioBackend, StreamConfig};
//!
//! // Get the platform-specific backend
//! let backend = AudioBackend::default();
//!
//! // Enumerate devices
//! for device in backend.enumerate_output_devices() {
//!     println!("{}: {}", device.id, device.name);
//! }
//!
//! // Open an output stream
//! let config = StreamConfig {
//!     sample_rate: 48000,
//!     buffer_size: 256,
//!     channels: 2,
//! };
//!
//! let stream = backend.open_output(config, |data, info| {
//!     // Fill the buffer with audio data
//!     // This callback runs on a real-time thread
//! })?;
//!
//! stream.start()?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod device;
pub mod error;
pub mod stream;
pub mod traits;

// Platform-specific backends
#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

pub use config::StreamConfig;
pub use device::{DeviceId, DeviceInfo, DeviceType};
pub use error::{Error, Result};
pub use stream::{AudioStream, StreamState};
pub use traits::{AudioBackend, AudioCallback, DuplexCallback, InputCallback};

/// Returns the default audio backend for the current platform.
#[must_use]
pub fn default_backend() -> impl AudioBackend {
    #[cfg(target_os = "linux")]
    {
        linux::AlsaBackend::new()
    }

    #[cfg(target_os = "windows")]
    {
        windows::WasapiBackend::new()
    }

    #[cfg(target_os = "macos")]
    {
        macos::CoreAudioBackend::new()
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        compile_error!("Unsupported platform")
    }
}
