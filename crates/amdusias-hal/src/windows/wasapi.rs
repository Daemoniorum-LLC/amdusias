//! WASAPI backend for Windows.
//!
//! Exclusive mode implementation for minimal latency.

use crate::{
    config::StreamConfig,
    device::{DeviceId, DeviceInfo},
    error::Result,
    stream::AudioStream,
    traits::{AudioBackend, AudioCallback, DuplexCallback, InputCallback},
    Error,
};

/// WASAPI audio backend.
pub struct WasapiBackend {
    // COM initialization state
}

impl WasapiBackend {
    /// Creates a new WASAPI backend.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WasapiBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// WASAPI output stream.
pub struct WasapiOutputStream {
    config: StreamConfig,
}

impl AudioStream for WasapiOutputStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("WASAPI not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

/// WASAPI input stream.
pub struct WasapiInputStream {
    config: StreamConfig,
}

impl AudioStream for WasapiInputStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("WASAPI not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

/// WASAPI duplex stream.
pub struct WasapiDuplexStream {
    config: StreamConfig,
}

impl AudioStream for WasapiDuplexStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("WASAPI not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

impl AudioBackend for WasapiBackend {
    type OutputStream = WasapiOutputStream;
    type InputStream = WasapiInputStream;
    type DuplexStream = WasapiDuplexStream;

    fn name(&self) -> &'static str {
        "WASAPI"
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
        Ok(WasapiOutputStream { config })
    }

    fn open_input<C: InputCallback>(
        &self,
        _device: &DeviceId,
        config: StreamConfig,
        _callback: C,
    ) -> Result<Self::InputStream> {
        Ok(WasapiInputStream { config })
    }

    fn open_duplex<C: DuplexCallback>(
        &self,
        _input_device: &DeviceId,
        _output_device: &DeviceId,
        config: StreamConfig,
        _callback: C,
    ) -> Result<Self::DuplexStream> {
        Ok(WasapiDuplexStream { config })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::{CallbackInfo, StreamState};

    // =========================================================================
    // Phase 3b TDD: WASAPI Backend Tests (Windows)
    // =========================================================================

    // -------------------------------------------------------------------------
    // WasapiBackend creation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_wasapi_backend_new() {
        let backend = WasapiBackend::new();
        assert_eq!(backend.name(), "WASAPI");
    }

    #[test]
    fn test_wasapi_backend_default() {
        let backend = WasapiBackend::default();
        assert_eq!(backend.name(), "WASAPI");
    }

    #[test]
    fn test_wasapi_backend_name() {
        let backend = WasapiBackend::new();
        assert_eq!(backend.name(), "WASAPI");
    }

    // -------------------------------------------------------------------------
    // Device enumeration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_wasapi_enumerate_devices() {
        let backend = WasapiBackend::new();
        let devices = backend.enumerate_devices();

        assert!(devices.is_ok());
        assert!(devices.unwrap().is_empty());
    }

    #[test]
    fn test_wasapi_enumerate_output_devices() {
        let backend = WasapiBackend::new();
        let devices = backend.enumerate_output_devices();

        assert!(devices.is_ok());
    }

    #[test]
    fn test_wasapi_enumerate_input_devices() {
        let backend = WasapiBackend::new();
        let devices = backend.enumerate_input_devices();

        assert!(devices.is_ok());
    }

    #[test]
    fn test_wasapi_default_output_device_not_found() {
        let backend = WasapiBackend::new();
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
    fn test_wasapi_default_input_device_not_found() {
        let backend = WasapiBackend::new();
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
    fn test_wasapi_open_output_stream() {
        let backend = WasapiBackend::new();
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
    fn test_wasapi_output_stream_state() {
        let backend = WasapiBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
        assert!(!stream.state().is_active());
    }

    #[test]
    fn test_wasapi_output_stream_start_not_implemented() {
        let backend = WasapiBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let mut stream = backend.open_output(&device_id, config, callback).unwrap();

        let result = stream.start();
        assert!(result.is_err());
        match result {
            Err(Error::BackendNotAvailable(msg)) => {
                assert!(msg.contains("WASAPI"));
            }
            _ => panic!("Expected BackendNotAvailable error"),
        }
    }

    #[test]
    fn test_wasapi_output_stream_stop() {
        let backend = WasapiBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let mut stream = backend.open_output(&device_id, config, callback).unwrap();

        assert!(stream.stop().is_ok());
    }

    #[test]
    fn test_wasapi_output_stream_latency() {
        let backend = WasapiBackend::new();
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
    fn test_wasapi_open_input_stream() {
        let backend = WasapiBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_data: &[f32], _info: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config, callback);

        assert!(stream.is_ok());
    }

    #[test]
    fn test_wasapi_input_stream_state() {
        let backend = WasapiBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("default");

        let callback = |_: &[f32], _: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config, callback).unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
    }

    #[test]
    fn test_wasapi_input_stream_start_not_implemented() {
        let backend = WasapiBackend::new();
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
    fn test_wasapi_open_duplex_stream() {
        let backend = WasapiBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let input_device = DeviceId::new("input");
        let output_device = DeviceId::new("output");

        let callback = |_input: &[f32], _output: &mut [f32], _info: &CallbackInfo| {};
        let stream = backend.open_duplex(&input_device, &output_device, config, callback);

        assert!(stream.is_ok());
    }

    #[test]
    fn test_wasapi_duplex_stream_state() {
        let backend = WasapiBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let input_device = DeviceId::new("input");
        let output_device = DeviceId::new("output");

        let callback = |_: &[f32], _: &mut [f32], _: &CallbackInfo| {};
        let stream = backend
            .open_duplex(&input_device, &output_device, config, callback)
            .unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
    }

    // -------------------------------------------------------------------------
    // Configuration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_wasapi_various_sample_rates() {
        let backend = WasapiBackend::new();
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
    fn test_wasapi_various_buffer_sizes() {
        let backend = WasapiBackend::new();
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
    fn test_wasapi_exclusive_mode() {
        let backend = WasapiBackend::new();
        let device_id = DeviceId::new("default");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        // Exclusive mode (default for professional audio)
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(true);
        let stream = backend.open_output(&device_id, config, callback).unwrap();
        assert!(stream.config().exclusive);

        // Shared mode
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(false);
        let stream = backend.open_output(&device_id, config, callback).unwrap();
        assert!(!stream.config().exclusive);
    }
}
