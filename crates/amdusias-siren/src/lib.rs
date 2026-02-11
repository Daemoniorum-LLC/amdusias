//! # amdusias-siren
//!
//! *The Siren's call is irresistible.*
//!
//! Siren is the sample instrument engine for Amdusias, providing enchanting
//! multi-sample playback with realistic articulations and modeling.
//!
//! ## Features
//!
//! - **Multi-sample instruments** with velocity layers and round-robin
//! - **Guitar modeling** (per-string, pickups, amps, cabinets)
//! - **Drum kits** with multi-mic positions and GM mapping
//! - **Articulation support** (sustain, staccato, palm mute, harmonics, slides)
//! - **Voice allocation** with configurable polyphony and stealing
//! - **Real-time parameter control** for expression and dynamics
//!
//! ## Example
//!
//! ```rust,ignore
//! use amdusias_siren::{Instrument, InstrumentPlayer, Note};
//!
//! // Load an instrument
//! let guitar = Instrument::load("instruments/electric-guitar.json")?;
//! let mut player = InstrumentPlayer::new(guitar, 48000.0);
//!
//! // Trigger a note
//! player.note_on(Note::new(64, 100)); // E4 at velocity 100
//!
//! // Process audio
//! player.process(&mut output_buffer);
//!
//! // Release the note
//! player.note_off(64);
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod articulation;
pub mod drum;
pub mod guitar;
pub mod instrument;
pub mod player;
pub mod sample;
pub mod voice;

pub use articulation::Articulation;
pub use drum::{DrumArticulation, DrumKit, DrumPiece, DrumPieceType, GmDrumMap, MicPosition};
pub use guitar::{GuitarInstrument, GuitarString};
pub use instrument::{Instrument, InstrumentCategory};
pub use player::InstrumentPlayer;
pub use sample::{Sample, SampleZone};
pub use voice::{Voice, VoiceAllocator};
