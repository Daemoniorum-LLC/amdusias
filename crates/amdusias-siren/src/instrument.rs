//! Instrument definitions.

use crate::{articulation::Articulation, sample::SampleZone};
use serde::{Deserialize, Serialize};

/// Instrument category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstrumentCategory {
    /// Acoustic/electric guitar.
    Guitar,
    /// Bass guitar.
    Bass,
    /// Piano/keyboard.
    Piano,
    /// Organ.
    Organ,
    /// Strings (violin, cello, etc.).
    Strings,
    /// Brass (trumpet, trombone, etc.).
    Brass,
    /// Woodwinds (flute, clarinet, etc.).
    Woodwinds,
    /// Percussion/drums.
    Percussion,
    /// Synthesizer.
    Synth,
    /// Choir/vocals.
    Choir,
    /// Sound effects.
    SoundFx,
    /// Other/custom.
    Other,
}

/// A multi-sample instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    /// Instrument ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Category.
    pub category: InstrumentCategory,
    /// Sample zones.
    pub zones: Vec<SampleZone>,
    /// Supported articulations.
    pub articulations: Vec<ArticulationMapping>,
    /// Default ADSR envelope.
    pub envelope: EnvelopeSettings,
    /// Maximum polyphony.
    pub max_voices: usize,
    /// Round-robin group count (for alternating samples).
    pub round_robin_groups: usize,
}

impl Instrument {
    /// Creates a new instrument.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, category: InstrumentCategory) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category,
            zones: Vec::new(),
            articulations: Vec::new(),
            envelope: EnvelopeSettings::default(),
            max_voices: 32,
            round_robin_groups: 1,
        }
    }

    /// Adds a sample zone.
    pub fn add_zone(&mut self, zone: SampleZone) {
        self.zones.push(zone);
    }

    /// Finds zones matching the given note, velocity, and articulation.
    pub fn find_zones(
        &self,
        note: u8,
        velocity: u8,
        articulation: Articulation,
    ) -> impl Iterator<Item = &SampleZone> {
        // First check if there's an articulation-specific zone
        let art_zones: Vec<_> = self
            .articulations
            .iter()
            .filter(|m| m.articulation == articulation)
            .flat_map(|m| m.zone_indices.iter())
            .filter_map(|&idx| self.zones.get(idx))
            .filter(|z| z.matches(note, velocity))
            .collect();

        if !art_zones.is_empty() {
            return art_zones.into_iter();
        }

        // Fall back to default zones
        self.zones
            .iter()
            .filter(move |z| z.matches(note, velocity))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/// Maps an articulation to specific sample zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticulationMapping {
    /// The articulation.
    pub articulation: Articulation,
    /// Indices into the instrument's zone array.
    pub zone_indices: Vec<usize>,
}

/// ADSR envelope settings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EnvelopeSettings {
    /// Attack time in seconds.
    pub attack: f32,
    /// Decay time in seconds.
    pub decay: f32,
    /// Sustain level (0.0 to 1.0).
    pub sustain: f32,
    /// Release time in seconds.
    pub release: f32,
}

impl Default for EnvelopeSettings {
    fn default() -> Self {
        Self {
            attack: 0.005,
            decay: 0.1,
            sustain: 0.8,
            release: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::SampleId;

    // =========================================================================
    // Phase 5 TDD: Instrument Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // InstrumentCategory tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_instrument_category_equality() {
        assert_eq!(InstrumentCategory::Guitar, InstrumentCategory::Guitar);
        assert_ne!(InstrumentCategory::Guitar, InstrumentCategory::Piano);
    }

    #[test]
    fn test_instrument_category_all_variants() {
        let categories = [
            InstrumentCategory::Guitar,
            InstrumentCategory::Bass,
            InstrumentCategory::Piano,
            InstrumentCategory::Organ,
            InstrumentCategory::Strings,
            InstrumentCategory::Brass,
            InstrumentCategory::Woodwinds,
            InstrumentCategory::Percussion,
            InstrumentCategory::Synth,
            InstrumentCategory::Choir,
            InstrumentCategory::SoundFx,
            InstrumentCategory::Other,
        ];

        assert_eq!(categories.len(), 12);
    }

    // -------------------------------------------------------------------------
    // Instrument creation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_instrument_creation() {
        let mut inst = Instrument::new("guitar-1", "Electric Guitar", InstrumentCategory::Guitar);
        inst.add_zone(SampleZone::new(SampleId(1), 60));

        assert_eq!(inst.zones.len(), 1);
        assert_eq!(inst.category, InstrumentCategory::Guitar);
    }

    #[test]
    fn test_instrument_new() {
        let inst = Instrument::new("piano-1", "Grand Piano", InstrumentCategory::Piano);

        assert_eq!(inst.id, "piano-1");
        assert_eq!(inst.name, "Grand Piano");
        assert_eq!(inst.category, InstrumentCategory::Piano);
        assert!(inst.zones.is_empty());
        assert!(inst.articulations.is_empty());
        assert_eq!(inst.max_voices, 32);
        assert_eq!(inst.round_robin_groups, 1);
    }

    #[test]
    fn test_instrument_add_zone() {
        let mut inst = Instrument::new("test", "Test", InstrumentCategory::Other);

        inst.add_zone(SampleZone::new(SampleId(1), 48));
        inst.add_zone(SampleZone::new(SampleId(2), 60));
        inst.add_zone(SampleZone::new(SampleId(3), 72));

        assert_eq!(inst.zones.len(), 3);
    }

    #[test]
    fn test_instrument_string_ownership() {
        let id = String::from("synth-1");
        let name = String::from("Lead Synth");
        let inst = Instrument::new(id, name, InstrumentCategory::Synth);

        assert_eq!(inst.id, "synth-1");
        assert_eq!(inst.name, "Lead Synth");
    }

    // -------------------------------------------------------------------------
    // Instrument find_zones tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_find_zones_basic() {
        let mut inst = Instrument::new("test", "Test", InstrumentCategory::Other);
        inst.add_zone(SampleZone::new(SampleId(1), 60));

        let zones: Vec<_> = inst.find_zones(60, 100, Articulation::Sustain).collect();
        assert_eq!(zones.len(), 1);
    }

    #[test]
    fn test_find_zones_velocity_layers() {
        let mut inst = Instrument::new("test", "Test", InstrumentCategory::Piano);

        // Add velocity layers
        inst.add_zone(SampleZone::new(SampleId(1), 60).with_velocity_range(0, 63));
        inst.add_zone(SampleZone::new(SampleId(2), 60).with_velocity_range(64, 127));

        // Soft velocity should find first zone
        let soft_zones: Vec<_> = inst.find_zones(60, 50, Articulation::Sustain).collect();
        assert_eq!(soft_zones.len(), 1);
        assert_eq!(soft_zones[0].sample_id, SampleId(1));

        // Loud velocity should find second zone
        let loud_zones: Vec<_> = inst.find_zones(60, 100, Articulation::Sustain).collect();
        assert_eq!(loud_zones.len(), 1);
        assert_eq!(loud_zones[0].sample_id, SampleId(2));
    }

    #[test]
    fn test_find_zones_key_ranges() {
        let mut inst = Instrument::new("test", "Test", InstrumentCategory::Piano);

        inst.add_zone(SampleZone::new(SampleId(1), 36).with_key_range(21, 47));
        inst.add_zone(SampleZone::new(SampleId(2), 60).with_key_range(48, 71));
        inst.add_zone(SampleZone::new(SampleId(3), 84).with_key_range(72, 108));

        // Low note
        let bass: Vec<_> = inst.find_zones(36, 100, Articulation::Sustain).collect();
        assert_eq!(bass.len(), 1);
        assert_eq!(bass[0].sample_id, SampleId(1));

        // Mid note
        let mid: Vec<_> = inst.find_zones(60, 100, Articulation::Sustain).collect();
        assert_eq!(mid.len(), 1);
        assert_eq!(mid[0].sample_id, SampleId(2));

        // High note
        let treble: Vec<_> = inst.find_zones(84, 100, Articulation::Sustain).collect();
        assert_eq!(treble.len(), 1);
        assert_eq!(treble[0].sample_id, SampleId(3));
    }

    #[test]
    fn test_find_zones_no_match() {
        let mut inst = Instrument::new("test", "Test", InstrumentCategory::Other);
        inst.add_zone(SampleZone::new(SampleId(1), 60).with_key_range(48, 72));

        // Note outside range
        let zones: Vec<_> = inst.find_zones(36, 100, Articulation::Sustain).collect();
        assert!(zones.is_empty());
    }

    // -------------------------------------------------------------------------
    // EnvelopeSettings tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_envelope_settings_default() {
        let env = EnvelopeSettings::default();

        assert_eq!(env.attack, 0.005);
        assert_eq!(env.decay, 0.1);
        assert_eq!(env.sustain, 0.8);
        assert_eq!(env.release, 0.2);
    }

    #[test]
    fn test_envelope_settings_custom() {
        let env = EnvelopeSettings {
            attack: 0.01,
            decay: 0.2,
            sustain: 0.7,
            release: 0.5,
        };

        assert_eq!(env.attack, 0.01);
        assert_eq!(env.decay, 0.2);
        assert_eq!(env.sustain, 0.7);
        assert_eq!(env.release, 0.5);
    }

    #[test]
    fn test_envelope_settings_piano() {
        // Piano has fast attack, no decay to sustain, long release
        let env = EnvelopeSettings {
            attack: 0.001,
            decay: 0.0,
            sustain: 1.0,
            release: 1.0,
        };

        assert!(env.attack < 0.01);
        assert_eq!(env.sustain, 1.0);
    }

    #[test]
    fn test_envelope_settings_pad() {
        // Pad has slow attack and release
        let env = EnvelopeSettings {
            attack: 0.5,
            decay: 0.2,
            sustain: 0.8,
            release: 1.0,
        };

        assert!(env.attack > 0.1);
        assert!(env.release > 0.5);
    }

    // -------------------------------------------------------------------------
    // ArticulationMapping tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_articulation_mapping() {
        let mapping = ArticulationMapping {
            articulation: Articulation::PalmMute,
            zone_indices: vec![0, 1, 2],
        };

        assert_eq!(mapping.articulation, Articulation::PalmMute);
        assert_eq!(mapping.zone_indices.len(), 3);
    }

    #[test]
    fn test_find_zones_with_articulation() {
        let mut inst = Instrument::new("guitar", "Electric Guitar", InstrumentCategory::Guitar);

        // Sustain zones
        inst.add_zone(SampleZone::new(SampleId(1), 60)); // index 0
        // Palm mute zones
        inst.add_zone(SampleZone::new(SampleId(2), 60)); // index 1

        // Map palm mute articulation to zone index 1
        inst.articulations.push(ArticulationMapping {
            articulation: Articulation::PalmMute,
            zone_indices: vec![1],
        });

        // Palm mute should find the mapped zone
        let mute_zones: Vec<_> = inst.find_zones(60, 100, Articulation::PalmMute).collect();
        assert_eq!(mute_zones.len(), 1);
        assert_eq!(mute_zones[0].sample_id, SampleId(2));

        // Sustain should fall back to default zones
        let sustain_zones: Vec<_> = inst.find_zones(60, 100, Articulation::Sustain).collect();
        assert_eq!(sustain_zones.len(), 2); // Both zones match the key/velocity
    }

    // -------------------------------------------------------------------------
    // Real-world instrument configuration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_multi_sample_piano() {
        let mut piano = Instrument::new("piano", "Grand Piano", InstrumentCategory::Piano);

        // 88 keys, 4 velocity layers = 352 zones
        for octave in 0..8 {
            for note in 0..12 {
                let midi_note = 21 + octave * 12 + note;
                if midi_note > 108 {
                    break;
                }

                // 4 velocity layers per note
                piano.add_zone(
                    SampleZone::new(SampleId(midi_note as u32 * 4), midi_note)
                        .with_key_range(midi_note, midi_note)
                        .with_velocity_range(0, 31),
                );
                piano.add_zone(
                    SampleZone::new(SampleId(midi_note as u32 * 4 + 1), midi_note)
                        .with_key_range(midi_note, midi_note)
                        .with_velocity_range(32, 63),
                );
                piano.add_zone(
                    SampleZone::new(SampleId(midi_note as u32 * 4 + 2), midi_note)
                        .with_key_range(midi_note, midi_note)
                        .with_velocity_range(64, 95),
                );
                piano.add_zone(
                    SampleZone::new(SampleId(midi_note as u32 * 4 + 3), midi_note)
                        .with_key_range(midi_note, midi_note)
                        .with_velocity_range(96, 127),
                );
            }
        }

        // Should have 88 * 4 = 352 zones
        assert_eq!(piano.zones.len(), 352);

        // Test middle C at different velocities
        let soft: Vec<_> = piano.find_zones(60, 20, Articulation::Sustain).collect();
        let loud: Vec<_> = piano.find_zones(60, 120, Articulation::Sustain).collect();

        assert_eq!(soft.len(), 1);
        assert_eq!(loud.len(), 1);
        assert_ne!(soft[0].sample_id, loud[0].sample_id);
    }

    #[test]
    fn test_guitar_instrument_config() {
        let mut guitar = Instrument::new("guitar", "Electric Guitar", InstrumentCategory::Guitar);
        guitar.max_voices = 12; // 6 strings, 2 voices per string
        guitar.round_robin_groups = 3; // 3 alternating samples

        assert_eq!(guitar.max_voices, 12);
        assert_eq!(guitar.round_robin_groups, 3);
    }
}
