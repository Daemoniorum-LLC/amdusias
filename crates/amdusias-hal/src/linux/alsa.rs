//! ALSA backend for Linux.
//!
//! Direct ALSA implementation bypassing PulseAudio for minimal latency.

use crate::{
    config::StreamConfig,
    device::{DeviceId, DeviceInfo},
    error::Result,
    stream::AudioStream,
    traits::{AudioBackend, AudioCallback, DuplexCallback, InputCallback},
    Error,
};

/// ALSA audio backend.
pub struct AlsaBackend {
    // Backend state will be added during implementation
}

impl AlsaBackend {
    /// Creates a new ALSA backend.
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AlsaBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// ALSA output stream.
pub struct AlsaOutputStream {
    config: StreamConfig,
    // PCM handle and state will be added during implementation
}

impl AudioStream for AlsaOutputStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        // TODO: Implement ALSA stream start
        Err(Error::BackendNotAvailable("ALSA not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2 // Estimate: 2 buffer periods
    }
}

/// ALSA input stream.
pub struct AlsaInputStream {
    config: StreamConfig,
}

impl AudioStream for AlsaInputStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("ALSA not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

/// ALSA duplex stream.
pub struct AlsaDuplexStream {
    config: StreamConfig,
}

impl AudioStream for AlsaDuplexStream {
    fn config(&self) -> &StreamConfig {
        &self.config
    }

    fn state(&self) -> crate::stream::StreamState {
        crate::stream::StreamState::Stopped
    }

    fn start(&mut self) -> Result<()> {
        Err(Error::BackendNotAvailable("ALSA not yet implemented".into()))
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn latency_samples(&self) -> usize {
        self.config.buffer_size * 2
    }
}

impl AudioBackend for AlsaBackend {
    type OutputStream = AlsaOutputStream;
    type InputStream = AlsaInputStream;
    type DuplexStream = AlsaDuplexStream;

    fn name(&self) -> &'static str {
        "ALSA"
    }

    fn enumerate_devices(&self) -> Result<Vec<DeviceInfo>> {
        // TODO: Enumerate ALSA devices using snd_device_name_hint
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
        Ok(AlsaOutputStream { config })
    }

    fn open_input<C: InputCallback>(
        &self,
        _device: &DeviceId,
        config: StreamConfig,
        _callback: C,
    ) -> Result<Self::InputStream> {
        Ok(AlsaInputStream { config })
    }

    fn open_duplex<C: DuplexCallback>(
        &self,
        _input_device: &DeviceId,
        _output_device: &DeviceId,
        config: StreamConfig,
        _callback: C,
    ) -> Result<Self::DuplexStream> {
        Ok(AlsaDuplexStream { config })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::{CallbackInfo, StreamState};

    // =========================================================================
    // Phase 3b TDD: ALSA Backend Tests (Linux)
    // =========================================================================

    // -------------------------------------------------------------------------
    // AlsaBackend creation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_alsa_backend_new() {
        let backend = AlsaBackend::new();
        assert_eq!(backend.name(), "ALSA");
    }

    #[test]
    fn test_alsa_backend_default() {
        let backend = AlsaBackend::default();
        assert_eq!(backend.name(), "ALSA");
    }

    #[test]
    fn test_alsa_backend_name() {
        let backend = AlsaBackend::new();
        assert_eq!(backend.name(), "ALSA");
    }

    // -------------------------------------------------------------------------
    // Device enumeration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_alsa_enumerate_devices() {
        let backend = AlsaBackend::new();
        let devices = backend.enumerate_devices();

        // Currently returns empty vec (stub)
        assert!(devices.is_ok());
        let devices = devices.unwrap();
        // Empty for now, but should not error
        assert!(devices.is_empty());
    }

    #[test]
    fn test_alsa_enumerate_output_devices() {
        let backend = AlsaBackend::new();
        let devices = backend.enumerate_output_devices();

        assert!(devices.is_ok());
    }

    #[test]
    fn test_alsa_enumerate_input_devices() {
        let backend = AlsaBackend::new();
        let devices = backend.enumerate_input_devices();

        assert!(devices.is_ok());
    }

    #[test]
    fn test_alsa_default_output_device_not_found() {
        let backend = AlsaBackend::new();
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
    fn test_alsa_default_input_device_not_found() {
        let backend = AlsaBackend::new();
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
    fn test_alsa_open_output_stream() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_data: &mut [f32], _info: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config.clone(), callback);

        assert!(stream.is_ok());
        let stream = stream.unwrap();
        assert_eq!(stream.config().sample_rate, 48000);
        assert_eq!(stream.config().buffer_size, 512);
        assert_eq!(stream.config().channels, 2);
    }

    #[test]
    fn test_alsa_output_stream_config() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(44100, 256, 1);
        let device_id = DeviceId::new("default");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        assert_eq!(stream.config().sample_rate, 44100);
        assert_eq!(stream.config().buffer_size, 256);
        assert_eq!(stream.config().channels, 1);
    }

    #[test]
    fn test_alsa_output_stream_state() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        // Initial state is stopped
        assert_eq!(stream.state(), StreamState::Stopped);
        assert!(!stream.state().is_active());
    }

    #[test]
    fn test_alsa_output_stream_start_not_implemented() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let mut stream = backend.open_output(&device_id, config, callback).unwrap();

        let result = stream.start();
        assert!(result.is_err());
        match result {
            Err(Error::BackendNotAvailable(msg)) => {
                assert!(msg.contains("ALSA"));
            }
            _ => panic!("Expected BackendNotAvailable error"),
        }
    }

    #[test]
    fn test_alsa_output_stream_stop() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let mut stream = backend.open_output(&device_id, config, callback).unwrap();

        // Stop should succeed
        let result = stream.stop();
        assert!(result.is_ok());
    }

    #[test]
    fn test_alsa_output_stream_latency() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_: &mut [f32], _: &CallbackInfo| {};
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        // Estimate: 2 buffer periods
        assert_eq!(stream.latency_samples(), 1024);

        // Latency in seconds
        let latency_secs = stream.latency_secs();
        assert!((latency_secs - (1024.0 / 48000.0)).abs() < 0.0001);
    }

    // -------------------------------------------------------------------------
    // Input stream tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_alsa_open_input_stream() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_data: &[f32], _info: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config.clone(), callback);

        assert!(stream.is_ok());
    }

    #[test]
    fn test_alsa_input_stream_config() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(96000, 128, 2);
        let device_id = DeviceId::new("hw:1,0");

        let callback = |_: &[f32], _: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config, callback).unwrap();

        assert_eq!(stream.config().sample_rate, 96000);
        assert_eq!(stream.config().buffer_size, 128);
    }

    #[test]
    fn test_alsa_input_stream_state() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_: &[f32], _: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config, callback).unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
    }

    #[test]
    fn test_alsa_input_stream_start_not_implemented() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_: &[f32], _: &CallbackInfo| {};
        let mut stream = backend.open_input(&device_id, config, callback).unwrap();

        let result = stream.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_alsa_input_stream_latency() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 256, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |_: &[f32], _: &CallbackInfo| {};
        let stream = backend.open_input(&device_id, config, callback).unwrap();

        assert_eq!(stream.latency_samples(), 512); // 256 * 2
    }

    // -------------------------------------------------------------------------
    // Duplex stream tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_alsa_open_duplex_stream() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let input_device = DeviceId::new("hw:0,0");
        let output_device = DeviceId::new("hw:0,0");

        let callback = |_input: &[f32], _output: &mut [f32], _info: &CallbackInfo| {};
        let stream = backend.open_duplex(&input_device, &output_device, config, callback);

        assert!(stream.is_ok());
    }

    #[test]
    fn test_alsa_duplex_stream_config() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 64, 2);
        let input_device = DeviceId::new("hw:0,0");
        let output_device = DeviceId::new("hw:1,0");

        let callback = |_: &[f32], _: &mut [f32], _: &CallbackInfo| {};
        let stream = backend
            .open_duplex(&input_device, &output_device, config, callback)
            .unwrap();

        assert_eq!(stream.config().sample_rate, 48000);
        assert_eq!(stream.config().buffer_size, 64);
    }

    #[test]
    fn test_alsa_duplex_stream_state() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let input_device = DeviceId::new("hw:0,0");
        let output_device = DeviceId::new("hw:0,0");

        let callback = |_: &[f32], _: &mut [f32], _: &CallbackInfo| {};
        let stream = backend
            .open_duplex(&input_device, &output_device, config, callback)
            .unwrap();

        assert_eq!(stream.state(), StreamState::Stopped);
    }

    #[test]
    fn test_alsa_duplex_stream_start_not_implemented() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let input_device = DeviceId::new("hw:0,0");
        let output_device = DeviceId::new("hw:0,0");

        let callback = |_: &[f32], _: &mut [f32], _: &CallbackInfo| {};
        let mut stream = backend
            .open_duplex(&input_device, &output_device, config, callback)
            .unwrap();

        let result = stream.start();
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // Callback trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_alsa_closure_as_audio_callback() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let mut counter = 0u32;
        let callback = move |data: &mut [f32], _info: &CallbackInfo| {
            counter += 1;
            for sample in data.iter_mut() {
                *sample = 0.0;
            }
        };

        let stream = backend.open_output(&device_id, config, callback);
        assert!(stream.is_ok());
    }

    #[test]
    fn test_alsa_closure_as_input_callback() {
        let backend = AlsaBackend::new();
        let config = StreamConfig::new(48000, 512, 2);
        let device_id = DeviceId::new("hw:0,0");

        let callback = |data: &[f32], _info: &CallbackInfo| {
            let _sum: f32 = data.iter().sum();
        };

        let stream = backend.open_input(&device_id, config, callback);
        assert!(stream.is_ok());
    }

    // -------------------------------------------------------------------------
    // Configuration variation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_alsa_various_sample_rates() {
        let backend = AlsaBackend::new();
        let device_id = DeviceId::new("hw:0,0");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        for rate in [22050, 44100, 48000, 88200, 96000, 192000] {
            let config = StreamConfig::new(rate, 512, 2);
            let stream = backend.open_output(&device_id, config, callback);
            assert!(stream.is_ok(), "Failed for sample rate {}", rate);
            assert_eq!(stream.unwrap().config().sample_rate, rate);
        }
    }

    #[test]
    fn test_alsa_various_buffer_sizes() {
        let backend = AlsaBackend::new();
        let device_id = DeviceId::new("hw:0,0");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        for size in [32, 64, 128, 256, 512, 1024, 2048, 4096] {
            let config = StreamConfig::new(48000, size, 2);
            let stream = backend.open_output(&device_id, config, callback);
            assert!(stream.is_ok(), "Failed for buffer size {}", size);
            assert_eq!(stream.unwrap().config().buffer_size, size);
        }
    }

    #[test]
    fn test_alsa_various_channel_counts() {
        let backend = AlsaBackend::new();
        let device_id = DeviceId::new("hw:0,0");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        for channels in [1, 2, 4, 6, 8] {
            let config = StreamConfig::new(48000, 512, channels);
            let stream = backend.open_output(&device_id, config, callback);
            assert!(stream.is_ok(), "Failed for {} channels", channels);
            assert_eq!(stream.unwrap().config().channels, channels);
        }
    }

    #[test]
    fn test_alsa_exclusive_mode_config() {
        let backend = AlsaBackend::new();
        let device_id = DeviceId::new("hw:0,0");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        // With exclusive mode
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(true);
        let stream = backend.open_output(&device_id, config, callback).unwrap();
        assert!(stream.config().exclusive);

        // Without exclusive mode
        let config = StreamConfig::new(48000, 512, 2).with_exclusive(false);
        let stream = backend.open_output(&device_id, config, callback).unwrap();
        assert!(!stream.config().exclusive);
    }

    // -------------------------------------------------------------------------
    // Latency calculation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_alsa_low_latency_config() {
        let backend = AlsaBackend::new();
        let device_id = DeviceId::new("hw:0,0");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        // Ultra-low latency: 64 samples @ 96kHz
        let config = StreamConfig::new(96000, 64, 2);
        let stream = backend.open_output(&device_id, config, callback).unwrap();

        let latency_ms = stream.latency_secs() * 1000.0;
        // 128 samples @ 96kHz = ~1.33ms
        assert!(
            latency_ms < 2.0,
            "Low latency should be <2ms, got {}ms",
            latency_ms
        );
    }

    #[test]
    fn test_alsa_latency_calculation() {
        let backend = AlsaBackend::new();
        let device_id = DeviceId::new("hw:0,0");
        let callback = |_: &mut [f32], _: &CallbackInfo| {};

        let test_cases = [
            (48000, 256, 512),   // 256 * 2 = 512 samples
            (48000, 512, 1024),  // 512 * 2 = 1024 samples
            (96000, 128, 256),   // 128 * 2 = 256 samples
        ];

        for (rate, buffer, expected_latency) in test_cases {
            let config = StreamConfig::new(rate, buffer, 2);
            let stream = backend.open_output(&device_id, config, callback).unwrap();
            assert_eq!(
                stream.latency_samples(),
                expected_latency,
                "Failed for rate={}, buffer={}",
                rate,
                buffer
            );
        }
    }
}
