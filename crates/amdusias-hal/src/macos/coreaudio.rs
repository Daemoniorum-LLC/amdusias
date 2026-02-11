//! CoreAudio backend for macOS.
//!
//! AudioUnit-based implementation for low latency.

use crate::{
    config::StreamConfig,
    device::{DeviceId, DeviceInfo},
    error::Result,
    stream::AudioStream,
    traits::{AudioBackend, AudioCallback, DuplexCallback, InputCallback},
    Error,
};

/// CoreAudio backend.
pub struct CoreAudioBackend {
    // AudioComponent state
}

impl CoreAudioBackend {
    /// Creates a new CoreAudio backend.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CoreAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// CoreAudio output stream.
pub struct CoreAudioOutputStream {
    config: StreamConfig,
}

impl AudioStream for CoreAudioOutputStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("CoreAudio not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

/// CoreAudio input stream.
pub struct CoreAudioInputStream {
    config: StreamConfig,
}

impl AudioStream for CoreAudioInputStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("CoreAudio not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

/// CoreAudio duplex stream.
pub struct CoreAudioDuplexStream {
    config: StreamConfig,
}

impl AudioStream for CoreAudioDuplexStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("CoreAudio not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

impl AudioBackend for CoreAudioBackend {
    type OutputStream = CoreAudioOutputStream;
    type InputStream = CoreAudioInputStream;
    type DuplexStream = CoreAudioDuplexStream;

    fn name(&self) -> &'static str {
        "CoreAudio"
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>> {
        Ok(Vec::new())
    }

    fn default_output_device(&self) -> Result<DeviceInfo> {
        Err(Error::DeviceNotFound("No default output device".into()))
    }

    fn default_input_device(&self) -> Result<DeviceInfo> {
        Err(Error::DeviceNotFound("No default input device".into()))
    }

    fn open_output<C: AudioCallback>(
        &self,
        _device: &DeviceId,
        config: StreamConfig,
        _callback: C,
    ) -> Result<Self::OutputStream> {
        Ok(CoreAudioOutputStream { config })
    }

    fn open_input<C: InputCallback>(
        &self,
        _device: &DeviceId,
        config: StreamConfig,
        _callback: C,
    ) -> Result<Self::InputStream> {
        Ok(CoreAudioInputStream { config })
    }

    fn open_duplex<C: DuplexCallback>(
        &self,
        _input_device: &DeviceId,
        _output_device: &DeviceId,
        config: StreamConfig,
        _callback: C,
    ) -> Result<Self::DuplexStream> {
        Ok(CoreAudioDuplexStream { config })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::{CallbackInfo, StreamState};

    // =========================================================================
    // Phase 3b TDD: CoreAudio Backend Tests (macOS)
    // =========================================================================

    // -------------------------------------------------------------------------
    // CoreAudioBackend creation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coreaudio_backend_new() {
        let backend = CoreAudioBackend::new();
        assert_eq!(backend.name(), "CoreAudio");
    }

    #[test]
    fn test_coreaudio_backend_default() {
        let backend = CoreAudioBackend::default();
        assert_eq!(backend.name(), "CoreAudio");
    }

    #[test]
    fn test_coreaudio_backend_name() {
        let backend = CoreAudioBackend::new();
        assert_eq!(backend.name(), "CoreAudio");
    }

    // -------------------------------------------------------------------------
    // Device enumeration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coreaudio_enumerate_devices() {
        let backend = CoreAudioBackend::new();
        let devices = backend.enumerate_devices();

        assert!(devices.is_ok());
        assert!(devices.unwrap().is_empty());
    }

    #[test]
    fn test_coreaudio_enumerate_output_devices() {
        let backend = CoreAudioBackend::new();
        let devices = backend.enumerate_output_devices();

        assert!(devices.is_ok());
    }

    #[test]
    fn test_coreaudio_enumerate_input_devices() {
        let backend = CoreAudioBackend::new();
        let devices = backend.enumerate_input_devices();

        assert!(devices.is_ok());
    }

    #[test]
    fn test_coreaudio_default_output_device_not_found() {
        let backend = CoreAudioBackend::new();
        let result = backend.default_output_device();

        assert!(result.is_err());
        match result {
            Err(Error::DeviceNotFound(msg)) => {
                assert!(msg.contains("default output"));
            }
            _ => panic!("Expected DeviceNotFound error"),
        }
    }

    #[test]
    fn test_coreaudio_default_input_device_not_found() {
        let backend = CoreAudioBackend::new();
        let result = backend.default_input_device();

        assert!(result.is_err());
        match result {
            Err(Error::DeviceNotFound(msg)) => {
                assert!(msg.contains("default input"));
            }
            _ => panic!("Expected DeviceNotFound error"),
        }
    }

    // -------------------------------------------------------------------------
    // Output stream tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coreaudio_open_output_stream() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_data: &mut [f32], _info: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config.clone(), callback);

        assert!(stream.is_ok());
        let stream = stream.unwrap();
        assert_eq!(stream.config().sample_rate, 48000);
        assert_eq!(stream.config().buffer_size, 512);
        assert_eq!(stream.config().channels, 2);
    }

    #[test]
    fn test_coreaudio_output_stream_state() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
        assert!(!stream.state().is_active());
    }

    #[test]
    fn test_coreaudio_output_stream_start_not_implemented() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let mut stream = backend.open_output(&device_id, config, callback).unwrap();

        let result = stream.start();
        assert!(result.is_err());
        match result {
            Err(Error::BackendNotAvailable(msg)) => {
                assert!(msg.contains("CoreAudio"));
            }
            _ => panic!("Expected BackendNotAvailable error"),
        }
    }

    #[test]
    fn test_coreaudio_output_stream_stop() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let mut stream = backend.open_output(&device_id, config, callback).unwrap();

        assert!(stream.stop().is_ok());
    }

    #[test]
    fn test_coreaudio_output_stream_latency() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        assert_eq!(stream.latency_samples(), 1024);
    }

    // -------------------------------------------------------------------------
    // Input stream tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coreaudio_open_input_stream() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_data: &[f32], _info: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config, callback);

        assert!(stream.is_ok());
    }

    #[test]
    fn test_coreaudio_input_stream_state() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &[f32], _: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config, callback).unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
    }

    #[test]
    fn test_coreaudio_input_stream_start_not_implemented() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &[f32], _: &CallbackInfo| {};
        let mut stream = backend.open_input(&device_id, config, callback).unwrap();

        let result = stream.start();
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // Duplex stream tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coreaudio_open_duplex_stream() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let input_device = DeviceId::new("input");
        let output_device = DeviceId::new("output");

        let callback = |_input: &[f32], _output: &mut [f32], _info: &CallbackInfo| {};
        let stream = backend.open_duplex(&input_device, &output_device, config, callback);

        assert!(stream.is_ok());
    }

    #[test]
    fn test_coreaudio_duplex_stream_state() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let input_device = DeviceId::new("input");
        let output_device = DeviceId::new("output");

        let callback = |_: &[f32], _: &mut [f32], _: &CallbackInfo| {};
        let stream = backend
            .open_duplex(&input_device, &output_device, config, callback)
            .unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
    }

    #[test]
    fn test_coreaudio_duplex_stream_latency() {
        let backend = CoreAudioBackend::new();
        let config = StreamConfig::new(48000, 256, 2);
        let input_device = DeviceId::new("input");
        let output_device = DeviceId::new("output");

        let callback = |_: &[f32], _: &mut [f32], _: &CallbackInfo| {};
        let stream = backend
            .open_duplex(&input_device, &output_device, config, callback)
            .unwrap();

        // Double buffering: 256 * 2 = 512
        assert_eq!(stream.latency_samples(), 512);
    }

    // -------------------------------------------------------------------------
    // Configuration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coreaudio_various_sample_rates() {
        let backend = CoreAudioBackend::new();
        let device_id = DeviceId::new("default");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        for rate in [44100, 48000, 96000, 192000] {
            let config = StreamConfig::new(rate, 512, 2);
            let stream = backend.open_output(&device_id, config, callback);
            assert!(stream.is_ok());
            assert_eq!(stream.unwrap().config().sample_rate, rate);
        }
    }

    #[test]
    fn test_coreaudio_various_buffer_sizes() {
        let backend = CoreAudioBackend::new();
        let device_id = DeviceId::new("default");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        for size in [128, 256, 512, 1024, 2048] {
            let config = StreamConfig::new(48000, size, 2);
            let stream = backend.open_output(&device_id, config, callback);
            assert!(stream.is_ok());
            assert_eq!(stream.unwrap().config().buffer_size, size);
        }
    }

    #[test]
    fn test_coreaudio_exclusive_mode() {
        let backend = CoreAudioBackend::new();
        let device_id = DeviceId::new("default");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        // Exclusive mode (hog mode on macOS)
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(true);
        let stream = backend.open_output(&device_id, config, callback).unwrap();
        assert!(stream.config().exclusive);

        // Shared mode
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(false);
        let stream = backend.open_output(&device_id, config, callback).unwrap();
        assert!(!stream.config().exclusive);
    }

    #[test]
    fn test_coreaudio_channel_configurations() {
        let backend = CoreAudioBackend::new();
        let device_id = DeviceId::new("default");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        // Test various channel counts common on macOS
        for channels in [1, 2, 6, 8] {
            let config = StreamConfig::new(48000, 512, channels);
            let stream = backend.open_output(&device_id, config, callback);
            assert!(stream.is_ok());
            assert_eq!(stream.unwrap().config().channels, channels);
        }
    }

    // -------------------------------------------------------------------------
    // Latency calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_coreaudio_latency_secs() {
        let backend = CoreAudioBackend::new();
        let device_id = DeviceId::new("default");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        let config = StreamConfig::new(48000, 512, 2);
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        // 1024 samples at 48kHz = ~21.33ms
        let latency_secs = stream.latency_secs();
        let expected = 1024.0 / 48000.0;
        assert!((latency_secs - expected).abs() < 0.0001);
    }

    #[test]
    fn test_coreaudio_low_latency_config() {
        let backend = CoreAudioBackend::new();
        let device_id = DeviceId::new("default");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        // Low latency: 64 samples at 96kHz
        let config = StreamConfig::new(96000, 64, 2);
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        // 128 samples at 96kHz = ~1.33ms
        let latency_ms = stream.latency_secs() * 1000.0;
        assert!(latency_ms < 2.0, "Expected <2ms latency, got {}ms", latency_ms);
    }
}
