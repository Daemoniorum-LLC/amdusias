//! Core traits for audio backends and callbacks.

use crate::{
    config::StreamConfig,
    device::DeviceInfo,
    stream::{AudioStream, CallbackInfo},
    DeviceId, Result,
};

/// Callback function type for audio output.
///
/// # Arguments
///
/// - `data`: Mutable slice of interleaved audio samples to fill.
/// - `info`: Information about the current callback (timing, etc.).
///
/// # Real-time Constraints
///
/// This callback runs on a real-time audio thread. You MUST NOT:
/// - Allocate memory
/// - Acquire locks (use lock-free structures)
/// - Perform I/O operations
/// - Call system functions that may block
pub trait AudioCallback: Send + 'static {
    /// Called to fill the output buffer with audio data.
    fn process(&mut self, data: &mut [f32], info: &CallbackInfo);

    /// Called when an error occurs in the audio stream.
    fn on_error(&mut self, _error: &crate::Error) {}
}

/// Callback for audio input.
pub trait InputCallback: Send + 'static {
    /// Called when input data is available.
    fn process(&mut self, data: &[f32], info: &CallbackInfo);

    /// Called when an error occurs in the audio stream.
    fn on_error(&mut self, _error: &crate::Error) {}
}

/// Callback for duplex (simultaneous input/output) streams.
pub trait DuplexCallback: Send + 'static {
    /// Called to process input and produce output.
    ///
    /// # Arguments
    ///
    /// - `input`: Input samples (read-only).
    /// - `output`: Output buffer to fill.
    /// - `info`: Callback information.
    fn process(&mut self, input: &[f32], output: &mut [f32], info: &CallbackInfo);

    /// Called when an error occurs.
    fn on_error(&mut self, _error: &crate::Error) {}
}

/// Implement `AudioCallback` for closures.
impl<F> AudioCallback for F
where
    F: FnMut(&mut [f32], &CallbackInfo) + Send + 'static,
{
    fn process(&mut self, data: &mut [f32], info: &CallbackInfo) {
        self(data, info);
    }
}

/// Implement `InputCallback` for closures.
impl<F> InputCallback for F
where
    F: FnMut(&[f32], &CallbackInfo) + Send + 'static,
{
    fn process(&mut self, data: &[f32], info: &CallbackInfo) {
        self(data, info);
    }
}

/// Implement `DuplexCallback` for closures.
impl<F> DuplexCallback for F
where
    F: FnMut(&[f32], &mut [f32], &CallbackInfo) + Send + 'static,
{
    fn process(&mut self, input: &[f32], output: &mut [f32], info: &CallbackInfo) {
        self(input, output, info);
    }
}

/// Trait for platform-specific audio backends.
pub trait AudioBackend: Send + Sync {
    /// The output stream type for this backend.
    type OutputStream: AudioStream;
    /// The input stream type for this backend.
    type InputStream: AudioStream;
    /// The duplex stream type for this backend.
    type DuplexStream: AudioStream;

    /// Returns the name of this backend (e.g., "ALSA", "WASAPI", "CoreAudio").
    fn name(&self) -> &'static str;

    /// Enumerates all available audio devices.
    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>>;

    /// Enumerates output devices only.
    fn enumerate_output_devices(&self) -> Result<Vec<DeviceInfo>> {
        Ok(self
            .enumerate_devices()?
            .into_iter()
            .filter(|d| d.supports_output())
            .collect())
    }

    /// Enumerates input devices only.
    fn enumerate_input_devices(&self) -> Result<Vec<DeviceInfo>> {
        Ok(self
            .enumerate_devices()?
            .into_iter()
            .filter(|d| d.supports_input())
            .collect())
    }

    /// Returns the default output device.
    fn default_output_device(&self) -> Result<DeviceInfo>;

    /// Returns the default input device.
    fn default_input_device(&self) -> Result<DeviceInfo>;

    /// Opens an output stream with the specified callback.
    fn open_output<C: AudioCallback>(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        callback: C,
    ) -> Result<Self::OutputStream>;

    /// Opens an output stream on the default device.
    fn open_default_output<C: AudioCallback>(
        &self,
        config: StreamConfig,
        callback: C,
    ) -> Result<Self::OutputStream> {
        let device = self.default_output_device()?;
        self.open_output(&device.id, config, callback)
    }

    /// Opens an input stream with the specified callback.
    fn open_input<C: InputCallback>(
        &self,
        device: &DeviceId,
        config: StreamConfig,
        callback: C,
    ) -> Result<Self::InputStream>;

    /// Opens an input stream on the default device.
    fn open_default_input<C: InputCallback>(
        &self,
        config: StreamConfig,
        callback: C,
    ) -> Result<Self::InputStream> {
        let device = self.default_input_device()?;
        self.open_input(&device.id, config, callback)
    }

    /// Opens a duplex (input + output) stream.
    fn open_duplex<C: DuplexCallback>(
        &self,
        input_device: &DeviceId,
        output_device: &DeviceId,
        config: StreamConfig,
        callback: C,
    ) -> Result<Self::DuplexStream>;
}

/// Marker trait for backends that support exclusive mode.
pub trait ExclusiveMode: AudioBackend {
    /// Returns true if exclusive mode is available on this system.
    fn exclusive_mode_available(&self) -> bool;
}

/// Marker trait for backends that support hot-plugging.
pub trait HotPlug: AudioBackend {
    /// Registers a callback to be called when devices are added or removed.
    fn register_device_change_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + 'static;
}
