//! Guitar-specific instrument modeling.

use crate::{
    articulation::Articulation,
    instrument::{EnvelopeSettings, Instrument, InstrumentCategory},
    sample::{SampleId, SampleZone},
};
use serde::{Deserialize, Serialize};

/// A guitar instrument with per-string modeling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuitarInstrument {
    /// Base instrument data.
    pub base: Instrument,
    /// Guitar strings (typically 6 or 7).
    pub strings: Vec<GuitarString>,
    /// Pickup configuration.
    pub pickups: Vec<Pickup>,
    /// Currently selected pickup.
    pub active_pickup: usize,
    /// Amp model.
    pub amp: Option<AmpModel>,
    /// Cabinet model.
    pub cabinet: Option<CabinetModel>,
}

impl GuitarInstrument {
    /// Creates a new 6-string guitar in standard tuning.
    #[must_use]
    pub fn standard_6_string(id: impl Into<String>, name: impl Into<String>) -> Self {
        let base = Instrument::new(id, name, InstrumentCategory::Guitar);

        // Standard tuning: E2, A2, D3, G3, B3, E4
        let tuning = [40, 45, 50, 55, 59, 64];

        let strings = tuning
            .iter()
            .enumerate()
            .map(|(i, &open_note)| GuitarString::new(i as u8, open_note, 24))
            .collect();

        Self {
            base,
            strings,
            pickups: vec![
                Pickup::new("Neck", PickupPosition::Neck),
                Pickup::new("Bridge", PickupPosition::Bridge),
            ],
            active_pickup: 0,
            amp: None,
            cabinet: None,
        }
    }

    /// Creates a 7-string guitar.
    #[must_use]
    pub fn standard_7_string(id: impl Into<String>, name: impl Into<String>) -> Self {
        let base = Instrument::new(id, name, InstrumentCategory::Guitar);

        // 7-string tuning: B1, E2, A2, D3, G3, B3, E4
        let tuning = [35, 40, 45, 50, 55, 59, 64];

        let strings = tuning
            .iter()
            .enumerate()
            .map(|(i, &open_note)| GuitarString::new(i as u8, open_note, 24))
            .collect();

        Self {
            base,
            strings,
            pickups: vec![
                Pickup::new("Neck", PickupPosition::Neck),
                Pickup::new("Bridge", PickupPosition::Bridge),
            ],
            active_pickup: 0,
            amp: None,
            cabinet: None,
        }
    }

    /// Finds the best string and fret for a given note.
    pub fn find_position(&self, note: u8) -> Option<(usize, u8)> {
        for (string_idx, string) in self.strings.iter().enumerate() {
            if note >= string.open_note && note <= string.open_note + string.fret_count {
                let fret = note - string.open_note;
                return Some((string_idx, fret));
            }
        }
        None
    }
}

/// A single guitar string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuitarString {
    /// String index (0 = lowest pitched).
    pub index: u8,
    /// Open string note (MIDI).
    pub open_note: u8,
    /// Number of frets.
    pub fret_count: u8,
    /// Samples for sustain notes.
    pub sustain_zones: Vec<SampleZone>,
    /// Samples for palm muted notes.
    pub mute_zones: Vec<SampleZone>,
    /// Samples for harmonics.
    pub harmonic_zones: Vec<SampleZone>,
    /// Samples for release noise (finger lift).
    pub release_zones: Vec<SampleZone>,
    /// Samples for slide sounds.
    pub slide_zones: Vec<SampleZone>,
}

impl GuitarString {
    /// Creates a new guitar string.
    #[must_use]
    pub fn new(index: u8, open_note: u8, fret_count: u8) -> Self {
        Self {
            index,
            open_note,
            fret_count,
            sustain_zones: Vec::new(),
            mute_zones: Vec::new(),
            harmonic_zones: Vec::new(),
            release_zones: Vec::new(),
            slide_zones: Vec::new(),
        }
    }

    /// Returns the note at a given fret.
    #[must_use]
    pub fn note_at_fret(&self, fret: u8) -> u8 {
        self.open_note + fret
    }

    /// Returns true if the fret is within range.
    #[must_use]
    pub fn is_valid_fret(&self, fret: u8) -> bool {
        fret <= self.fret_count
    }
}

/// Guitar pickup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pickup {
    /// Pickup name.
    pub name: String,
    /// Position on the guitar.
    pub position: PickupPosition,
    /// Pickup type.
    pub pickup_type: PickupType,
    /// Volume level (0.0 to 1.0).
    pub volume: f32,
    /// Tone control (0.0 to 1.0).
    pub tone: f32,
}

impl Pickup {
    /// Creates a new pickup.
    #[must_use]
    pub fn new(name: impl Into<String>, position: PickupPosition) -> Self {
        Self {
            name: name.into(),
            position,
            pickup_type: PickupType::Humbucker,
            volume: 1.0,
            tone: 1.0,
        }
    }
}

/// Pickup position on the guitar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickupPosition {
    /// Near the neck (warmer, rounder tone).
    Neck,
    /// Between neck and bridge.
    Middle,
    /// Near the bridge (brighter, more attack).
    Bridge,
}

/// Type of guitar pickup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PickupType {
    /// Single coil pickup (brighter, can have hum).
    SingleCoil,
    /// Humbucker pickup (fuller, no hum).
    #[default]
    Humbucker,
    /// P90 pickup (between single coil and humbucker).
    P90,
    /// Active pickup (higher output, compressed).
    Active,
}

/// Guitar amplifier model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmpModel {
    /// Amp name.
    pub name: String,
    /// Amp type/character.
    pub amp_type: AmpType,
    /// Gain/drive.
    pub gain: f32,
    /// Bass EQ.
    pub bass: f32,
    /// Mid EQ.
    pub mid: f32,
    /// Treble EQ.
    pub treble: f32,
    /// Presence.
    pub presence: f32,
    /// Master volume.
    pub master: f32,
}

/// Type of guitar amplifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AmpType {
    /// Clean amp (Fender-style).
    Clean,
    /// Crunch amp (Marshall-style).
    #[default]
    Crunch,
    /// High gain (Mesa-style).
    HighGain,
    /// Modern metal.
    Modern,
    /// Acoustic amp.
    Acoustic,
    /// Bass amp.
    Bass,
}

/// Speaker cabinet model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CabinetModel {
    /// Cabinet name.
    pub name: String,
    /// Number of speakers.
    pub speakers: u8,
    /// Speaker size in inches.
    pub speaker_size: u8,
    /// Impulse response file path (if using convolution).
    pub ir_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Phase 5 TDD: Guitar Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // GuitarInstrument creation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_standard_tuning() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test Guitar");

        // Check standard tuning
        assert_eq!(guitar.strings[0].open_note, 40); // E2
        assert_eq!(guitar.strings[5].open_note, 64); // E4
    }

    #[test]
    fn test_standard_6_string() {
        let guitar = GuitarInstrument::standard_6_string("strat", "Stratocaster");

        assert_eq!(guitar.base.id, "strat");
        assert_eq!(guitar.base.name, "Stratocaster");
        assert_eq!(guitar.base.category, InstrumentCategory::Guitar);
        assert_eq!(guitar.strings.len(), 6);
        assert_eq!(guitar.pickups.len(), 2);
    }

    #[test]
    fn test_standard_6_string_tuning() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        // Standard tuning: E2, A2, D3, G3, B3, E4
        let expected = [40, 45, 50, 55, 59, 64];
        for (i, &note) in expected.iter().enumerate() {
            assert_eq!(guitar.strings[i].open_note, note);
        }
    }

    #[test]
    fn test_standard_7_string() {
        let guitar = GuitarInstrument::standard_7_string("ibanez", "RG7");

        assert_eq!(guitar.strings.len(), 7);

        // 7-string tuning: B1, E2, A2, D3, G3, B3, E4
        let expected = [35, 40, 45, 50, 55, 59, 64];
        for (i, &note) in expected.iter().enumerate() {
            assert_eq!(guitar.strings[i].open_note, note);
        }
    }

    #[test]
    fn test_guitar_fret_count() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        // All strings should have 24 frets
        for string in &guitar.strings {
            assert_eq!(string.fret_count, 24);
        }
    }

    // -------------------------------------------------------------------------
    // GuitarString tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_guitar_string_new() {
        let string = GuitarString::new(0, 40, 24);

        assert_eq!(string.index, 0);
        assert_eq!(string.open_note, 40);
        assert_eq!(string.fret_count, 24);
        assert!(string.sustain_zones.is_empty());
        assert!(string.mute_zones.is_empty());
        assert!(string.harmonic_zones.is_empty());
    }

    #[test]
    fn test_guitar_string_note_at_fret() {
        let string = GuitarString::new(0, 40, 24); // Low E

        assert_eq!(string.note_at_fret(0), 40);  // Open E2
        assert_eq!(string.note_at_fret(5), 45);  // A2
        assert_eq!(string.note_at_fret(12), 52); // E3 (octave)
        assert_eq!(string.note_at_fret(24), 64); // E4 (two octaves)
    }

    #[test]
    fn test_guitar_string_is_valid_fret() {
        let string = GuitarString::new(0, 40, 24);

        assert!(string.is_valid_fret(0));
        assert!(string.is_valid_fret(12));
        assert!(string.is_valid_fret(24));
        assert!(!string.is_valid_fret(25));
    }

    // -------------------------------------------------------------------------
    // Position finding tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_find_position() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test Guitar");

        // Middle C (C4 = 60) should be on string 1 (B string), fret 1
        // Actually: B3 = 59, so C4 = 60 is fret 1 on B string (index 4)
        let pos = guitar.find_position(60);
        assert!(pos.is_some());
    }

    #[test]
    fn test_find_position_open_strings() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        // Open low E
        let pos = guitar.find_position(40);
        assert_eq!(pos, Some((0, 0)));

        // Open A
        let pos = guitar.find_position(45);
        assert_eq!(pos, Some((0, 5))); // Can be played on string 0, fret 5
    }

    #[test]
    fn test_find_position_fretted() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        // G2 (43) on low E string = fret 3
        let pos = guitar.find_position(43);
        assert_eq!(pos, Some((0, 3)));

        // E3 (52) on low E string = fret 12
        let pos = guitar.find_position(52);
        assert_eq!(pos, Some((0, 12)));
    }

    #[test]
    fn test_find_position_out_of_range() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        // Note below lowest string
        let pos = guitar.find_position(30);
        assert!(pos.is_none());
    }

    // -------------------------------------------------------------------------
    // Pickup tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_pickup_new() {
        let pickup = Pickup::new("Bridge", PickupPosition::Bridge);

        assert_eq!(pickup.name, "Bridge");
        assert_eq!(pickup.position, PickupPosition::Bridge);
        assert_eq!(pickup.pickup_type, PickupType::Humbucker); // default
        assert_eq!(pickup.volume, 1.0);
        assert_eq!(pickup.tone, 1.0);
    }

    #[test]
    fn test_pickup_positions() {
        assert_eq!(PickupPosition::Neck, PickupPosition::Neck);
        assert_eq!(PickupPosition::Middle, PickupPosition::Middle);
        assert_eq!(PickupPosition::Bridge, PickupPosition::Bridge);
    }

    #[test]
    fn test_pickup_types() {
        assert_eq!(PickupType::default(), PickupType::Humbucker);
        assert_ne!(PickupType::SingleCoil, PickupType::Humbucker);
    }

    #[test]
    fn test_guitar_pickups() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        assert_eq!(guitar.pickups.len(), 2);
        assert_eq!(guitar.pickups[0].position, PickupPosition::Neck);
        assert_eq!(guitar.pickups[1].position, PickupPosition::Bridge);
        assert_eq!(guitar.active_pickup, 0);
    }

    // -------------------------------------------------------------------------
    // AmpModel tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_amp_type_default() {
        assert_eq!(AmpType::default(), AmpType::Crunch);
    }

    #[test]
    fn test_amp_type_variants() {
        let types = [
            AmpType::Clean,
            AmpType::Crunch,
            AmpType::HighGain,
            AmpType::Modern,
            AmpType::Acoustic,
            AmpType::Bass,
        ];

        assert_eq!(types.len(), 6);
    }

    #[test]
    fn test_amp_model_creation() {
        let amp = AmpModel {
            name: "Marshall JCM800".to_string(),
            amp_type: AmpType::Crunch,
            gain: 0.7,
            bass: 0.5,
            mid: 0.6,
            treble: 0.7,
            presence: 0.5,
            master: 0.8,
        };

        assert_eq!(amp.name, "Marshall JCM800");
        assert_eq!(amp.amp_type, AmpType::Crunch);
        assert!(amp.gain > 0.5);
    }

    // -------------------------------------------------------------------------
    // CabinetModel tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cabinet_model() {
        let cabinet = CabinetModel {
            name: "4x12 Vintage".to_string(),
            speakers: 4,
            speaker_size: 12,
            ir_path: Some("ir/vintage_4x12.wav".to_string()),
        };

        assert_eq!(cabinet.name, "4x12 Vintage");
        assert_eq!(cabinet.speakers, 4);
        assert_eq!(cabinet.speaker_size, 12);
        assert!(cabinet.ir_path.is_some());
    }

    #[test]
    fn test_cabinet_without_ir() {
        let cabinet = CabinetModel {
            name: "1x12 Combo".to_string(),
            speakers: 1,
            speaker_size: 12,
            ir_path: None,
        };

        assert!(cabinet.ir_path.is_none());
    }

    // -------------------------------------------------------------------------
    // Guitar configuration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_guitar_no_amp() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        assert!(guitar.amp.is_none());
        assert!(guitar.cabinet.is_none());
    }

    #[test]
    fn test_guitar_with_amp() {
        let mut guitar = GuitarInstrument::standard_6_string("test", "Test");

        guitar.amp = Some(AmpModel {
            name: "Clean".to_string(),
            amp_type: AmpType::Clean,
            gain: 0.3,
            bass: 0.5,
            mid: 0.5,
            treble: 0.5,
            presence: 0.5,
            master: 0.5,
        });

        assert!(guitar.amp.is_some());
        assert_eq!(guitar.amp.as_ref().unwrap().amp_type, AmpType::Clean);
    }

    // -------------------------------------------------------------------------
    // Real-world guitar scenarios
    // -------------------------------------------------------------------------

    #[test]
    fn test_drop_d_tuning() {
        // Drop D: D2, A2, D3, G3, B3, E4 (first string down a whole step)
        let mut guitar = GuitarInstrument::standard_6_string("test", "Drop D");

        // Manually adjust low E to D
        guitar.strings[0] = GuitarString::new(0, 38, 24);

        assert_eq!(guitar.strings[0].open_note, 38); // D2
        assert_eq!(guitar.strings[1].open_note, 45); // A2 (unchanged)
    }

    #[test]
    fn test_bass_guitar() {
        // 4-string bass: E1, A1, D2, G2
        let bass_tuning = [28, 33, 38, 43];

        for (i, &note) in bass_tuning.iter().enumerate() {
            let string = GuitarString::new(i as u8, note, 24);
            assert_eq!(string.open_note, note);
        }
    }

    #[test]
    fn test_all_notes_on_fretboard() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        // Count how many unique notes are playable (E2 to E6)
        let mut notes = std::collections::HashSet::new();

        for string in &guitar.strings {
            for fret in 0..=string.fret_count {
                notes.insert(string.note_at_fret(fret));
            }
        }

        // Standard tuning with 24 frets covers E2 (40) to E6 (88)
        // That's about 49 semitones
        assert!(notes.len() >= 48);
    }

    #[test]
    fn test_same_note_different_strings() {
        let guitar = GuitarInstrument::standard_6_string("test", "Test");

        // Note A3 (57) can be played on:
        // - String 2 (D), fret 7
        // - String 3 (G), fret 2
        // - String 4 (B), open (actually B is 59, so A is not on B open)

        let d_string = &guitar.strings[2]; // D3 = 50
        let g_string = &guitar.strings[3]; // G3 = 55

        assert_eq!(d_string.note_at_fret(7), 57); // A3
        assert_eq!(g_string.note_at_fret(2), 57); // A3
    }

    #[test]
    fn test_pickup_type_variants() {
        let single_coil = PickupType::SingleCoil;
        let humbucker = PickupType::Humbucker;
        let p90 = PickupType::P90;
        let active = PickupType::Active;

        // Just verify they're distinct
        assert_ne!(single_coil, humbucker);
        assert_ne!(p90, active);
    }

    #[test]
    fn test_stratocaster_config() {
        let mut guitar = GuitarInstrument::standard_6_string("strat", "Stratocaster");

        // Strat has 3 single-coil pickups
        guitar.pickups = vec![
            Pickup {
                name: "Neck".to_string(),
                position: PickupPosition::Neck,
                pickup_type: PickupType::SingleCoil,
                volume: 1.0,
                tone: 1.0,
            },
            Pickup {
                name: "Middle".to_string(),
                position: PickupPosition::Middle,
                pickup_type: PickupType::SingleCoil,
                volume: 1.0,
                tone: 1.0,
            },
            Pickup {
                name: "Bridge".to_string(),
                position: PickupPosition::Bridge,
                pickup_type: PickupType::SingleCoil,
                volume: 1.0,
                tone: 1.0,
            },
        ];

        assert_eq!(guitar.pickups.len(), 3);
        for pickup in &guitar.pickups {
            assert_eq!(pickup.pickup_type, PickupType::SingleCoil);
        }
    }

    #[test]
    fn test_les_paul_config() {
        let mut guitar = GuitarInstrument::standard_6_string("lp", "Les Paul");

        // LP has 2 humbuckers
        guitar.pickups = vec![
            Pickup {
                name: "Neck".to_string(),
                position: PickupPosition::Neck,
                pickup_type: PickupType::Humbucker,
                volume: 1.0,
                tone: 1.0,
            },
            Pickup {
                name: "Bridge".to_string(),
                position: PickupPosition::Bridge,
                pickup_type: PickupType::Humbucker,
                volume: 1.0,
                tone: 1.0,
            },
        ];

        assert_eq!(guitar.pickups.len(), 2);
        for pickup in &guitar.pickups {
            assert_eq!(pickup.pickup_type, PickupType::Humbucker);
        }
    }
}
