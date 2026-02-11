//! Instrument player for real-time playback.

use crate::{
    articulation::Articulation,
    instrument::Instrument,
    sample::Sample,
    voice::VoiceAllocator,
};
use std::collections::HashMap;

/// Instrument player for real-time sample playback.
pub struct InstrumentPlayer {
    /// The instrument being played.
    instrument: Instrument,
    /// Voice allocator.
    allocator: VoiceAllocator,
    /// Loaded samples.
    samples: HashMap<crate::sample::SampleId, Sample>,
    /// Sample rate.
    sample_rate: f32,
}

impl InstrumentPlayer {
    /// Creates a new instrument player.
    #[must_use]
    pub fn new(instrument: Instrument, sample_rate: f32) -> Self {
        let max_voices = instrument.max_voices;
        Self {
            instrument,
            allocator: VoiceAllocator::new(max_voices, sample_rate),
            samples: HashMap::new(),
            sample_rate,
        }
    }

    /// Loads a sample into the player.
    pub fn load_sample(&mut self, sample: Sample) {
        self.samples.insert(sample.id, sample);
    }

    /// Triggers a note.
    pub fn note_on(&mut self, note: u8, velocity: u8) {
        self.note_on_with_articulation(note, velocity, Articulation::default());
    }

    /// Triggers a note with a specific articulation.
    pub fn note_on_with_articulation(
        &mut self,
        note: u8,
        velocity: u8,
        articulation: Articulation,
    ) {
        // Find matching zones
        let zones: Vec<_> = self
            .instrument
            .find_zones(note, velocity, articulation)
            .enumerate()
            .collect();

        if zones.is_empty() {
            return;
        }

        // Use first matching zone (could be round-robin in future)
        let (zone_index, zone) = &zones[0];

        // Allocate a voice
        if let Some(voice) = self.allocator.allocate() {
            voice.trigger(note, velocity, articulation, zone, *zone_index);
        }
    }

    /// Releases a note.
    pub fn note_off(&mut self, note: u8) {
        if let Some(voice) = self.allocator.find_voice(note) {
            voice.release();
        }
    }

    /// Releases all notes.
    pub fn all_notes_off(&mut self) {
        self.allocator.release_all();
    }

    /// Processes audio into the output buffer.
    ///
    /// The buffer should be interleaved stereo (L, R, L, R, ...).
    pub fn process(&mut self, output: &mut [f32]) {
        let frames = output.len() / 2;

        for frame in 0..frames {
            let mut left = 0.0;
            let mut right = 0.0;

            for voice in self.allocator.active_voices() {
                let zone_index = voice.zone_index();
                if let Some(zone) = self.instrument.zones.get(zone_index) {
                    if let Some(sample) = self.samples.get(&zone.sample_id) {
                        let (l, r) = voice.process(&sample.data, sample.channels as usize);
                        left += l;
                        right += r;
                    }
                }
            }

            output[frame * 2] = left;
            output[frame * 2 + 1] = right;
        }
    }

    /// Returns the number of active voices.
    #[must_use]
    pub fn active_voice_count(&self) -> usize {
        self.allocator.active_count()
    }

    /// Returns a reference to the instrument.
    #[must_use]
    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }
}
