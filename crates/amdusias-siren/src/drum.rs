//! Drum and percussion instrument definitions.
//!
//! This module provides comprehensive drum kit modeling including:
//! - Individual drum pieces with multiple articulations
//! - Multi-microphone position support (close, overhead, room)
//! - GM (General MIDI) drum mapping
//! - Velocity-sensitive sample selection
//! - Round-robin for natural variation
//! - Choke groups for mutually exclusive sounds
//!
//! ## Choke Groups
//!
//! Choke groups define pieces that should stop each other when triggered.
//! The classic example is the hi-hat: when you close the hi-hat (pedal),
//! any ringing open hi-hat sound should be choked (stopped).
//!
//! ```rust,ignore
//! use amdusias_siren::drum::{DrumPiece, DrumPieceType};
//!
//! // Hi-hat pieces share choke group 1
//! let closed_hihat = DrumPiece::new("hh-closed", "Closed Hi-Hat", DrumPieceType::HiHat)
//!     .with_choke_group(1)
//!     .with_midi_note(42);
//!
//! let open_hihat = DrumPiece::new("hh-open", "Open Hi-Hat", DrumPieceType::HiHat)
//!     .with_choke_group(1)
//!     .with_midi_note(46);
//!
//! let pedal_hihat = DrumPiece::new("hh-pedal", "Pedal Hi-Hat", DrumPieceType::HiHat)
//!     .with_choke_group(1)
//!     .with_midi_note(44);
//! ```
//!
//! When any piece in a choke group is triggered, all other voices playing
//! pieces in the same group should be released. Use [`DrumKit::find_choke_group`]
//! to query pieces in a group.

use crate::sample::{SampleId, SampleZone};
use serde::{Deserialize, Serialize};

/// Types of drum pieces in a kit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrumPieceType {
    /// Bass drum / kick.
    Kick,
    /// Snare drum.
    Snare,
    /// Hi-hat (open/closed/pedal).
    HiHat,
    /// Rack tom.
    Tom,
    /// Floor tom.
    FloorTom,
    /// Crash cymbal.
    Crash,
    /// Ride cymbal.
    Ride,
    /// China cymbal.
    China,
    /// Splash cymbal.
    Splash,
    /// Cowbell.
    Cowbell,
    /// Tambourine.
    Tambourine,
    /// Clap/handclap.
    Clap,
    /// Rim click (side stick).
    RimClick,
    /// Cross stick.
    CrossStick,
    /// Other percussion.
    Other,
}

/// Drum-specific articulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrumArticulation {
    /// Normal hit (center of head).
    Center,
    /// Edge hit (near rim).
    Edge,
    /// Rim shot (stick hits rim and head simultaneously).
    RimShot,
    /// Cross stick / side stick.
    CrossStick,
    /// Rim only (no head contact).
    RimOnly,
    /// Flam (grace note before main hit).
    Flam,
    /// Drag (two grace notes before main hit).
    Drag,
    /// Buzz roll.
    BuzzRoll,
    /// Ghost note (very soft).
    Ghost,
    /// Dead stroke (muted immediately).
    DeadStroke,

    // Hi-hat specific
    /// Hi-hat fully closed.
    Closed,
    /// Hi-hat slightly open (tight).
    HalfOpen,
    /// Hi-hat fully open.
    Open,
    /// Hi-hat pedal close sound.
    PedalClose,
    /// Hi-hat foot splash.
    FootSplash,

    // Cymbal specific
    /// Bell of cymbal.
    Bell,
    /// Bow of cymbal (main surface).
    Bow,
    /// Crash hit.
    CrashHit,
    /// Choke (grab after hit).
    Choke,
    /// Muted hit.
    Muted,
    /// Scrape/sizzle.
    Scrape,
}

impl Default for DrumArticulation {
    fn default() -> Self {
        Self::Center
    }
}

impl DrumArticulation {
    /// Returns true if this is a hi-hat specific articulation.
    #[must_use]
    pub const fn is_hihat_specific(&self) -> bool {
        matches!(
            self,
            Self::Closed | Self::HalfOpen | Self::Open | Self::PedalClose | Self::FootSplash
        )
    }

    /// Returns true if this is a cymbal specific articulation.
    #[must_use]
    pub const fn is_cymbal_specific(&self) -> bool {
        matches!(
            self,
            Self::Bell | Self::Bow | Self::CrashHit | Self::Choke | Self::Scrape
        )
    }

    /// Returns true if this articulation requires a grace note.
    #[must_use]
    pub const fn has_grace_note(&self) -> bool {
        matches!(self, Self::Flam | Self::Drag)
    }

    /// Returns the number of grace notes for this articulation.
    #[must_use]
    pub const fn grace_note_count(&self) -> u8 {
        match self {
            Self::Flam => 1,
            Self::Drag => 2,
            _ => 0,
        }
    }

    /// Returns the velocity modifier for this articulation (relative to input velocity).
    #[must_use]
    pub const fn velocity_modifier(&self) -> f32 {
        match self {
            Self::Ghost => 0.3,
            Self::Edge => 0.9,
            Self::RimOnly => 0.7,
            Self::Muted => 0.6,
            _ => 1.0,
        }
    }
}

/// Microphone position for drum recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MicPosition {
    /// Close microphone (directly on the drum).
    Close,
    /// Overhead microphone (above kit).
    Overhead,
    /// Room microphone (ambient).
    Room,
    /// Bottom microphone (under snare, inside kick).
    Bottom,
    /// Top microphone (above snare/tom).
    Top,
}

impl MicPosition {
    /// Returns the typical distance in meters for this mic position.
    #[must_use]
    pub const fn typical_distance_meters(&self) -> f32 {
        match self {
            Self::Close | Self::Top | Self::Bottom => 0.1,
            Self::Overhead => 1.5,
            Self::Room => 3.0,
        }
    }

    /// Returns the typical pan position (-1.0 to 1.0) for this mic.
    /// 0.0 is center.
    #[must_use]
    pub const fn default_pan(&self) -> f32 {
        match self {
            Self::Close | Self::Top | Self::Bottom => 0.0, // Mono, panned per piece
            Self::Overhead => 0.0,                          // Usually stereo pair
            Self::Room => 0.0,                              // Usually stereo pair
        }
    }
}

/// A sample layer for a specific mic position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicLayer {
    /// Microphone position.
    pub position: MicPosition,
    /// Sample zones for this mic (velocity layers).
    pub zones: Vec<SampleZone>,
    /// Volume level (0.0 to 1.0).
    pub level: f32,
    /// Pan position (-1.0 to 1.0).
    pub pan: f32,
    /// Whether this mic is currently enabled.
    pub enabled: bool,
}

impl MicLayer {
    /// Creates a new mic layer.
    #[must_use]
    pub fn new(position: MicPosition) -> Self {
        Self {
            position,
            zones: Vec::new(),
            level: 1.0,
            pan: position.default_pan(),
            enabled: true,
        }
    }

    /// Adds a sample zone to this mic layer.
    pub fn add_zone(&mut self, zone: SampleZone) {
        self.zones.push(zone);
    }

    /// Sets the level.
    #[must_use]
    pub fn with_level(mut self, level: f32) -> Self {
        self.level = level.clamp(0.0, 1.0);
        self
    }

    /// Sets the pan.
    #[must_use]
    pub fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan.clamp(-1.0, 1.0);
        self
    }
}

/// A single drum piece with multiple articulations and mic positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumPiece {
    /// Piece identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Type of drum piece.
    pub piece_type: DrumPieceType,
    /// MIDI note for this piece (GM mapping).
    pub midi_note: u8,
    /// Articulation layers (each articulation can have multiple mic positions).
    pub articulations: Vec<ArticulationLayer>,
    /// Round-robin group count.
    pub round_robin_groups: usize,
    /// Current round-robin index.
    #[serde(skip)]
    pub current_rr_index: usize,
    /// Whether this piece chokes other pieces (e.g., hi-hat).
    pub choke_group: Option<u8>,
}

/// A layer of samples for a specific articulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticulationLayer {
    /// The articulation this layer handles.
    pub articulation: DrumArticulation,
    /// Mic layers for this articulation.
    pub mic_layers: Vec<MicLayer>,
}

impl ArticulationLayer {
    /// Creates a new articulation layer.
    #[must_use]
    pub fn new(articulation: DrumArticulation) -> Self {
        Self {
            articulation,
            mic_layers: Vec::new(),
        }
    }

    /// Adds a mic layer.
    pub fn add_mic_layer(&mut self, layer: MicLayer) {
        self.mic_layers.push(layer);
    }
}

impl DrumPiece {
    /// Creates a new drum piece.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, piece_type: DrumPieceType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            piece_type,
            midi_note: piece_type.gm_default_note(),
            articulations: Vec::new(),
            round_robin_groups: 1,
            current_rr_index: 0,
            choke_group: None,
        }
    }

    /// Sets the MIDI note.
    #[must_use]
    pub fn with_midi_note(mut self, note: u8) -> Self {
        self.midi_note = note;
        self
    }

    /// Sets the choke group.
    #[must_use]
    pub fn with_choke_group(mut self, group: u8) -> Self {
        self.choke_group = Some(group);
        self
    }

    /// Adds an articulation layer.
    pub fn add_articulation(&mut self, layer: ArticulationLayer) {
        self.articulations.push(layer);
    }

    /// Gets the next round-robin index and advances.
    pub fn advance_round_robin(&mut self) -> usize {
        let idx = self.current_rr_index;
        self.current_rr_index = (self.current_rr_index + 1) % self.round_robin_groups;
        idx
    }

    /// Finds the articulation layer for the given articulation.
    #[must_use]
    pub fn find_articulation(&self, articulation: DrumArticulation) -> Option<&ArticulationLayer> {
        self.articulations
            .iter()
            .find(|a| a.articulation == articulation)
    }
}

impl DrumPieceType {
    /// Returns the default GM MIDI note for this piece type.
    #[must_use]
    pub const fn gm_default_note(&self) -> u8 {
        match self {
            Self::Kick => 36,       // C1 (Bass Drum 1)
            Self::Snare => 38,      // D1 (Acoustic Snare)
            Self::HiHat => 42,      // F#1 (Closed Hi-Hat)
            Self::Tom => 48,        // C2 (Hi Mid Tom)
            Self::FloorTom => 43,   // G1 (High Floor Tom)
            Self::Crash => 49,      // C#2 (Crash Cymbal 1)
            Self::Ride => 51,       // D#2 (Ride Cymbal 1)
            Self::China => 52,      // E2 (Chinese Cymbal)
            Self::Splash => 55,     // G2 (Splash Cymbal)
            Self::Cowbell => 56,    // G#2 (Cowbell)
            Self::Tambourine => 54, // F#2 (Tambourine)
            Self::Clap => 39,       // D#1 (Hand Clap)
            Self::RimClick => 37,   // C#1 (Side Stick)
            Self::CrossStick => 37, // Same as rim click
            Self::Other => 60,      // C3
        }
    }

    /// Returns true if this piece type is a cymbal.
    #[must_use]
    pub const fn is_cymbal(&self) -> bool {
        matches!(
            self,
            Self::HiHat | Self::Crash | Self::Ride | Self::China | Self::Splash
        )
    }

    /// Returns true if this piece type is a drum (has a head).
    #[must_use]
    pub const fn is_drum(&self) -> bool {
        matches!(
            self,
            Self::Kick | Self::Snare | Self::Tom | Self::FloorTom
        )
    }
}

/// A complete drum kit with multiple pieces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrumKit {
    /// Kit identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Drum pieces in the kit.
    pub pieces: Vec<DrumPiece>,
    /// Global overhead mic settings.
    pub overhead_level: f32,
    /// Global room mic settings.
    pub room_level: f32,
    /// Kit tuning offset in semitones.
    pub tuning: f32,
}

impl DrumKit {
    /// Creates a new drum kit.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pieces: Vec::new(),
            overhead_level: 0.7,
            room_level: 0.3,
            tuning: 0.0,
        }
    }

    /// Adds a piece to the kit.
    pub fn add_piece(&mut self, piece: DrumPiece) {
        self.pieces.push(piece);
    }

    /// Finds a piece by MIDI note.
    #[must_use]
    pub fn find_by_note(&self, note: u8) -> Option<&DrumPiece> {
        self.pieces.iter().find(|p| p.midi_note == note)
    }

    /// Finds a mutable piece by MIDI note.
    pub fn find_by_note_mut(&mut self, note: u8) -> Option<&mut DrumPiece> {
        self.pieces.iter_mut().find(|p| p.midi_note == note)
    }

    /// Finds pieces in the same choke group.
    #[must_use]
    pub fn find_choke_group(&self, group: u8) -> Vec<&DrumPiece> {
        self.pieces
            .iter()
            .filter(|p| p.choke_group == Some(group))
            .collect()
    }

    /// Creates a standard rock kit configuration.
    #[must_use]
    pub fn standard_rock_kit() -> Self {
        let mut kit = Self::new("rock-kit", "Standard Rock Kit");

        // Kick drum
        kit.add_piece(DrumPiece::new("kick", "Kick", DrumPieceType::Kick));

        // Snare
        kit.add_piece(DrumPiece::new("snare", "Snare", DrumPieceType::Snare));

        // Hi-hat (with choke group)
        kit.add_piece(
            DrumPiece::new("hihat", "Hi-Hat", DrumPieceType::HiHat).with_choke_group(1),
        );

        // Toms
        kit.add_piece(
            DrumPiece::new("tom-high", "High Tom", DrumPieceType::Tom).with_midi_note(50),
        );
        kit.add_piece(
            DrumPiece::new("tom-mid", "Mid Tom", DrumPieceType::Tom).with_midi_note(48),
        );
        kit.add_piece(
            DrumPiece::new("tom-low", "Low Tom", DrumPieceType::Tom).with_midi_note(47),
        );
        kit.add_piece(DrumPiece::new(
            "floor-tom",
            "Floor Tom",
            DrumPieceType::FloorTom,
        ));

        // Cymbals
        kit.add_piece(DrumPiece::new("crash", "Crash", DrumPieceType::Crash));
        kit.add_piece(DrumPiece::new("ride", "Ride", DrumPieceType::Ride));

        kit
    }

    /// Creates a minimal jazz kit configuration.
    #[must_use]
    pub fn jazz_kit() -> Self {
        let mut kit = Self::new("jazz-kit", "Jazz Kit");

        kit.add_piece(DrumPiece::new("kick", "Kick", DrumPieceType::Kick));
        kit.add_piece(DrumPiece::new("snare", "Snare", DrumPieceType::Snare));
        kit.add_piece(
            DrumPiece::new("hihat", "Hi-Hat", DrumPieceType::HiHat).with_choke_group(1),
        );
        kit.add_piece(DrumPiece::new("ride", "Ride", DrumPieceType::Ride));

        // Jazz kits typically have smaller tuning (higher pitched)
        kit.tuning = 2.0;

        kit
    }
}

/// GM (General MIDI) Drum Map.
/// Maps MIDI notes to drum piece types according to the GM standard.
#[derive(Debug, Clone, Copy)]
pub struct GmDrumMap;

impl GmDrumMap {
    /// GM drum note range (35-81).
    pub const NOTE_RANGE: (u8, u8) = (35, 81);

    /// Acoustic Bass Drum.
    pub const ACOUSTIC_BASS_DRUM: u8 = 35;
    /// Bass Drum 1.
    pub const BASS_DRUM_1: u8 = 36;
    /// Side Stick.
    pub const SIDE_STICK: u8 = 37;
    /// Acoustic Snare.
    pub const ACOUSTIC_SNARE: u8 = 38;
    /// Hand Clap.
    pub const HAND_CLAP: u8 = 39;
    /// Electric Snare.
    pub const ELECTRIC_SNARE: u8 = 40;
    /// Low Floor Tom.
    pub const LOW_FLOOR_TOM: u8 = 41;
    /// Closed Hi-Hat.
    pub const CLOSED_HI_HAT: u8 = 42;
    /// High Floor Tom.
    pub const HIGH_FLOOR_TOM: u8 = 43;
    /// Pedal Hi-Hat.
    pub const PEDAL_HI_HAT: u8 = 44;
    /// Low Tom.
    pub const LOW_TOM: u8 = 45;
    /// Open Hi-Hat.
    pub const OPEN_HI_HAT: u8 = 46;
    /// Low-Mid Tom.
    pub const LOW_MID_TOM: u8 = 47;
    /// Hi-Mid Tom.
    pub const HI_MID_TOM: u8 = 48;
    /// Crash Cymbal 1.
    pub const CRASH_CYMBAL_1: u8 = 49;
    /// High Tom.
    pub const HIGH_TOM: u8 = 50;
    /// Ride Cymbal 1.
    pub const RIDE_CYMBAL_1: u8 = 51;
    /// Chinese Cymbal.
    pub const CHINESE_CYMBAL: u8 = 52;
    /// Ride Bell.
    pub const RIDE_BELL: u8 = 53;
    /// Tambourine.
    pub const TAMBOURINE: u8 = 54;
    /// Splash Cymbal.
    pub const SPLASH_CYMBAL: u8 = 55;
    /// Cowbell.
    pub const COWBELL: u8 = 56;
    /// Crash Cymbal 2.
    pub const CRASH_CYMBAL_2: u8 = 57;
    /// Vibraslap.
    pub const VIBRASLAP: u8 = 58;
    /// Ride Cymbal 2.
    pub const RIDE_CYMBAL_2: u8 = 59;

    /// Returns the drum piece type for a GM note.
    #[must_use]
    pub const fn piece_type_for_note(note: u8) -> Option<DrumPieceType> {
        match note {
            35 | 36 => Some(DrumPieceType::Kick),
            37 => Some(DrumPieceType::RimClick),
            38 | 40 => Some(DrumPieceType::Snare),
            39 => Some(DrumPieceType::Clap),
            41 | 43 => Some(DrumPieceType::FloorTom),
            42 | 44 | 46 => Some(DrumPieceType::HiHat),
            45 | 47 | 48 | 50 => Some(DrumPieceType::Tom),
            49 | 57 => Some(DrumPieceType::Crash),
            51 | 53 | 59 => Some(DrumPieceType::Ride),
            52 => Some(DrumPieceType::China),
            54 => Some(DrumPieceType::Tambourine),
            55 => Some(DrumPieceType::Splash),
            56 => Some(DrumPieceType::Cowbell),
            _ => None,
        }
    }

    /// Returns the articulation for a GM hi-hat note.
    #[must_use]
    pub const fn hihat_articulation_for_note(note: u8) -> Option<DrumArticulation> {
        match note {
            42 => Some(DrumArticulation::Closed),
            44 => Some(DrumArticulation::PedalClose),
            46 => Some(DrumArticulation::Open),
            _ => None,
        }
    }

    /// Returns true if this note is in the GM drum range.
    #[must_use]
    pub const fn is_valid_note(note: u8) -> bool {
        note >= Self::NOTE_RANGE.0 && note <= Self::NOTE_RANGE.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Phase 5 TDD: Drum Module Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // DrumPieceType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drum_piece_type_equality() {
        assert_eq!(DrumPieceType::Kick, DrumPieceType::Kick);
        assert_ne!(DrumPieceType::Kick, DrumPieceType::Snare);
    }

    #[test]
    fn test_drum_piece_type_all_variants() {
        let types = [
            DrumPieceType::Kick,
            DrumPieceType::Snare,
            DrumPieceType::HiHat,
            DrumPieceType::Tom,
            DrumPieceType::FloorTom,
            DrumPieceType::Crash,
            DrumPieceType::Ride,
            DrumPieceType::China,
            DrumPieceType::Splash,
            DrumPieceType::Cowbell,
            DrumPieceType::Tambourine,
            DrumPieceType::Clap,
            DrumPieceType::RimClick,
            DrumPieceType::CrossStick,
            DrumPieceType::Other,
        ];
        assert_eq!(types.len(), 15);
    }

    #[test]
    fn test_drum_piece_type_gm_notes() {
        assert_eq!(DrumPieceType::Kick.gm_default_note(), 36);
        assert_eq!(DrumPieceType::Snare.gm_default_note(), 38);
        assert_eq!(DrumPieceType::HiHat.gm_default_note(), 42);
        assert_eq!(DrumPieceType::Crash.gm_default_note(), 49);
        assert_eq!(DrumPieceType::Ride.gm_default_note(), 51);
    }

    #[test]
    fn test_drum_piece_type_is_cymbal() {
        assert!(DrumPieceType::HiHat.is_cymbal());
        assert!(DrumPieceType::Crash.is_cymbal());
        assert!(DrumPieceType::Ride.is_cymbal());
        assert!(DrumPieceType::China.is_cymbal());
        assert!(DrumPieceType::Splash.is_cymbal());

        assert!(!DrumPieceType::Kick.is_cymbal());
        assert!(!DrumPieceType::Snare.is_cymbal());
        assert!(!DrumPieceType::Tom.is_cymbal());
    }

    #[test]
    fn test_drum_piece_type_is_drum() {
        assert!(DrumPieceType::Kick.is_drum());
        assert!(DrumPieceType::Snare.is_drum());
        assert!(DrumPieceType::Tom.is_drum());
        assert!(DrumPieceType::FloorTom.is_drum());

        assert!(!DrumPieceType::HiHat.is_drum());
        assert!(!DrumPieceType::Crash.is_drum());
        assert!(!DrumPieceType::Cowbell.is_drum());
    }

    // -------------------------------------------------------------------------
    // DrumArticulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drum_articulation_default() {
        let art = DrumArticulation::default();
        assert_eq!(art, DrumArticulation::Center);
    }

    #[test]
    fn test_drum_articulation_equality() {
        assert_eq!(DrumArticulation::RimShot, DrumArticulation::RimShot);
        assert_ne!(DrumArticulation::RimShot, DrumArticulation::CrossStick);
    }

    #[test]
    fn test_drum_articulation_is_hihat_specific() {
        assert!(DrumArticulation::Closed.is_hihat_specific());
        assert!(DrumArticulation::HalfOpen.is_hihat_specific());
        assert!(DrumArticulation::Open.is_hihat_specific());
        assert!(DrumArticulation::PedalClose.is_hihat_specific());
        assert!(DrumArticulation::FootSplash.is_hihat_specific());

        assert!(!DrumArticulation::Center.is_hihat_specific());
        assert!(!DrumArticulation::RimShot.is_hihat_specific());
    }

    #[test]
    fn test_drum_articulation_is_cymbal_specific() {
        assert!(DrumArticulation::Bell.is_cymbal_specific());
        assert!(DrumArticulation::Bow.is_cymbal_specific());
        assert!(DrumArticulation::CrashHit.is_cymbal_specific());
        assert!(DrumArticulation::Choke.is_cymbal_specific());
        assert!(DrumArticulation::Scrape.is_cymbal_specific());

        assert!(!DrumArticulation::Center.is_cymbal_specific());
        assert!(!DrumArticulation::Closed.is_cymbal_specific());
    }

    #[test]
    fn test_drum_articulation_has_grace_note() {
        assert!(DrumArticulation::Flam.has_grace_note());
        assert!(DrumArticulation::Drag.has_grace_note());

        assert!(!DrumArticulation::Center.has_grace_note());
        assert!(!DrumArticulation::RimShot.has_grace_note());
        assert!(!DrumArticulation::Ghost.has_grace_note());
    }

    #[test]
    fn test_drum_articulation_grace_note_count() {
        assert_eq!(DrumArticulation::Flam.grace_note_count(), 1);
        assert_eq!(DrumArticulation::Drag.grace_note_count(), 2);
        assert_eq!(DrumArticulation::Center.grace_note_count(), 0);
        assert_eq!(DrumArticulation::RimShot.grace_note_count(), 0);
    }

    #[test]
    fn test_drum_articulation_velocity_modifier() {
        assert_eq!(DrumArticulation::Ghost.velocity_modifier(), 0.3);
        assert_eq!(DrumArticulation::Edge.velocity_modifier(), 0.9);
        assert_eq!(DrumArticulation::RimOnly.velocity_modifier(), 0.7);
        assert_eq!(DrumArticulation::Muted.velocity_modifier(), 0.6);
        assert_eq!(DrumArticulation::Center.velocity_modifier(), 1.0);
        assert_eq!(DrumArticulation::RimShot.velocity_modifier(), 1.0);
    }

    // -------------------------------------------------------------------------
    // MicPosition tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mic_position_equality() {
        assert_eq!(MicPosition::Close, MicPosition::Close);
        assert_ne!(MicPosition::Close, MicPosition::Room);
    }

    #[test]
    fn test_mic_position_all_variants() {
        let positions = [
            MicPosition::Close,
            MicPosition::Overhead,
            MicPosition::Room,
            MicPosition::Bottom,
            MicPosition::Top,
        ];
        assert_eq!(positions.len(), 5);
    }

    #[test]
    fn test_mic_position_typical_distance() {
        // Close mics should be nearby
        assert!(MicPosition::Close.typical_distance_meters() < 0.5);
        assert!(MicPosition::Top.typical_distance_meters() < 0.5);
        assert!(MicPosition::Bottom.typical_distance_meters() < 0.5);

        // Overhead should be medium distance
        assert!(MicPosition::Overhead.typical_distance_meters() > 1.0);
        assert!(MicPosition::Overhead.typical_distance_meters() < 2.5);

        // Room should be far
        assert!(MicPosition::Room.typical_distance_meters() > 2.0);
    }

    #[test]
    fn test_mic_position_default_pan() {
        // All mic positions should default to center
        assert_eq!(MicPosition::Close.default_pan(), 0.0);
        assert_eq!(MicPosition::Overhead.default_pan(), 0.0);
        assert_eq!(MicPosition::Room.default_pan(), 0.0);
    }

    // -------------------------------------------------------------------------
    // MicLayer tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mic_layer_new() {
        let layer = MicLayer::new(MicPosition::Close);

        assert_eq!(layer.position, MicPosition::Close);
        assert!(layer.zones.is_empty());
        assert_eq!(layer.level, 1.0);
        assert!(layer.enabled);
    }

    #[test]
    fn test_mic_layer_add_zone() {
        let mut layer = MicLayer::new(MicPosition::Overhead);
        layer.add_zone(SampleZone::new(SampleId(1), 36));
        layer.add_zone(SampleZone::new(SampleId(2), 36).with_velocity_range(64, 127));

        assert_eq!(layer.zones.len(), 2);
    }

    #[test]
    fn test_mic_layer_with_level() {
        let layer = MicLayer::new(MicPosition::Room).with_level(0.5);
        assert_eq!(layer.level, 0.5);
    }

    #[test]
    fn test_mic_layer_level_clamping() {
        let too_high = MicLayer::new(MicPosition::Room).with_level(1.5);
        let too_low = MicLayer::new(MicPosition::Room).with_level(-0.5);

        assert_eq!(too_high.level, 1.0);
        assert_eq!(too_low.level, 0.0);
    }

    #[test]
    fn test_mic_layer_with_pan() {
        let layer = MicLayer::new(MicPosition::Close).with_pan(-0.5);
        assert_eq!(layer.pan, -0.5);
    }

    #[test]
    fn test_mic_layer_pan_clamping() {
        let too_left = MicLayer::new(MicPosition::Close).with_pan(-1.5);
        let too_right = MicLayer::new(MicPosition::Close).with_pan(1.5);

        assert_eq!(too_left.pan, -1.0);
        assert_eq!(too_right.pan, 1.0);
    }

    // -------------------------------------------------------------------------
    // ArticulationLayer tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_articulation_layer_new() {
        let layer = ArticulationLayer::new(DrumArticulation::RimShot);

        assert_eq!(layer.articulation, DrumArticulation::RimShot);
        assert!(layer.mic_layers.is_empty());
    }

    #[test]
    fn test_articulation_layer_add_mic() {
        let mut layer = ArticulationLayer::new(DrumArticulation::Center);
        layer.add_mic_layer(MicLayer::new(MicPosition::Close));
        layer.add_mic_layer(MicLayer::new(MicPosition::Overhead));
        layer.add_mic_layer(MicLayer::new(MicPosition::Room));

        assert_eq!(layer.mic_layers.len(), 3);
    }

    // -------------------------------------------------------------------------
    // DrumPiece tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drum_piece_new() {
        let piece = DrumPiece::new("kick-1", "Kick Drum", DrumPieceType::Kick);

        assert_eq!(piece.id, "kick-1");
        assert_eq!(piece.name, "Kick Drum");
        assert_eq!(piece.piece_type, DrumPieceType::Kick);
        assert_eq!(piece.midi_note, 36); // GM default
        assert!(piece.articulations.is_empty());
        assert_eq!(piece.round_robin_groups, 1);
        assert_eq!(piece.current_rr_index, 0);
        assert_eq!(piece.choke_group, None);
    }

    #[test]
    fn test_drum_piece_with_midi_note() {
        let piece = DrumPiece::new("kick-2", "Kick 2", DrumPieceType::Kick).with_midi_note(35);

        assert_eq!(piece.midi_note, 35);
    }

    #[test]
    fn test_drum_piece_with_choke_group() {
        let piece = DrumPiece::new("hihat", "Hi-Hat", DrumPieceType::HiHat).with_choke_group(1);

        assert_eq!(piece.choke_group, Some(1));
    }

    #[test]
    fn test_drum_piece_add_articulation() {
        let mut piece = DrumPiece::new("snare", "Snare", DrumPieceType::Snare);
        piece.add_articulation(ArticulationLayer::new(DrumArticulation::Center));
        piece.add_articulation(ArticulationLayer::new(DrumArticulation::RimShot));
        piece.add_articulation(ArticulationLayer::new(DrumArticulation::CrossStick));

        assert_eq!(piece.articulations.len(), 3);
    }

    #[test]
    fn test_drum_piece_round_robin() {
        let mut piece = DrumPiece::new("snare", "Snare", DrumPieceType::Snare);
        piece.round_robin_groups = 4;

        assert_eq!(piece.advance_round_robin(), 0);
        assert_eq!(piece.advance_round_robin(), 1);
        assert_eq!(piece.advance_round_robin(), 2);
        assert_eq!(piece.advance_round_robin(), 3);
        assert_eq!(piece.advance_round_robin(), 0); // Wraps
    }

    #[test]
    fn test_drum_piece_find_articulation() {
        let mut piece = DrumPiece::new("snare", "Snare", DrumPieceType::Snare);
        piece.add_articulation(ArticulationLayer::new(DrumArticulation::Center));
        piece.add_articulation(ArticulationLayer::new(DrumArticulation::RimShot));

        assert!(piece.find_articulation(DrumArticulation::Center).is_some());
        assert!(piece.find_articulation(DrumArticulation::RimShot).is_some());
        assert!(piece.find_articulation(DrumArticulation::Ghost).is_none());
    }

    // -------------------------------------------------------------------------
    // DrumKit tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_drum_kit_new() {
        let kit = DrumKit::new("my-kit", "My Custom Kit");

        assert_eq!(kit.id, "my-kit");
        assert_eq!(kit.name, "My Custom Kit");
        assert!(kit.pieces.is_empty());
        assert_eq!(kit.overhead_level, 0.7);
        assert_eq!(kit.room_level, 0.3);
        assert_eq!(kit.tuning, 0.0);
    }

    #[test]
    fn test_drum_kit_add_piece() {
        let mut kit = DrumKit::new("test", "Test Kit");
        kit.add_piece(DrumPiece::new("kick", "Kick", DrumPieceType::Kick));
        kit.add_piece(DrumPiece::new("snare", "Snare", DrumPieceType::Snare));

        assert_eq!(kit.pieces.len(), 2);
    }

    #[test]
    fn test_drum_kit_find_by_note() {
        let mut kit = DrumKit::new("test", "Test Kit");
        kit.add_piece(DrumPiece::new("kick", "Kick", DrumPieceType::Kick));
        kit.add_piece(DrumPiece::new("snare", "Snare", DrumPieceType::Snare));

        let kick = kit.find_by_note(36);
        assert!(kick.is_some());
        assert_eq!(kick.unwrap().piece_type, DrumPieceType::Kick);

        let snare = kit.find_by_note(38);
        assert!(snare.is_some());
        assert_eq!(snare.unwrap().piece_type, DrumPieceType::Snare);

        let missing = kit.find_by_note(99);
        assert!(missing.is_none());
    }

    #[test]
    fn test_drum_kit_find_by_note_mut() {
        let mut kit = DrumKit::new("test", "Test Kit");
        kit.add_piece(DrumPiece::new("kick", "Kick", DrumPieceType::Kick));

        if let Some(kick) = kit.find_by_note_mut(36) {
            kick.round_robin_groups = 8;
        }

        assert_eq!(kit.pieces[0].round_robin_groups, 8);
    }

    #[test]
    fn test_drum_kit_choke_groups() {
        let mut kit = DrumKit::new("test", "Test Kit");

        // Hi-hat variants in same choke group
        kit.add_piece(
            DrumPiece::new("hh-closed", "Closed HH", DrumPieceType::HiHat)
                .with_midi_note(42)
                .with_choke_group(1),
        );
        kit.add_piece(
            DrumPiece::new("hh-open", "Open HH", DrumPieceType::HiHat)
                .with_midi_note(46)
                .with_choke_group(1),
        );
        kit.add_piece(
            DrumPiece::new("hh-pedal", "Pedal HH", DrumPieceType::HiHat)
                .with_midi_note(44)
                .with_choke_group(1),
        );

        // Snare not in choke group
        kit.add_piece(DrumPiece::new("snare", "Snare", DrumPieceType::Snare));

        let choke_group = kit.find_choke_group(1);
        assert_eq!(choke_group.len(), 3);
    }

    #[test]
    fn test_drum_kit_standard_rock() {
        let kit = DrumKit::standard_rock_kit();

        assert_eq!(kit.id, "rock-kit");
        assert!(kit.pieces.len() >= 9); // Kick, snare, hihat, 3 toms, floor tom, crash, ride

        // Should have essential pieces
        assert!(kit.find_by_note(36).is_some()); // Kick
        assert!(kit.find_by_note(38).is_some()); // Snare
        assert!(kit.find_by_note(42).is_some()); // Hi-hat
        assert!(kit.find_by_note(49).is_some()); // Crash
        assert!(kit.find_by_note(51).is_some()); // Ride
    }

    #[test]
    fn test_drum_kit_jazz() {
        let kit = DrumKit::jazz_kit();

        assert_eq!(kit.id, "jazz-kit");
        assert_eq!(kit.tuning, 2.0); // Higher pitched for jazz

        // Jazz kits are more minimal
        assert!(kit.pieces.len() >= 4); // Kick, snare, hihat, ride
    }

    // -------------------------------------------------------------------------
    // GM Drum Map tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gm_drum_map_range() {
        assert_eq!(GmDrumMap::NOTE_RANGE, (35, 81));
    }

    #[test]
    fn test_gm_drum_map_is_valid_note() {
        assert!(GmDrumMap::is_valid_note(35));
        assert!(GmDrumMap::is_valid_note(36));
        assert!(GmDrumMap::is_valid_note(81));
        assert!(!GmDrumMap::is_valid_note(34));
        assert!(!GmDrumMap::is_valid_note(82));
    }

    #[test]
    fn test_gm_drum_map_constants() {
        assert_eq!(GmDrumMap::ACOUSTIC_BASS_DRUM, 35);
        assert_eq!(GmDrumMap::BASS_DRUM_1, 36);
        assert_eq!(GmDrumMap::SIDE_STICK, 37);
        assert_eq!(GmDrumMap::ACOUSTIC_SNARE, 38);
        assert_eq!(GmDrumMap::HAND_CLAP, 39);
        assert_eq!(GmDrumMap::CLOSED_HI_HAT, 42);
        assert_eq!(GmDrumMap::OPEN_HI_HAT, 46);
        assert_eq!(GmDrumMap::CRASH_CYMBAL_1, 49);
        assert_eq!(GmDrumMap::RIDE_CYMBAL_1, 51);
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_note_kicks() {
        assert_eq!(
            GmDrumMap::piece_type_for_note(35),
            Some(DrumPieceType::Kick)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(36),
            Some(DrumPieceType::Kick)
        );
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_note_snares() {
        assert_eq!(
            GmDrumMap::piece_type_for_note(38),
            Some(DrumPieceType::Snare)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(40),
            Some(DrumPieceType::Snare)
        );
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_note_hihats() {
        assert_eq!(
            GmDrumMap::piece_type_for_note(42),
            Some(DrumPieceType::HiHat)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(44),
            Some(DrumPieceType::HiHat)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(46),
            Some(DrumPieceType::HiHat)
        );
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_note_toms() {
        assert_eq!(
            GmDrumMap::piece_type_for_note(45),
            Some(DrumPieceType::Tom)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(47),
            Some(DrumPieceType::Tom)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(48),
            Some(DrumPieceType::Tom)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(50),
            Some(DrumPieceType::Tom)
        );
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_note_floor_toms() {
        assert_eq!(
            GmDrumMap::piece_type_for_note(41),
            Some(DrumPieceType::FloorTom)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(43),
            Some(DrumPieceType::FloorTom)
        );
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_note_cymbals() {
        assert_eq!(
            GmDrumMap::piece_type_for_note(49),
            Some(DrumPieceType::Crash)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(57),
            Some(DrumPieceType::Crash)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(51),
            Some(DrumPieceType::Ride)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(52),
            Some(DrumPieceType::China)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(55),
            Some(DrumPieceType::Splash)
        );
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_note_percussion() {
        assert_eq!(
            GmDrumMap::piece_type_for_note(39),
            Some(DrumPieceType::Clap)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(54),
            Some(DrumPieceType::Tambourine)
        );
        assert_eq!(
            GmDrumMap::piece_type_for_note(56),
            Some(DrumPieceType::Cowbell)
        );
    }

    #[test]
    fn test_gm_drum_map_piece_type_for_unknown() {
        // Notes outside typical drum sounds
        assert_eq!(GmDrumMap::piece_type_for_note(60), None);
        assert_eq!(GmDrumMap::piece_type_for_note(100), None);
    }

    #[test]
    fn test_gm_drum_map_hihat_articulation() {
        assert_eq!(
            GmDrumMap::hihat_articulation_for_note(42),
            Some(DrumArticulation::Closed)
        );
        assert_eq!(
            GmDrumMap::hihat_articulation_for_note(44),
            Some(DrumArticulation::PedalClose)
        );
        assert_eq!(
            GmDrumMap::hihat_articulation_for_note(46),
            Some(DrumArticulation::Open)
        );
        assert_eq!(GmDrumMap::hihat_articulation_for_note(50), None);
    }

    // -------------------------------------------------------------------------
    // Multi-mic configuration tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_multi_mic_snare_setup() {
        let mut snare = DrumPiece::new("snare", "Snare", DrumPieceType::Snare);

        // Create articulation with multiple mics
        let mut center_art = ArticulationLayer::new(DrumArticulation::Center);

        // Top mic (main snare sound)
        let mut top_mic = MicLayer::new(MicPosition::Top);
        top_mic.add_zone(SampleZone::new(SampleId(1), 38).with_velocity_range(0, 63));
        top_mic.add_zone(SampleZone::new(SampleId(2), 38).with_velocity_range(64, 127));
        center_art.add_mic_layer(top_mic);

        // Bottom mic (snare wires)
        let mut bottom_mic = MicLayer::new(MicPosition::Bottom);
        bottom_mic.add_zone(SampleZone::new(SampleId(3), 38).with_velocity_range(0, 63));
        bottom_mic.add_zone(SampleZone::new(SampleId(4), 38).with_velocity_range(64, 127));
        center_art.add_mic_layer(bottom_mic.with_level(0.6));

        snare.add_articulation(center_art);

        let art = snare.find_articulation(DrumArticulation::Center).unwrap();
        assert_eq!(art.mic_layers.len(), 2);
        assert_eq!(art.mic_layers[0].position, MicPosition::Top);
        assert_eq!(art.mic_layers[1].position, MicPosition::Bottom);
        assert_eq!(art.mic_layers[0].zones.len(), 2);
        assert_eq!(art.mic_layers[1].level, 0.6);
    }

    #[test]
    fn test_multi_mic_kick_setup() {
        let mut kick = DrumPiece::new("kick", "Kick", DrumPieceType::Kick);

        let mut center_art = ArticulationLayer::new(DrumArticulation::Center);

        // Inside kick (beater attack)
        let inside_mic = MicLayer::new(MicPosition::Close).with_level(0.8);
        center_art.add_mic_layer(inside_mic);

        // Outside kick (resonant bass)
        let outside_mic = MicLayer::new(MicPosition::Room).with_level(0.5);
        center_art.add_mic_layer(outside_mic);

        kick.add_articulation(center_art);

        let art = kick.find_articulation(DrumArticulation::Center).unwrap();
        assert_eq!(art.mic_layers.len(), 2);
    }

    #[test]
    fn test_overhead_room_bleed() {
        // Overhead and room mics capture bleed from multiple drums
        let kit = DrumKit::standard_rock_kit();

        // Verify we have pieces that would bleed into overheads
        let cymbals: Vec<_> = kit.pieces.iter().filter(|p| p.piece_type.is_cymbal()).collect();
        let drums: Vec<_> = kit.pieces.iter().filter(|p| p.piece_type.is_drum()).collect();

        assert!(!cymbals.is_empty());
        assert!(!drums.is_empty());

        // In real implementation, overhead_level and room_level control bleed
        assert!(kit.overhead_level > 0.0);
        assert!(kit.room_level > 0.0);
    }

    // -------------------------------------------------------------------------
    // Velocity layer tests for drums
    // -------------------------------------------------------------------------

    #[test]
    fn test_drum_velocity_layers() {
        let mut snare = DrumPiece::new("snare", "Snare", DrumPieceType::Snare);

        let mut center_art = ArticulationLayer::new(DrumArticulation::Center);
        let mut close_mic = MicLayer::new(MicPosition::Close);

        // 4 velocity layers for realistic dynamics
        close_mic.add_zone(SampleZone::new(SampleId(1), 38).with_velocity_range(1, 31));   // Ghost
        close_mic.add_zone(SampleZone::new(SampleId(2), 38).with_velocity_range(32, 63));  // Soft
        close_mic.add_zone(SampleZone::new(SampleId(3), 38).with_velocity_range(64, 95));  // Medium
        close_mic.add_zone(SampleZone::new(SampleId(4), 38).with_velocity_range(96, 127)); // Hard

        center_art.add_mic_layer(close_mic);
        snare.add_articulation(center_art);

        let art = snare.find_articulation(DrumArticulation::Center).unwrap();
        assert_eq!(art.mic_layers[0].zones.len(), 4);

        // Check velocity ranges don't overlap
        assert!(art.mic_layers[0].zones[0].matches(38, 20));
        assert!(!art.mic_layers[0].zones[0].matches(38, 50));
    }

    #[test]
    fn test_round_robin_natural_variation() {
        let mut piece = DrumPiece::new("snare", "Snare", DrumPieceType::Snare);
        piece.round_robin_groups = 6; // 6 round-robin samples

        // Simulate hitting snare 12 times
        let mut rr_sequence = Vec::new();
        for _ in 0..12 {
            rr_sequence.push(piece.advance_round_robin());
        }

        // Should cycle through 0-5 twice
        assert_eq!(rr_sequence[0..6], [0, 1, 2, 3, 4, 5]);
        assert_eq!(rr_sequence[6..12], [0, 1, 2, 3, 4, 5]);
    }

    // -------------------------------------------------------------------------
    // Real-world drum kit configurations
    // -------------------------------------------------------------------------

    #[test]
    fn test_realistic_kit_piece_count() {
        // A typical professional kit has:
        // 1 kick, 1 snare, 1 hi-hat, 2-4 toms, 1-3 crashes, 1-2 rides
        let kit = DrumKit::standard_rock_kit();

        let kicks: Vec<_> = kit.pieces.iter().filter(|p| p.piece_type == DrumPieceType::Kick).collect();
        let snares: Vec<_> = kit.pieces.iter().filter(|p| p.piece_type == DrumPieceType::Snare).collect();
        let hihats: Vec<_> = kit.pieces.iter().filter(|p| p.piece_type == DrumPieceType::HiHat).collect();
        let toms: Vec<_> = kit.pieces.iter().filter(|p| matches!(p.piece_type, DrumPieceType::Tom | DrumPieceType::FloorTom)).collect();
        let crashes: Vec<_> = kit.pieces.iter().filter(|p| p.piece_type == DrumPieceType::Crash).collect();
        let rides: Vec<_> = kit.pieces.iter().filter(|p| p.piece_type == DrumPieceType::Ride).collect();

        assert_eq!(kicks.len(), 1);
        assert_eq!(snares.len(), 1);
        assert_eq!(hihats.len(), 1);
        assert!(toms.len() >= 3, "Should have at least 3 toms including floor tom");
        assert!(crashes.len() >= 1);
        assert!(rides.len() >= 1);
    }

    #[test]
    fn test_hihat_choke_behavior() {
        let mut kit = DrumKit::new("test", "Test");

        // All hi-hat states should be in same choke group
        kit.add_piece(
            DrumPiece::new("hh-closed", "Closed", DrumPieceType::HiHat)
                .with_midi_note(42)
                .with_choke_group(1),
        );
        kit.add_piece(
            DrumPiece::new("hh-open", "Open", DrumPieceType::HiHat)
                .with_midi_note(46)
                .with_choke_group(1),
        );
        kit.add_piece(
            DrumPiece::new("hh-pedal", "Pedal", DrumPieceType::HiHat)
                .with_midi_note(44)
                .with_choke_group(1),
        );

        // Snare should NOT be in hi-hat choke group
        kit.add_piece(DrumPiece::new("snare", "Snare", DrumPieceType::Snare));

        let hihat_choke = kit.find_choke_group(1);
        assert_eq!(hihat_choke.len(), 3);

        // When open hi-hat is hit, closed/pedal should choke it
        // When closed is hit, open should choke
        // This is handled by voice allocation in the player
    }

    #[test]
    fn test_snare_articulation_variety() {
        // A well-sampled snare should have multiple articulations
        let mut snare = DrumPiece::new("snare", "Snare", DrumPieceType::Snare);

        snare.add_articulation(ArticulationLayer::new(DrumArticulation::Center));
        snare.add_articulation(ArticulationLayer::new(DrumArticulation::RimShot));
        snare.add_articulation(ArticulationLayer::new(DrumArticulation::CrossStick));
        snare.add_articulation(ArticulationLayer::new(DrumArticulation::Edge));
        snare.add_articulation(ArticulationLayer::new(DrumArticulation::Flam));
        snare.add_articulation(ArticulationLayer::new(DrumArticulation::Drag));
        snare.add_articulation(ArticulationLayer::new(DrumArticulation::BuzzRoll));
        snare.add_articulation(ArticulationLayer::new(DrumArticulation::Ghost));

        assert_eq!(snare.articulations.len(), 8);
    }

    #[test]
    fn test_cymbal_bell_vs_bow() {
        let mut ride = DrumPiece::new("ride", "Ride", DrumPieceType::Ride);

        ride.add_articulation(ArticulationLayer::new(DrumArticulation::Bow));   // Main ride sound
        ride.add_articulation(ArticulationLayer::new(DrumArticulation::Bell));  // Bell hit
        ride.add_articulation(ArticulationLayer::new(DrumArticulation::CrashHit)); // Crash on ride

        assert!(ride.find_articulation(DrumArticulation::Bow).is_some());
        assert!(ride.find_articulation(DrumArticulation::Bell).is_some());
        assert!(ride.find_articulation(DrumArticulation::CrashHit).is_some());
    }

    #[test]
    fn test_tuning_affects_kit() {
        let default_kit = DrumKit::standard_rock_kit();
        let jazz_kit = DrumKit::jazz_kit();

        // Jazz kits are typically tuned higher
        assert!(jazz_kit.tuning > default_kit.tuning);
    }
}
