//! Audio device enumeration and information.

use crate::config::{BufferSizeRange, SampleRateRange};

/// Unique identifier for an audio device.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);

impl DeviceId {
    /// Creates a new device ID.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the device ID as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of audio device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Output device (speakers, headphones, audio interface output).
    Output,
    /// Input device (microphone, audio interface input).
    Input,
    /// Duplex device (supports both input and output).
    Duplex,
}

/// Information about an audio device.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Unique device identifier.
    pub id: DeviceId,
    /// Human-readable device name.
    pub name: String,
    /// Device type (input, output, duplex).
    pub device_type: DeviceType,
    /// Whether this is the system default device.
    pub is_default: bool,
    /// Supported sample rates.
    pub sample_rates: SampleRateRange,
    /// Supported buffer sizes.
    pub buffer_sizes: BufferSizeRange,
    /// Maximum number of input channels.
    pub max_input_channels: usize,
    /// Maximum number of output channels.
    pub max_output_channels: usize,
}

impl DeviceInfo {
    /// Returns true if this device supports input.
    #[must_use]
    pub const fn supports_input(&self) -> bool {
        matches!(self.device_type, DeviceType::Input | DeviceType::Duplex)
    }

    /// Returns true if this device supports output.
    #[must_use]
    pub const fn supports_output(&self) -> bool {
        matches!(self.device_type, DeviceType::Output | DeviceType::Duplex)
    }

    /// Returns true if this device supports the given sample rate.
    #[must_use]
    pub fn supports_sample_rate(&self, rate: u32) -> bool {
        self.sample_rates.contains(rate)
    }

    /// Returns true if this device supports the given buffer size.
    #[must_use]
    pub fn supports_buffer_size(&self, size: usize) -> bool {
        self.buffer_sizes.contains(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type() {
        let info = DeviceInfo {
            id: DeviceId::new("test"),
            name: "Test Device".to_string(),
            device_type: DeviceType::Duplex,
            is_default: true,
            sample_rates: SampleRateRange::Discrete(vec![44100, 48000, 96000]),
            buffer_sizes: BufferSizeRange {
                min: 64,
                max: 4096,
                preferred: 512,
            },
            max_input_channels: 2,
            max_output_channels: 2,
        };

        assert!(info.supports_input());
        assert!(info.supports_output());
        assert!(info.supports_sample_rate(48000));
        assert!(!info.supports_sample_rate(22050));
    }

    // =========================================================================
    // Phase 3 TDD: Comprehensive device tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // DeviceId tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_device_id_new() {
        let id = DeviceId::new("hw:0,0");
        assert_eq!(id.as_str(), "hw:0,0");
    }

    #[test]
    fn test_device_id_from_string() {
        let id = DeviceId::new(String::from("default"));
        assert_eq!(id.as_str(), "default");
    }

    #[test]
    fn test_device_id_display() {
        let id = DeviceId::new("my-audio-device");
        let display = format!("{}", id);
        assert_eq!(display, "my-audio-device");
    }

    #[test]
    fn test_device_id_eq() {
        let id1 = DeviceId::new("device");
        let id2 = DeviceId::new("device");
        let id3 = DeviceId::new("other");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_device_id_clone() {
        let id = DeviceId::new("cloneable");
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_device_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(DeviceId::new("device1"));
        set.insert(DeviceId::new("device2"));
        set.insert(DeviceId::new("device1")); // Duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&DeviceId::new("device1")));
    }

    #[test]
    fn test_device_id_debug() {
        let id = DeviceId::new("debug-test");
        let debug = format!("{:?}", id);
        assert!(debug.contains("DeviceId"));
        assert!(debug.contains("debug-test"));
    }

    // -------------------------------------------------------------------------
    // DeviceType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_device_type_output() {
        assert_eq!(DeviceType::Output, DeviceType::Output);
        assert_ne!(DeviceType::Output, DeviceType::Input);
    }

    #[test]
    fn test_device_type_clone() {
        let dt = DeviceType::Duplex;
        let cloned = dt.clone();
        assert_eq!(dt, cloned);
    }

    #[test]
    fn test_device_type_copy() {
        let dt = DeviceType::Input;
        let copied: DeviceType = dt; // Copy, not move
        assert_eq!(dt, copied);
    }

    #[test]
    fn test_device_type_debug() {
        assert!(format!("{:?}", DeviceType::Output).contains("Output"));
        assert!(format!("{:?}", DeviceType::Input).contains("Input"));
        assert!(format!("{:?}", DeviceType::Duplex).contains("Duplex"));
    }

    // -------------------------------------------------------------------------
    // DeviceInfo tests
    // -------------------------------------------------------------------------

    fn create_test_device(device_type: DeviceType) -> DeviceInfo {
        DeviceInfo {
            id: DeviceId::new("test-device"),
            name: "Test Audio Device".to_string(),
            device_type,
            is_default: false,
            sample_rates: SampleRateRange::Discrete(vec![44100, 48000, 96000]),
            buffer_sizes: BufferSizeRange {
                min: 64,
                max: 4096,
                preferred: 512,
            },
            max_input_channels: 2,
            max_output_channels: 2,
        }
    }

    #[test]
    fn test_device_info_supports_input_output_only() {
        let device = create_test_device(DeviceType::Output);
        assert!(!device.supports_input());
        assert!(device.supports_output());
    }

    #[test]
    fn test_device_info_supports_input_input_only() {
        let device = create_test_device(DeviceType::Input);
        assert!(device.supports_input());
        assert!(!device.supports_output());
    }

    #[test]
    fn test_device_info_supports_input_duplex() {
        let device = create_test_device(DeviceType::Duplex);
        assert!(device.supports_input());
        assert!(device.supports_output());
    }

    #[test]
    fn test_device_info_supports_sample_rate() {
        let device = create_test_device(DeviceType::Output);

        assert!(device.supports_sample_rate(44100));
        assert!(device.supports_sample_rate(48000));
        assert!(device.supports_sample_rate(96000));
        assert!(!device.supports_sample_rate(22050));
        assert!(!device.supports_sample_rate(192000));
    }

    #[test]
    fn test_device_info_supports_buffer_size() {
        let device = create_test_device(DeviceType::Output);

        assert!(device.supports_buffer_size(64));
        assert!(device.supports_buffer_size(256));
        assert!(device.supports_buffer_size(512));
        assert!(device.supports_buffer_size(4096));
        assert!(!device.supports_buffer_size(32));
        assert!(!device.supports_buffer_size(8192));
    }

    #[test]
    fn test_device_info_is_default() {
        let mut device = create_test_device(DeviceType::Output);
        assert!(!device.is_default);

        device.is_default = true;
        assert!(device.is_default);
    }

    #[test]
    fn test_device_info_channel_counts() {
        let device = create_test_device(DeviceType::Duplex);
        assert_eq!(device.max_input_channels, 2);
        assert_eq!(device.max_output_channels, 2);
    }

    #[test]
    fn test_device_info_multichannel() {
        let device = DeviceInfo {
            id: DeviceId::new("pro-interface"),
            name: "Pro Audio Interface".to_string(),
            device_type: DeviceType::Duplex,
            is_default: false,
            sample_rates: SampleRateRange::Range {
                min: 44100,
                max: 192000,
            },
            buffer_sizes: BufferSizeRange {
                min: 32,
                max: 8192,
                preferred: 128,
            },
            max_input_channels: 18,
            max_output_channels: 20,
        };

        assert_eq!(device.max_input_channels, 18);
        assert_eq!(device.max_output_channels, 20);
        assert!(device.supports_sample_rate(192000));
        assert!(device.supports_buffer_size(32));
    }

    #[test]
    fn test_device_info_clone() {
        let device = create_test_device(DeviceType::Duplex);
        let cloned = device.clone();

        assert_eq!(cloned.id, device.id);
        assert_eq!(cloned.name, device.name);
        assert_eq!(cloned.device_type, device.device_type);
        assert_eq!(cloned.is_default, device.is_default);
        assert_eq!(cloned.max_input_channels, device.max_input_channels);
        assert_eq!(cloned.max_output_channels, device.max_output_channels);
    }

    #[test]
    fn test_device_info_debug() {
        let device = create_test_device(DeviceType::Output);
        let debug = format!("{:?}", device);

        assert!(debug.contains("DeviceInfo"));
        assert!(debug.contains("test-device"));
        assert!(debug.contains("Test Audio Device"));
    }

    // -------------------------------------------------------------------------
    // Real-world device scenario tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_usb_audio_interface() {
        let device = DeviceInfo {
            id: DeviceId::new("usb-focusrite-2i2"),
            name: "Focusrite Scarlett 2i2".to_string(),
            device_type: DeviceType::Duplex,
            is_default: false,
            sample_rates: SampleRateRange::Discrete(vec![44100, 48000, 88200, 96000]),
            buffer_sizes: BufferSizeRange {
                min: 64,
                max: 2048,
                preferred: 256,
            },
            max_input_channels: 2,
            max_output_channels: 2,
        };

        assert!(device.supports_input());
        assert!(device.supports_output());
        assert!(device.supports_sample_rate(96000));
        assert!(device.supports_buffer_size(256));
    }

    #[test]
    fn test_builtin_speakers() {
        let device = DeviceInfo {
            id: DeviceId::new("builtin-speakers"),
            name: "Built-in Output".to_string(),
            device_type: DeviceType::Output,
            is_default: true,
            sample_rates: SampleRateRange::Discrete(vec![44100, 48000]),
            buffer_sizes: BufferSizeRange {
                min: 256,
                max: 4096,
                preferred: 512,
            },
            max_input_channels: 0,
            max_output_channels: 2,
        };

        assert!(!device.supports_input());
        assert!(device.supports_output());
        assert!(device.is_default);
        assert!(!device.supports_sample_rate(96000));
    }

    #[test]
    fn test_builtin_microphone() {
        let device = DeviceInfo {
            id: DeviceId::new("builtin-mic"),
            name: "Built-in Microphone".to_string(),
            device_type: DeviceType::Input,
            is_default: true,
            sample_rates: SampleRateRange::Discrete(vec![44100, 48000]),
            buffer_sizes: BufferSizeRange {
                min: 256,
                max: 4096,
                preferred: 512,
            },
            max_input_channels: 2,
            max_output_channels: 0,
        };

        assert!(device.supports_input());
        assert!(!device.supports_output());
        assert!(device.is_default);
    }
}
