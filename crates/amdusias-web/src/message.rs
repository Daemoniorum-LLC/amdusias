//! Message types for communication between main thread and AudioWorklet.

use serde::{Deserialize, Serialize};

/// Message type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Parameter change.
    Param,
    /// Note on event.
    NoteOn,
    /// Note off event.
    NoteOff,
    /// All notes off.
    AllNotesOff,
    /// Transport command (play/pause/stop).
    Transport,
    /// Meter data (from processor to main thread).
    Meter,
    /// Error message.
    Error,
}

/// A message sent between main thread and AudioWorklet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message type.
    pub msg_type: MessageType,
    /// Parameter ID (for Param messages).
    pub param_id: Option<u32>,
    /// Value (for Param messages).
    pub value: Option<f32>,
    /// MIDI note number (for NoteOn/NoteOff).
    pub note: Option<u8>,
    /// MIDI velocity (for NoteOn).
    pub velocity: Option<u8>,
    /// Error message text.
    pub error: Option<String>,
}

impl Message {
    /// Creates a parameter change message.
    #[must_use]
    pub fn param(param_id: u32, value: f32) -> Self {
        Self {
            msg_type: MessageType::Param,
            param_id: Some(param_id),
            value: Some(value),
            note: None,
            velocity: None,
            error: None,
        }
    }

    /// Creates a note on message.
    #[must_use]
    pub fn note_on(note: u8, velocity: u8) -> Self {
        Self {
            msg_type: MessageType::NoteOn,
            param_id: None,
            value: None,
            note: Some(note),
            velocity: Some(velocity),
            error: None,
        }
    }

    /// Creates a note off message.
    #[must_use]
    pub fn note_off(note: u8) -> Self {
        Self {
            msg_type: MessageType::NoteOff,
            param_id: None,
            value: None,
            note: Some(note),
            velocity: None,
            error: None,
        }
    }

    /// Creates an all notes off message.
    #[must_use]
    pub fn all_notes_off() -> Self {
        Self {
            msg_type: MessageType::AllNotesOff,
            param_id: None,
            value: None,
            note: None,
            velocity: None,
            error: None,
        }
    }

    /// Creates an error message.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            msg_type: MessageType::Error,
            param_id: None,
            value: None,
            note: None,
            velocity: None,
            error: Some(message.into()),
        }
    }
}

/// Well-known parameter IDs.
pub mod params {
    /// Master gain in dB.
    pub const MASTER_GAIN: u32 = 0;
    /// Reverb mix (0-1).
    pub const REVERB_MIX: u32 = 1;
    /// Reverb room size (0-1).
    pub const REVERB_SIZE: u32 = 2;
    /// Compressor threshold in dB.
    pub const COMP_THRESHOLD: u32 = 10;
    /// Compressor ratio.
    pub const COMP_RATIO: u32 = 11;
    /// Compressor attack in ms.
    pub const COMP_ATTACK: u32 = 12;
    /// Compressor release in ms.
    pub const COMP_RELEASE: u32 = 13;
}
