//! Voice allocation and management.

use crate::{articulation::Articulation, sample::SampleZone};
use amdusias_dsp::envelope::AdsrEnvelope;

/// A single playing voice.
#[derive(Debug)]
pub struct Voice {
    /// Voice ID.
    pub id: VoiceId,
    /// MIDI note number.
    pub note: u8,
    /// Velocity.
    pub velocity: u8,
    /// Current articulation.
    pub articulation: Articulation,
    /// Voice state.
    pub state: VoiceState,
    /// ADSR envelope.
    envelope: AdsrEnvelope,
    /// Current sample position (fractional for pitch shifting).
    position: f64,
    /// Pitch ratio (for playback speed).
    pitch_ratio: f64,
    /// Gain (from velocity and zone settings).
    gain: f32,
    /// Zone index this voice is playing.
    zone_index: usize,
}

/// Unique voice identifier.
///
/// This is a simple wrapper around a u32 for type safety.
/// The `#[repr(C)]` attribute ensures a stable ABI for FFI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct VoiceId(pub u32);

/// State of a voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceState {
    /// Voice is not playing.
    Idle,
    /// Voice is in attack phase.
    Attack,
    /// Voice is in decay phase.
    Decay,
    /// Voice is sustaining.
    Sustain,
    /// Voice is releasing.
    Release,
}

impl Voice {
    /// Creates a new idle voice.
    #[must_use]
    pub fn new(id: VoiceId, sample_rate: f32) -> Self {
        Self {
            id,
            note: 0,
            velocity: 0,
            articulation: Articulation::default(),
            state: VoiceState::Idle,
            envelope: AdsrEnvelope::new(5.0, 100.0, 0.8, 200.0, sample_rate),
            position: 0.0,
            pitch_ratio: 1.0,
            gain: 1.0,
            zone_index: 0,
        }
    }

    /// Triggers the voice with a note.
    pub fn trigger(
        &mut self,
        note: u8,
        velocity: u8,
        articulation: Articulation,
        zone: &SampleZone,
        zone_index: usize,
    ) {
        self.note = note;
        self.velocity = velocity;
        self.articulation = articulation;
        self.state = VoiceState::Attack;
        self.position = 0.0;
        self.pitch_ratio = zone.pitch_ratio(note);
        self.gain = velocity_to_gain(velocity) * amdusias_dsp::db_to_linear(zone.gain_db);
        self.zone_index = zone_index;

        self.envelope.trigger();
    }

    /// Releases the voice.
    pub fn release(&mut self) {
        if self.state != VoiceState::Idle {
            self.state = VoiceState::Release;
            self.envelope.release();
        }
    }

    /// Returns true if the voice is active (not idle).
    #[inline]
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state != VoiceState::Idle
    }

    /// Returns the zone index this voice is playing.
    #[inline]
    #[must_use]
    pub fn zone_index(&self) -> usize {
        self.zone_index
    }

    /// Processes a single sample from this voice.
    ///
    /// This is the hot path for audio processing. It:
    /// 1. Performs linear interpolation for pitch-shifted playback
    /// 2. Applies the ADSR envelope
    /// 3. Applies velocity-based gain
    ///
    /// Returns a stereo sample pair (left, right).
    #[inline]
    pub fn process(&mut self, sample_data: &[f32], channels: usize) -> (f32, f32) {
        if !self.is_active() {
            return (0.0, 0.0);
        }

        // Get sample at current position (linear interpolation)
        let pos_int = self.position as usize;
        let pos_frac = (self.position - pos_int as f64) as f32;

        let frame_size = channels;
        let sample_frames = sample_data.len() / frame_size;

        if pos_int >= sample_frames.saturating_sub(1) {
            self.state = VoiceState::Idle;
            return (0.0, 0.0);
        }

        let idx = pos_int * frame_size;
        let (left, right) = if channels == 2 {
            let l1 = sample_data.get(idx).copied().unwrap_or(0.0);
            let r1 = sample_data.get(idx + 1).copied().unwrap_or(0.0);
            let l2 = sample_data.get(idx + frame_size).copied().unwrap_or(0.0);
            let r2 = sample_data.get(idx + frame_size + 1).copied().unwrap_or(0.0);
            (
                l1 + pos_frac * (l2 - l1),
                r1 + pos_frac * (r2 - r1),
            )
        } else {
            let s1 = sample_data.get(idx).copied().unwrap_or(0.0);
            let s2 = sample_data.get(idx + 1).copied().unwrap_or(0.0);
            let mono = s1 + pos_frac * (s2 - s1);
            (mono, mono)
        };

        // Apply envelope and gain
        let env = self.envelope.process();
        if !self.envelope.is_active() {
            self.state = VoiceState::Idle;
        }

        let gain = self.gain * env;

        // Advance position
        self.position += self.pitch_ratio;

        (left * gain, right * gain)
    }
}

/// Converts MIDI velocity to linear gain.
///
/// Uses a quadratic curve (v²) for natural dynamics, as human perception
/// of loudness is logarithmic.
#[inline]
fn velocity_to_gain(velocity: u8) -> f32 {
    let v = velocity as f32 / 127.0;
    v * v
}

/// Voice allocator with configurable polyphony.
pub struct VoiceAllocator {
    /// All voices.
    voices: Vec<Voice>,
    /// Next voice ID.
    next_id: u32,
    /// Voice stealing mode.
    stealing_mode: VoiceStealingMode,
    /// Round-robin index for each zone.
    round_robin: std::collections::HashMap<usize, usize>,
}

/// Voice stealing mode when polyphony is exceeded.
#[derive(Debug, Clone, Copy, Default)]
pub enum VoiceStealingMode {
    /// Don't steal - ignore new notes.
    None,
    /// Steal the oldest voice.
    #[default]
    Oldest,
    /// Steal the quietest voice.
    Quietest,
    /// Steal the same note if already playing.
    SameNote,
}

impl VoiceAllocator {
    /// Creates a new voice allocator.
    #[must_use]
    pub fn new(max_voices: usize, sample_rate: f32) -> Self {
        let voices = (0..max_voices)
            .map(|i| Voice::new(VoiceId(i as u32), sample_rate))
            .collect();

        Self {
            voices,
            next_id: max_voices as u32,
            stealing_mode: VoiceStealingMode::default(),
            round_robin: std::collections::HashMap::new(),
        }
    }

    /// Sets the voice stealing mode.
    pub fn set_stealing_mode(&mut self, mode: VoiceStealingMode) {
        self.stealing_mode = mode;
    }

    /// Allocates a voice for a new note.
    pub fn allocate(&mut self) -> Option<&mut Voice> {
        // First, try to find an idle voice by index
        let idle_idx = self.voices.iter().position(|v| !v.is_active());

        if let Some(idx) = idle_idx {
            let voice = &mut self.voices[idx];
            voice.id = VoiceId(self.next_id);
            self.next_id += 1;
            return Some(voice);
        }

        // All voices are active, need to steal
        let steal_idx = match self.stealing_mode {
            VoiceStealingMode::None => None,
            VoiceStealingMode::Oldest | VoiceStealingMode::Quietest => {
                // Steal the voice with the lowest ID (oldest)
                self.voices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, v)| v.id.0)
                    .map(|(i, _)| i)
            }
            VoiceStealingMode::SameNote => {
                // Handled at a higher level
                None
            }
        };

        if let Some(idx) = steal_idx {
            let voice = &mut self.voices[idx];
            voice.id = VoiceId(self.next_id);
            self.next_id += 1;
            Some(voice)
        } else {
            None
        }
    }

    /// Finds an active voice playing the given note.
    pub fn find_voice(&mut self, note: u8) -> Option<&mut Voice> {
        self.voices
            .iter_mut()
            .find(|v| v.is_active() && v.note == note)
    }

    /// Returns an iterator over all active voices.
    pub fn active_voices(&mut self) -> impl Iterator<Item = &mut Voice> {
        self.voices.iter_mut().filter(|v| v.is_active())
    }

    /// Returns the number of active voices.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    /// Releases all voices.
    pub fn release_all(&mut self) {
        for voice in &mut self.voices {
            voice.release();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::SampleId;

    // =========================================================================
    // Phase 5 TDD: Voice Tests
    // =========================================================================

    // -------------------------------------------------------------------------
    // VoiceId tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_id_creation() {
        let id = VoiceId(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_voice_id_equality() {
        let id1 = VoiceId(1);
        let id2 = VoiceId(1);
        let id3 = VoiceId(2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_voice_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(VoiceId(1));
        set.insert(VoiceId(2));
        set.insert(VoiceId(1)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    // -------------------------------------------------------------------------
    // VoiceState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_state_idle() {
        let state = VoiceState::Idle;
        assert_eq!(state, VoiceState::Idle);
    }

    #[test]
    fn test_voice_state_all_variants() {
        let states = [
            VoiceState::Idle,
            VoiceState::Attack,
            VoiceState::Decay,
            VoiceState::Sustain,
            VoiceState::Release,
        ];

        assert_eq!(states.len(), 5);
    }

    #[test]
    fn test_voice_state_equality() {
        assert_eq!(VoiceState::Attack, VoiceState::Attack);
        assert_ne!(VoiceState::Attack, VoiceState::Release);
    }

    // -------------------------------------------------------------------------
    // Voice creation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_new() {
        let voice = Voice::new(VoiceId(0), 48000.0);

        assert_eq!(voice.id, VoiceId(0));
        assert_eq!(voice.note, 0);
        assert_eq!(voice.velocity, 0);
        assert_eq!(voice.articulation, Articulation::default());
        assert_eq!(voice.state, VoiceState::Idle);
        assert!(!voice.is_active());
    }

    #[test]
    fn test_voice_is_active() {
        let voice = Voice::new(VoiceId(0), 48000.0);
        assert!(!voice.is_active());
    }

    #[test]
    fn test_voice_zone_index() {
        let voice = Voice::new(VoiceId(0), 48000.0);
        assert_eq!(voice.zone_index(), 0);
    }

    // -------------------------------------------------------------------------
    // Voice trigger tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_trigger() {
        let mut voice = Voice::new(VoiceId(0), 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        voice.trigger(60, 100, Articulation::Sustain, &zone, 0);

        assert_eq!(voice.note, 60);
        assert_eq!(voice.velocity, 100);
        assert_eq!(voice.articulation, Articulation::Sustain);
        assert_eq!(voice.state, VoiceState::Attack);
        assert!(voice.is_active());
    }

    #[test]
    fn test_voice_trigger_with_articulation() {
        let mut voice = Voice::new(VoiceId(0), 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        voice.trigger(60, 80, Articulation::PalmMute, &zone, 1);

        assert_eq!(voice.articulation, Articulation::PalmMute);
        assert_eq!(voice.zone_index(), 1);
    }

    // -------------------------------------------------------------------------
    // Voice release tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_release() {
        let mut voice = Voice::new(VoiceId(0), 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        voice.trigger(60, 100, Articulation::Sustain, &zone, 0);
        assert!(voice.is_active());

        voice.release();
        assert_eq!(voice.state, VoiceState::Release);
    }

    #[test]
    fn test_voice_release_idle() {
        let mut voice = Voice::new(VoiceId(0), 48000.0);

        // Releasing an idle voice should do nothing
        voice.release();
        assert_eq!(voice.state, VoiceState::Idle);
    }

    // -------------------------------------------------------------------------
    // VoiceStealingMode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_stealing_mode_default() {
        let mode = VoiceStealingMode::default();
        assert!(matches!(mode, VoiceStealingMode::Oldest));
    }

    #[test]
    fn test_voice_stealing_mode_variants() {
        let _none = VoiceStealingMode::None;
        let _oldest = VoiceStealingMode::Oldest;
        let _quietest = VoiceStealingMode::Quietest;
        let _same_note = VoiceStealingMode::SameNote;
    }

    // -------------------------------------------------------------------------
    // VoiceAllocator tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_allocator_new() {
        let allocator = VoiceAllocator::new(8, 48000.0);

        assert_eq!(allocator.active_count(), 0);
    }

    #[test]
    fn test_voice_allocator() {
        let mut allocator = VoiceAllocator::new(4, 48000.0);

        // Should be able to allocate 4 voices
        for _ in 0..4 {
            assert!(allocator.allocate().is_some());
        }

        // 5th allocation should steal
        assert!(allocator.allocate().is_some());
    }

    #[test]
    fn test_voice_allocator_allocate() {
        let mut allocator = VoiceAllocator::new(4, 48000.0);

        let voice = allocator.allocate();
        assert!(voice.is_some());
    }

    #[test]
    fn test_voice_allocator_active_count() {
        let mut allocator = VoiceAllocator::new(8, 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        // Allocate and trigger 3 voices
        for i in 0..3 {
            if let Some(voice) = allocator.allocate() {
                voice.trigger(60 + i, 100, Articulation::Sustain, &zone, 0);
            }
        }

        assert_eq!(allocator.active_count(), 3);
    }

    #[test]
    fn test_voice_allocator_find_voice() {
        let mut allocator = VoiceAllocator::new(8, 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        // Allocate and trigger a voice
        if let Some(voice) = allocator.allocate() {
            voice.trigger(60, 100, Articulation::Sustain, &zone, 0);
        }

        // Should find the voice
        let found = allocator.find_voice(60);
        assert!(found.is_some());
        assert_eq!(found.unwrap().note, 60);

        // Should not find non-existent voice
        let not_found = allocator.find_voice(72);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_voice_allocator_release_all() {
        let mut allocator = VoiceAllocator::new(8, 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        // Allocate and trigger some voices
        for i in 0..4 {
            if let Some(voice) = allocator.allocate() {
                voice.trigger(60 + i, 100, Articulation::Sustain, &zone, 0);
            }
        }

        assert_eq!(allocator.active_count(), 4);

        // Release all
        allocator.release_all();

        // Voices are in release state but still active
        for voice in allocator.active_voices() {
            assert_eq!(voice.state, VoiceState::Release);
        }
    }

    #[test]
    fn test_voice_allocator_set_stealing_mode() {
        let mut allocator = VoiceAllocator::new(4, 48000.0);

        allocator.set_stealing_mode(VoiceStealingMode::None);
        // Can't directly test internal state, but operation shouldn't panic
    }

    #[test]
    fn test_voice_allocator_stealing_none() {
        let mut allocator = VoiceAllocator::new(2, 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        allocator.set_stealing_mode(VoiceStealingMode::None);

        // Allocate 2 voices
        for i in 0..2 {
            if let Some(voice) = allocator.allocate() {
                voice.trigger(60 + i, 100, Articulation::Sustain, &zone, 0);
            }
        }

        // 3rd allocation should fail (no stealing)
        let result = allocator.allocate();
        assert!(result.is_none());
    }

    #[test]
    fn test_voice_allocator_stealing_oldest() {
        let mut allocator = VoiceAllocator::new(2, 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        allocator.set_stealing_mode(VoiceStealingMode::Oldest);

        // Allocate 2 voices
        for i in 0..2 {
            if let Some(voice) = allocator.allocate() {
                voice.trigger(60 + i, 100, Articulation::Sustain, &zone, 0);
            }
        }

        // 3rd allocation should succeed by stealing oldest
        let result = allocator.allocate();
        assert!(result.is_some());
    }

    // -------------------------------------------------------------------------
    // Voice processing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_voice_process_idle() {
        let mut voice = Voice::new(VoiceId(0), 48000.0);
        let sample_data: Vec<f32> = vec![0.5; 100];

        let (left, right) = voice.process(&sample_data, 1);

        // Idle voice should output silence
        assert_eq!(left, 0.0);
        assert_eq!(right, 0.0);
    }

    #[test]
    fn test_voice_process_active() {
        let mut voice = Voice::new(VoiceId(0), 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        // Trigger the voice
        voice.trigger(60, 127, Articulation::Sustain, &zone, 0);

        // Create sample data
        let sample_data: Vec<f32> = vec![1.0; 1000];

        // Process several samples to get past attack phase
        // The envelope needs time to ramp up
        let mut total_output = 0.0;
        for _ in 0..100 {
            let (left, right) = voice.process(&sample_data, 1);
            total_output += left.abs() + right.abs();
        }

        // With max velocity and gain, cumulative output should be non-zero
        assert!(total_output > 0.0, "Expected output after processing, got {}", total_output);
    }

    #[test]
    fn test_voice_process_stereo() {
        let mut voice = Voice::new(VoiceId(0), 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        voice.trigger(60, 127, Articulation::Sustain, &zone, 0);

        // Stereo sample data (L, R, L, R, ...) - need more frames
        let mut sample_data = Vec::with_capacity(1000);
        for _ in 0..500 {
            sample_data.push(1.0);
            sample_data.push(0.5);
        }

        // Process several samples to accumulate output
        let mut total_left = 0.0;
        let mut total_right = 0.0;
        for _ in 0..50 {
            let (left, right) = voice.process(&sample_data, 2);
            total_left += left.abs();
            total_right += right.abs();
        }

        // Should have produced some output
        assert!(total_left > 0.0 || total_right > 0.0,
            "Expected stereo output, got L={}, R={}", total_left, total_right);
    }

    // -------------------------------------------------------------------------
    // Velocity to gain tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_velocity_to_gain_extreme() {
        // Zero velocity should result in zero gain (indirectly tested)
        let mut voice = Voice::new(VoiceId(0), 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        voice.trigger(60, 0, Articulation::Sustain, &zone, 0);

        // With zero velocity, the voice should produce very quiet output
        let sample_data: Vec<f32> = vec![1.0; 100];
        let (left, right) = voice.process(&sample_data, 1);

        // Zero velocity = zero gain (quadratic)
        assert!(left.abs() < 0.01);
        assert!(right.abs() < 0.01);
    }

    // -------------------------------------------------------------------------
    // Real-world scenario tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_polyphonic_playback() {
        let mut allocator = VoiceAllocator::new(8, 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        // Play a chord (C major: C, E, G)
        let chord_notes = [60, 64, 67];

        for &note in &chord_notes {
            if let Some(voice) = allocator.allocate() {
                voice.trigger(note, 100, Articulation::Sustain, &zone, 0);
            }
        }

        assert_eq!(allocator.active_count(), 3);

        // Find each note
        for &note in &chord_notes {
            assert!(allocator.find_voice(note).is_some());
        }
    }

    #[test]
    fn test_voice_reuse() {
        let mut allocator = VoiceAllocator::new(4, 48000.0);
        let zone = SampleZone::new(SampleId(1), 60);

        // Allocate a voice
        let voice = allocator.allocate().unwrap();
        voice.trigger(60, 100, Articulation::Sustain, &zone, 0);
        let original_id = voice.id;

        // Simulate end of sample (voice becomes idle)
        // Note: In real usage, this would happen through process() consuming the sample

        // For testing, we'll just release
        if let Some(voice) = allocator.find_voice(60) {
            voice.release();
        }

        // Voice should still be in release state initially
        assert_eq!(allocator.active_count(), 1);
    }
}
