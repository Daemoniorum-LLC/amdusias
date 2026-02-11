//! Sample types and zone definitions.

use serde::{Deserialize, Serialize};

/// A loaded audio sample.
#[derive(Debug, Clone)]
pub struct Sample {
    /// Sample ID.
    pub id: SampleId,
    /// Sample name.
    pub name: String,
    /// Sample data (mono or stereo interleaved).
    pub data: Vec<f32>,
    /// Number of channels.
    pub channels: u8,
    /// Original sample rate.
    pub sample_rate: u32,
    /// Loop mode.
    pub loop_mode: LoopMode,
    /// Loop start point (in samples).
    pub loop_start: u32,
    /// Loop end point (in samples).
    pub loop_end: u32,
}

/// Unique sample identifier.
///
/// This is a simple wrapper around a u32 for type safety.
/// The `#[repr(C)]` attribute ensures a stable ABI for FFI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)]
pub struct SampleId(pub u32);

/// Loop mode for samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LoopMode {
    /// No looping (play once).
    #[default]
    None,
    /// Loop forward.
    Forward,
    /// Loop ping-pong (forward then backward).
    PingPong,
    /// Loop backward.
    Backward,
}

/// A sample zone defines when a sample should play.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleZone {
    /// Reference to the sample.
    pub sample_id: SampleId,
    /// MIDI note range (inclusive).
    pub key_range: (u8, u8),
    /// Velocity range (inclusive).
    pub velocity_range: (u8, u8),
    /// Root key (the note at which the sample plays at original pitch).
    pub root_key: u8,
    /// Fine tuning in cents.
    pub tune_cents: i16,
    /// Gain adjustment in dB.
    pub gain_db: f32,
    /// Pan position (-1.0 to 1.0).
    pub pan: f32,
}

impl SampleZone {
    /// Creates a new sample zone.
    #[must_use]
    pub fn new(sample_id: SampleId, root_key: u8) -> Self {
        Self {
            sample_id,
            key_range: (0, 127),
            velocity_range: (0, 127),
            root_key,
            tune_cents: 0,
            gain_db: 0.0,
            pan: 0.0,
        }
    }

    /// Sets the key range.
    #[must_use]
    pub fn with_key_range(mut self, low: u8, high: u8) -> Self {
        self.key_range = (low, high);
        self
    }

    /// Sets the velocity range.
    #[must_use]
    pub fn with_velocity_range(mut self, low: u8, high: u8) -> Self {
        self.velocity_range = (low, high);
        self
    }

    /// Returns true if this zone matches the given note and velocity.
    #[must_use]
    pub fn matches(&self, note: u8, velocity: u8) -> bool {
        note >= self.key_range.0
            && note <= self.key_range.1
            && velocity >= self.velocity_range.0
            && velocity <= self.velocity_range.1
    }

    /// Calculates the pitch ratio for a given note.
    #[must_use]
    pub fn pitch_ratio(&self, note: u8) -> f64 {
        let semitone_diff = note as f64 - self.root_key as f64;
        let cent_diff = semitone_diff * 100.0 + self.tune_cents as f64;
        2.0_f64.powf(cent_diff / 1200.0)
    }
}

/// Sample reference for lazy loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRef {
    /// Sample ID.
    pub id: SampleId,
    /// Path to sample file.
    pub path: String,
    /// Whether the sample is loaded.
    #[serde(skip)]
    pub loaded: bool,
}

impl SampleRef {
    /// Creates a new sample reference.
    #[must_use]
    pub fn new(id: SampleId, path: impl Into<String>) -> Self {
        Self {
            id,
            path: path.into(),
            loaded: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Phase 5 TDD: Sample and Zone Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // SampleId tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sample_id_creation() {
        let id = SampleId(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_sample_id_equality() {
        let id1 = SampleId(1);
        let id2 = SampleId(1);
        let id3 = SampleId(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_sample_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SampleId(1));
        set.insert(SampleId(2));
        set.insert(SampleId(1)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_sample_id_clone() {
        let id = SampleId(100);
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    // -------------------------------------------------------------------------
    // LoopMode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_loop_mode_default() {
        let mode = LoopMode::default();
        assert_eq!(mode, LoopMode::None);
    }

    #[test]
    fn test_loop_mode_variants() {
        assert_eq!(LoopMode::None, LoopMode::None);
        assert_eq!(LoopMode::Forward, LoopMode::Forward);
        assert_eq!(LoopMode::PingPong, LoopMode::PingPong);
        assert_eq!(LoopMode::Backward, LoopMode::Backward);
    }

    #[test]
    fn test_loop_mode_clone() {
        let mode = LoopMode::PingPong;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    // -------------------------------------------------------------------------
    // Sample tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sample_creation() {
        let sample = Sample {
            id: SampleId(1),
            name: "Test Sample".to_string(),
            data: vec![0.0, 0.5, -0.5, 1.0],
            channels: 1,
            sample_rate: 44100,
            loop_mode: LoopMode::None,
            loop_start: 0,
            loop_end: 0,
        };

        assert_eq!(sample.id, SampleId(1));
        assert_eq!(sample.name, "Test Sample");
        assert_eq!(sample.data.len(), 4);
        assert_eq!(sample.channels, 1);
        assert_eq!(sample.sample_rate, 44100);
    }

    #[test]
    fn test_sample_stereo() {
        let sample = Sample {
            id: SampleId(2),
            name: "Stereo Sample".to_string(),
            data: vec![0.0, 0.0, 0.5, 0.5, 1.0, 1.0], // L, R, L, R, L, R
            channels: 2,
            sample_rate: 48000,
            loop_mode: LoopMode::Forward,
            loop_start: 0,
            loop_end: 2, // 3 frames
        };

        assert_eq!(sample.channels, 2);
        assert_eq!(sample.data.len() / sample.channels as usize, 3); // 3 frames
    }

    #[test]
    fn test_sample_with_loop() {
        let sample = Sample {
            id: SampleId(3),
            name: "Looped Sample".to_string(),
            data: vec![0.0; 1000],
            channels: 1,
            sample_rate: 44100,
            loop_mode: LoopMode::Forward,
            loop_start: 100,
            loop_end: 900,
        };

        assert_eq!(sample.loop_mode, LoopMode::Forward);
        assert_eq!(sample.loop_start, 100);
        assert_eq!(sample.loop_end, 900);
    }

    // -------------------------------------------------------------------------
    // SampleZone tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zone_new() {
        let zone = SampleZone::new(SampleId(1), 60);

        assert_eq!(zone.sample_id, SampleId(1));
        assert_eq!(zone.root_key, 60);
        assert_eq!(zone.key_range, (0, 127));
        assert_eq!(zone.velocity_range, (0, 127));
        assert_eq!(zone.tune_cents, 0);
        assert_eq!(zone.gain_db, 0.0);
        assert_eq!(zone.pan, 0.0);
    }

    #[test]
    fn test_zone_with_key_range() {
        let zone = SampleZone::new(SampleId(1), 60)
            .with_key_range(48, 72);

        assert_eq!(zone.key_range, (48, 72));
    }

    #[test]
    fn test_zone_with_velocity_range() {
        let zone = SampleZone::new(SampleId(1), 60)
            .with_velocity_range(64, 127);

        assert_eq!(zone.velocity_range, (64, 127));
    }

    #[test]
    fn test_zone_matching() {
        let zone = SampleZone::new(SampleId(1), 60)
            .with_key_range(48, 72)
            .with_velocity_range(64, 127);

        assert!(zone.matches(60, 100));
        assert!(!zone.matches(40, 100)); // Out of key range
        assert!(!zone.matches(60, 32)); // Out of velocity range
    }

    #[test]
    fn test_zone_matches_boundary() {
        let zone = SampleZone::new(SampleId(1), 60)
            .with_key_range(48, 72)
            .with_velocity_range(64, 127);

        // Test boundary values
        assert!(zone.matches(48, 64));   // Both at minimum
        assert!(zone.matches(72, 127));  // Both at maximum
        assert!(!zone.matches(47, 64));  // Key just below
        assert!(!zone.matches(73, 64));  // Key just above
        assert!(!zone.matches(60, 63));  // Velocity just below
    }

    #[test]
    fn test_pitch_ratio() {
        let zone = SampleZone::new(SampleId(1), 60);

        // Same note = ratio 1.0
        assert!((zone.pitch_ratio(60) - 1.0).abs() < 1e-10);

        // Octave up = ratio 2.0
        assert!((zone.pitch_ratio(72) - 2.0).abs() < 1e-10);

        // Octave down = ratio 0.5
        assert!((zone.pitch_ratio(48) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_pitch_ratio_semitone() {
        let zone = SampleZone::new(SampleId(1), 60);

        // One semitone up
        let expected = 2.0_f64.powf(1.0 / 12.0);
        assert!((zone.pitch_ratio(61) - expected).abs() < 1e-10);

        // One semitone down
        let expected = 2.0_f64.powf(-1.0 / 12.0);
        assert!((zone.pitch_ratio(59) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_pitch_ratio_with_tune_cents() {
        let mut zone = SampleZone::new(SampleId(1), 60);
        zone.tune_cents = 50; // 50 cents sharp

        // At root key, pitch ratio should be slightly higher
        let ratio = zone.pitch_ratio(60);
        assert!(ratio > 1.0);
        assert!(ratio < 2.0_f64.powf(1.0 / 12.0)); // Less than one semitone
    }

    #[test]
    fn test_pitch_ratio_fifth() {
        let zone = SampleZone::new(SampleId(1), 60);

        // Perfect fifth (7 semitones)
        let ratio = zone.pitch_ratio(67);
        let expected = 2.0_f64.powf(7.0 / 12.0);
        assert!((ratio - expected).abs() < 1e-10);
    }

    // -------------------------------------------------------------------------
    // SampleRef tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sample_ref_new() {
        let sample_ref = SampleRef::new(SampleId(1), "samples/piano_c4.wav");

        assert_eq!(sample_ref.id, SampleId(1));
        assert_eq!(sample_ref.path, "samples/piano_c4.wav");
        assert!(!sample_ref.loaded);
    }

    #[test]
    fn test_sample_ref_string_ownership() {
        let path = String::from("samples/guitar_e2.wav");
        let sample_ref = SampleRef::new(SampleId(2), path);

        assert_eq!(sample_ref.path, "samples/guitar_e2.wav");
    }

    // -------------------------------------------------------------------------
    // Velocity layer tests (simulating real instrument behavior)
    // -------------------------------------------------------------------------

    #[test]
    fn test_velocity_layers() {
        // Simulate a piano with 4 velocity layers
        let soft = SampleZone::new(SampleId(1), 60).with_velocity_range(0, 31);
        let medium_soft = SampleZone::new(SampleId(2), 60).with_velocity_range(32, 63);
        let medium_loud = SampleZone::new(SampleId(3), 60).with_velocity_range(64, 95);
        let loud = SampleZone::new(SampleId(4), 60).with_velocity_range(96, 127);

        // Check layer selection
        assert!(soft.matches(60, 20));
        assert!(medium_soft.matches(60, 50));
        assert!(medium_loud.matches(60, 80));
        assert!(loud.matches(60, 120));

        // Check non-overlapping
        assert!(!soft.matches(60, 50));
        assert!(!loud.matches(60, 50));
    }

    #[test]
    fn test_multizone_keyboard() {
        // Simulate zones across a keyboard
        let bass_zone = SampleZone::new(SampleId(1), 36).with_key_range(21, 47);
        let mid_zone = SampleZone::new(SampleId(2), 60).with_key_range(48, 71);
        let treble_zone = SampleZone::new(SampleId(3), 84).with_key_range(72, 108);

        assert!(bass_zone.matches(40, 100));
        assert!(mid_zone.matches(60, 100));
        assert!(treble_zone.matches(80, 100));

        // Verify zones don't overlap
        assert!(!bass_zone.matches(60, 100));
        assert!(!mid_zone.matches(40, 100));
    }
}

/// Property-based tests for pitch ratio calculations.
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Pitch ratio at root key should always be 1.0 (within floating point tolerance).
        #[test]
        fn prop_pitch_ratio_at_root_is_one(root_key in 0u8..=127) {
            let zone = SampleZone::new(SampleId(1), root_key);
            let ratio = zone.pitch_ratio(root_key);
            prop_assert!((ratio - 1.0).abs() < 1e-10, "Expected ratio 1.0, got {}", ratio);
        }

        /// Pitch ratio should double for every octave up.
        #[test]
        fn prop_pitch_ratio_octave_up_doubles(root_key in 0u8..=115) {
            let zone = SampleZone::new(SampleId(1), root_key);
            let octave_up = root_key + 12;
            let ratio = zone.pitch_ratio(octave_up);
            prop_assert!((ratio - 2.0).abs() < 1e-10, "Expected ratio 2.0, got {}", ratio);
        }

        /// Pitch ratio should halve for every octave down.
        #[test]
        fn prop_pitch_ratio_octave_down_halves(root_key in 12u8..=127) {
            let zone = SampleZone::new(SampleId(1), root_key);
            let octave_down = root_key - 12;
            let ratio = zone.pitch_ratio(octave_down);
            prop_assert!((ratio - 0.5).abs() < 1e-10, "Expected ratio 0.5, got {}", ratio);
        }

        /// Pitch ratio should always be positive.
        #[test]
        fn prop_pitch_ratio_always_positive(
            root_key in 0u8..=127,
            note in 0u8..=127,
            tune_cents in -1200i16..=1200
        ) {
            let mut zone = SampleZone::new(SampleId(1), root_key);
            zone.tune_cents = tune_cents;
            let ratio = zone.pitch_ratio(note);
            prop_assert!(ratio > 0.0, "Ratio should be positive, got {}", ratio);
        }

        /// Pitch ratio should increase monotonically with note number.
        #[test]
        fn prop_pitch_ratio_monotonic(
            root_key in 0u8..=127,
            note1 in 0u8..=126
        ) {
            let zone = SampleZone::new(SampleId(1), root_key);
            let note2 = note1 + 1;
            let ratio1 = zone.pitch_ratio(note1);
            let ratio2 = zone.pitch_ratio(note2);
            prop_assert!(ratio2 > ratio1, "Higher note should have higher ratio");
        }

        /// Tune cents should affect pitch ratio correctly.
        #[test]
        fn prop_tune_cents_direction(root_key in 0u8..=127, cents in 1i16..=100) {
            let mut zone_sharp = SampleZone::new(SampleId(1), root_key);
            zone_sharp.tune_cents = cents;

            let mut zone_flat = SampleZone::new(SampleId(2), root_key);
            zone_flat.tune_cents = -cents;

            let ratio_sharp = zone_sharp.pitch_ratio(root_key);
            let ratio_flat = zone_flat.pitch_ratio(root_key);

            prop_assert!(ratio_sharp > 1.0, "Sharp tuning should raise pitch");
            prop_assert!(ratio_flat < 1.0, "Flat tuning should lower pitch");
        }

        /// Zone matching should respect key range boundaries.
        #[test]
        fn prop_zone_key_range_boundary(
            low in 0u8..=120,
            high in 0u8..=127
        ) {
            prop_assume!(low <= high);

            let zone = SampleZone::new(SampleId(1), 60)
                .with_key_range(low, high);

            // Inside range should match
            if low <= 60 && 60 <= high {
                prop_assert!(zone.matches(60, 100));
            }

            // Below range should not match
            if low > 0 {
                prop_assert!(!zone.matches(low - 1, 100));
            }

            // Above range should not match
            if high < 127 {
                prop_assert!(!zone.matches(high + 1, 100));
            }
        }

        /// Zone matching should respect velocity range boundaries.
        #[test]
        fn prop_zone_velocity_range_boundary(
            low in 0u8..=120,
            high in 0u8..=127
        ) {
            prop_assume!(low <= high);

            let zone = SampleZone::new(SampleId(1), 60)
                .with_velocity_range(low, high);

            // Below range should not match
            if low > 0 {
                prop_assert!(!zone.matches(60, low - 1));
            }

            // Above range should not match
            if high < 127 {
                prop_assert!(!zone.matches(60, high + 1));
            }
        }
    }
}
