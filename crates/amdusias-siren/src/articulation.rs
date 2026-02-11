//! Articulation definitions for realistic instrument expression.

use serde::{Deserialize, Serialize};

/// Articulation types for instruments.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Articulation {
    /// Normal sustain (default).
    Sustain,
    /// Short, detached notes.
    Staccato,
    /// Very short notes with abrupt release.
    Staccatissimo,
    /// Smooth connection between notes.
    Legato,
    /// Accented attack.
    Accent,
    /// Strong accent with longer decay.
    Marcato,
    /// Gradually decreasing volume.
    Decrescendo,
    /// Gradually increasing volume.
    Crescendo,

    // Guitar-specific articulations
    /// Palm mute (muted with palm near bridge).
    PalmMute,
    /// Natural harmonic.
    NaturalHarmonic,
    /// Artificial/pinch harmonic.
    ArtificialHarmonic,
    /// Hammer-on (ascending legato).
    HammerOn,
    /// Pull-off (descending legato).
    PullOff,
    /// Slide up to note.
    SlideUp,
    /// Slide down from note.
    SlideDown,
    /// Slide between notes.
    SlideInto,
    /// Pitch bend.
    Bend {
        /// Bend amount in cents.
        cents: i16,
    },
    /// Pre-bend (bend before attack).
    PreBend {
        /// Bend amount in cents.
        cents: i16,
    },
    /// Vibrato.
    Vibrato {
        /// Depth in cents.
        depth: f32,
        /// Rate in Hz.
        rate: f32,
    },
    /// Let the note ring.
    LetRing,
    /// Dead/ghost note (muted).
    DeadNote,
    /// Tapped note.
    Tap,
    /// Tremolo picking.
    TremoloPicking {
        /// Speed in subdivisions per beat.
        speed: u8,
    },
    /// Whammy bar dive.
    WhammyDive {
        /// Depth in semitones.
        semitones: f32,
    },

    // String-specific
    /// Up-bow (strings).
    UpBow,
    /// Down-bow (strings).
    DownBow,
    /// Pizzicato (plucked strings).
    Pizzicato,
    /// Col legno (with wood of bow).
    ColLegno,
    /// Sul ponticello (near bridge).
    SulPonticello,
    /// Sul tasto (over fingerboard).
    SulTasto,

    // Wind-specific
    /// Tongued attack.
    Tongued,
    /// Slurred.
    Slurred,
    /// Flutter tongue.
    FlutterTongue,
}

impl Articulation {
    /// Returns true if this articulation is guitar-specific.
    #[must_use]
    pub const fn is_guitar_specific(&self) -> bool {
        matches!(
            self,
            Self::PalmMute
                | Self::NaturalHarmonic
                | Self::ArtificialHarmonic
                | Self::HammerOn
                | Self::PullOff
                | Self::SlideUp
                | Self::SlideDown
                | Self::SlideInto
                | Self::Bend { .. }
                | Self::PreBend { .. }
                | Self::Tap
                | Self::TremoloPicking { .. }
                | Self::WhammyDive { .. }
        )
    }

    /// Returns true if this articulation affects the note's attack.
    #[must_use]
    pub const fn affects_attack(&self) -> bool {
        matches!(
            self,
            Self::Staccato
                | Self::Staccatissimo
                | Self::Accent
                | Self::Marcato
                | Self::HammerOn
                | Self::PullOff
                | Self::Tap
                | Self::Tongued
        )
    }

    /// Returns true if this articulation affects the note's sustain.
    #[must_use]
    pub const fn affects_sustain(&self) -> bool {
        matches!(
            self,
            Self::PalmMute
                | Self::DeadNote
                | Self::Staccato
                | Self::Staccatissimo
                | Self::LetRing
        )
    }

    /// Returns the default duration modifier for this articulation.
    #[must_use]
    pub const fn duration_modifier(&self) -> f32 {
        match self {
            Self::Staccato => 0.5,
            Self::Staccatissimo => 0.25,
            Self::LetRing => 2.0,
            _ => 1.0,
        }
    }
}

impl Default for Articulation {
    fn default() -> Self {
        Self::Sustain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Phase 5 TDD: Articulation Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // Default and equality tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_articulation_default() {
        let art = Articulation::default();
        assert_eq!(art, Articulation::Sustain);
    }

    #[test]
    fn test_articulation_equality() {
        assert_eq!(Articulation::Sustain, Articulation::Sustain);
        assert_eq!(Articulation::PalmMute, Articulation::PalmMute);
        assert_ne!(Articulation::Sustain, Articulation::Staccato);
    }

    #[test]
    fn test_articulation_clone() {
        let art = Articulation::Vibrato { depth: 50.0, rate: 5.0 };
        let cloned = art.clone();
        assert_eq!(art, cloned);
    }

    // -------------------------------------------------------------------------
    // Guitar-specific articulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_guitar_specific_true() {
        assert!(Articulation::PalmMute.is_guitar_specific());
        assert!(Articulation::NaturalHarmonic.is_guitar_specific());
        assert!(Articulation::ArtificialHarmonic.is_guitar_specific());
        assert!(Articulation::HammerOn.is_guitar_specific());
        assert!(Articulation::PullOff.is_guitar_specific());
        assert!(Articulation::SlideUp.is_guitar_specific());
        assert!(Articulation::SlideDown.is_guitar_specific());
        assert!(Articulation::SlideInto.is_guitar_specific());
        assert!(Articulation::Bend { cents: 200 }.is_guitar_specific());
        assert!(Articulation::PreBend { cents: 100 }.is_guitar_specific());
        assert!(Articulation::Tap.is_guitar_specific());
        assert!(Articulation::TremoloPicking { speed: 4 }.is_guitar_specific());
        assert!(Articulation::WhammyDive { semitones: 12.0 }.is_guitar_specific());
    }

    #[test]
    fn test_is_guitar_specific_false() {
        assert!(!Articulation::Sustain.is_guitar_specific());
        assert!(!Articulation::Staccato.is_guitar_specific());
        assert!(!Articulation::Legato.is_guitar_specific());
        assert!(!Articulation::Accent.is_guitar_specific());
        assert!(!Articulation::Pizzicato.is_guitar_specific());
        assert!(!Articulation::FlutterTongue.is_guitar_specific());
    }

    // -------------------------------------------------------------------------
    // Attack-affecting articulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_affects_attack_true() {
        assert!(Articulation::Staccato.affects_attack());
        assert!(Articulation::Staccatissimo.affects_attack());
        assert!(Articulation::Accent.affects_attack());
        assert!(Articulation::Marcato.affects_attack());
        assert!(Articulation::HammerOn.affects_attack());
        assert!(Articulation::PullOff.affects_attack());
        assert!(Articulation::Tap.affects_attack());
        assert!(Articulation::Tongued.affects_attack());
    }

    #[test]
    fn test_affects_attack_false() {
        assert!(!Articulation::Sustain.affects_attack());
        assert!(!Articulation::Legato.affects_attack());
        assert!(!Articulation::PalmMute.affects_attack());
        assert!(!Articulation::Vibrato { depth: 50.0, rate: 5.0 }.affects_attack());
    }

    // -------------------------------------------------------------------------
    // Sustain-affecting articulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_affects_sustain_true() {
        assert!(Articulation::PalmMute.affects_sustain());
        assert!(Articulation::DeadNote.affects_sustain());
        assert!(Articulation::Staccato.affects_sustain());
        assert!(Articulation::Staccatissimo.affects_sustain());
        assert!(Articulation::LetRing.affects_sustain());
    }

    #[test]
    fn test_affects_sustain_false() {
        assert!(!Articulation::Sustain.affects_sustain());
        assert!(!Articulation::Accent.affects_sustain());
        assert!(!Articulation::HammerOn.affects_sustain());
        assert!(!Articulation::Bend { cents: 200 }.affects_sustain());
    }

    // -------------------------------------------------------------------------
    // Duration modifier tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_duration_modifier_staccato() {
        assert_eq!(Articulation::Staccato.duration_modifier(), 0.5);
    }

    #[test]
    fn test_duration_modifier_staccatissimo() {
        assert_eq!(Articulation::Staccatissimo.duration_modifier(), 0.25);
    }

    #[test]
    fn test_duration_modifier_let_ring() {
        assert_eq!(Articulation::LetRing.duration_modifier(), 2.0);
    }

    #[test]
    fn test_duration_modifier_default() {
        assert_eq!(Articulation::Sustain.duration_modifier(), 1.0);
        assert_eq!(Articulation::Legato.duration_modifier(), 1.0);
        assert_eq!(Articulation::PalmMute.duration_modifier(), 1.0);
        assert_eq!(Articulation::HammerOn.duration_modifier(), 1.0);
    }

    // -------------------------------------------------------------------------
    // Bend articulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_bend_articulation() {
        let bend = Articulation::Bend { cents: 200 };
        match bend {
            Articulation::Bend { cents } => assert_eq!(cents, 200),
            _ => panic!("Expected Bend articulation"),
        }
    }

    #[test]
    fn test_prebend_articulation() {
        let prebend = Articulation::PreBend { cents: 100 };
        match prebend {
            Articulation::PreBend { cents } => assert_eq!(cents, 100),
            _ => panic!("Expected PreBend articulation"),
        }
    }

    #[test]
    fn test_bend_whole_tone() {
        // Whole tone = 200 cents
        let bend = Articulation::Bend { cents: 200 };
        assert!(bend.is_guitar_specific());
    }

    #[test]
    fn test_bend_half_tone() {
        // Half tone = 100 cents
        let bend = Articulation::Bend { cents: 100 };
        assert!(bend.is_guitar_specific());
    }

    // -------------------------------------------------------------------------
    // Vibrato articulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_vibrato_articulation() {
        let vibrato = Articulation::Vibrato { depth: 50.0, rate: 6.0 };
        match vibrato {
            Articulation::Vibrato { depth, rate } => {
                assert_eq!(depth, 50.0);
                assert_eq!(rate, 6.0);
            }
            _ => panic!("Expected Vibrato articulation"),
        }
    }

    #[test]
    fn test_vibrato_subtle() {
        // Subtle vibrato for classical guitar
        let vibrato = Articulation::Vibrato { depth: 15.0, rate: 4.0 };
        match vibrato {
            Articulation::Vibrato { depth, rate } => {
                assert!(depth < 30.0);
                assert!(rate < 5.0);
            }
            _ => panic!("Expected Vibrato"),
        }
    }

    #[test]
    fn test_vibrato_aggressive() {
        // Aggressive vibrato for rock guitar
        let vibrato = Articulation::Vibrato { depth: 100.0, rate: 7.0 };
        match vibrato {
            Articulation::Vibrato { depth, rate } => {
                assert!(depth > 50.0);
                assert!(rate > 5.0);
            }
            _ => panic!("Expected Vibrato"),
        }
    }

    // -------------------------------------------------------------------------
    // Tremolo picking tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_tremolo_picking() {
        let tremolo = Articulation::TremoloPicking { speed: 4 };
        match tremolo {
            Articulation::TremoloPicking { speed } => assert_eq!(speed, 4),
            _ => panic!("Expected TremoloPicking"),
        }
    }

    #[test]
    fn test_tremolo_picking_16th_notes() {
        let tremolo = Articulation::TremoloPicking { speed: 4 };
        assert!(tremolo.is_guitar_specific());
    }

    // -------------------------------------------------------------------------
    // Whammy bar tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_whammy_dive() {
        let whammy = Articulation::WhammyDive { semitones: 12.0 };
        match whammy {
            Articulation::WhammyDive { semitones } => assert_eq!(semitones, 12.0),
            _ => panic!("Expected WhammyDive"),
        }
    }

    #[test]
    fn test_whammy_dive_octave() {
        let whammy = Articulation::WhammyDive { semitones: 12.0 };
        assert!(whammy.is_guitar_specific());
    }

    // -------------------------------------------------------------------------
    // String articulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_string_articulations() {
        assert!(!Articulation::UpBow.is_guitar_specific());
        assert!(!Articulation::DownBow.is_guitar_specific());
        assert!(!Articulation::Pizzicato.is_guitar_specific());
        assert!(!Articulation::ColLegno.is_guitar_specific());
        assert!(!Articulation::SulPonticello.is_guitar_specific());
        assert!(!Articulation::SulTasto.is_guitar_specific());
    }

    // -------------------------------------------------------------------------
    // Wind articulation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_wind_articulations() {
        assert!(!Articulation::Tongued.is_guitar_specific());
        assert!(!Articulation::Slurred.is_guitar_specific());
        assert!(!Articulation::FlutterTongue.is_guitar_specific());
    }

    // -------------------------------------------------------------------------
    // Common articulation combinations (real-world scenarios)
    // -------------------------------------------------------------------------

    #[test]
    fn test_articulation_categories() {
        // Check that categories are correctly identified
        let general_articulations = [
            Articulation::Sustain,
            Articulation::Staccato,
            Articulation::Legato,
            Articulation::Accent,
        ];

        for art in &general_articulations {
            assert!(!art.is_guitar_specific());
        }
    }
}
