//! Envelope detection for dynamics processing.

use crate::Sample;

/// Envelope detection mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeMode {
    /// Peak detection (follows peaks).
    Peak,
    /// RMS detection (measures average power).
    Rms,
    /// True peak with oversampling.
    TruePeak,
}

/// Envelope detector with configurable attack/release.
#[derive(Debug, Clone)]
pub struct EnvelopeDetector {
    /// Current envelope value.
    envelope: f32,
    /// Attack coefficient.
    attack_coeff: f32,
    /// Release coefficient.
    release_coeff: f32,
    /// Detection mode.
    mode: EnvelopeMode,
    /// RMS accumulator for RMS mode.
    rms_acc: f32,
    /// RMS window size.
    rms_window: usize,
    /// Current position in RMS window.
    rms_pos: usize,
}

impl EnvelopeDetector {
    /// Creates a new envelope detector.
    ///
    /// # Arguments
    ///
    /// - `attack_ms`: Attack time in milliseconds.
    /// - `release_ms`: Release time in milliseconds.
    /// - `sample_rate`: Sample rate in Hz.
    /// - `mode`: Detection mode.
    #[must_use]
    pub fn new(attack_ms: f32, release_ms: f32, sample_rate: f32, mode: EnvelopeMode) -> Self {
        Self {
            envelope: 0.0,
            attack_coeff: Self::time_to_coeff(attack_ms, sample_rate),
            release_coeff: Self::time_to_coeff(release_ms, sample_rate),
            mode,
            rms_acc: 0.0,
            rms_window: (sample_rate * 0.01) as usize, // 10ms window
            rms_pos: 0,
        }
    }

    /// Converts time constant to coefficient.
    fn time_to_coeff(time_ms: f32, sample_rate: f32) -> f32 {
        if time_ms <= 0.0 {
            0.0
        } else {
            (-1.0 / (time_ms * sample_rate / 1000.0)).exp()
        }
    }

    /// Sets attack time.
    pub fn set_attack(&mut self, attack_ms: f32, sample_rate: f32) {
        self.attack_coeff = Self::time_to_coeff(attack_ms, sample_rate);
    }

    /// Sets release time.
    pub fn set_release(&mut self, release_ms: f32, sample_rate: f32) {
        self.release_coeff = Self::time_to_coeff(release_ms, sample_rate);
    }

    /// Processes a sample and returns the envelope value.
    pub fn process(&mut self, input: Sample) -> f32 {
        let input_level = match self.mode {
            EnvelopeMode::Peak | EnvelopeMode::TruePeak => input.abs(),
            EnvelopeMode::Rms => {
                self.rms_acc += input * input;
                self.rms_pos += 1;

                if self.rms_pos >= self.rms_window {
                    let rms = (self.rms_acc / self.rms_window as f32).sqrt();
                    self.rms_acc = 0.0;
                    self.rms_pos = 0;
                    rms
                } else {
                    return self.envelope;
                }
            }
        };

        // Branching envelope follower
        let coeff = if input_level > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };

        self.envelope = input_level + coeff * (self.envelope - input_level);
        self.envelope
    }

    /// Returns the current envelope value without processing.
    #[must_use]
    pub fn current(&self) -> f32 {
        self.envelope
    }

    /// Resets the envelope detector.
    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.rms_acc = 0.0;
        self.rms_pos = 0;
    }
}

/// ADSR envelope generator for synthesizers.
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    /// Attack time in samples.
    attack_samples: f32,
    /// Decay time in samples.
    decay_samples: f32,
    /// Sustain level (0.0 to 1.0).
    sustain_level: f32,
    /// Release time in samples.
    release_samples: f32,
    /// Current envelope stage.
    stage: AdsrStage,
    /// Current position in stage.
    stage_pos: f32,
    /// Current envelope value.
    value: f32,
    /// Value at start of release.
    release_start_value: f32,
}

/// ADSR envelope stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AdsrStage {
    /// Idle (envelope = 0).
    Idle,
    /// Attack (rising to 1.0).
    Attack,
    /// Decay (falling to sustain).
    Decay,
    /// Sustain (holding at sustain level).
    Sustain,
    /// Release (falling to 0).
    Release,
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope.
    #[must_use]
    pub fn new(attack_ms: f32, decay_ms: f32, sustain: f32, release_ms: f32, sample_rate: f32) -> Self {
        let ms_to_samples = sample_rate / 1000.0;
        Self {
            attack_samples: attack_ms * ms_to_samples,
            decay_samples: decay_ms * ms_to_samples,
            sustain_level: sustain.clamp(0.0, 1.0),
            release_samples: release_ms * ms_to_samples,
            stage: AdsrStage::Idle,
            stage_pos: 0.0,
            value: 0.0,
            release_start_value: 0.0,
        }
    }

    /// Triggers the envelope (note on).
    pub fn trigger(&mut self) {
        self.stage = AdsrStage::Attack;
        self.stage_pos = 0.0;
    }

    /// Releases the envelope (note off).
    pub fn release(&mut self) {
        if self.stage != AdsrStage::Idle {
            self.release_start_value = self.value;
            self.stage = AdsrStage::Release;
            self.stage_pos = 0.0;
        }
    }

    /// Processes one sample and returns the envelope value.
    pub fn process(&mut self) -> f32 {
        match self.stage {
            AdsrStage::Idle => {
                self.value = 0.0;
            }
            AdsrStage::Attack => {
                self.value = self.stage_pos / self.attack_samples;
                self.stage_pos += 1.0;

                if self.stage_pos >= self.attack_samples {
                    self.stage = AdsrStage::Decay;
                    self.stage_pos = 0.0;
                    self.value = 1.0;
                }
            }
            AdsrStage::Decay => {
                let t = self.stage_pos / self.decay_samples;
                self.value = 1.0 - t * (1.0 - self.sustain_level);
                self.stage_pos += 1.0;

                if self.stage_pos >= self.decay_samples {
                    self.stage = AdsrStage::Sustain;
                    self.value = self.sustain_level;
                }
            }
            AdsrStage::Sustain => {
                self.value = self.sustain_level;
            }
            AdsrStage::Release => {
                let t = self.stage_pos / self.release_samples;
                self.value = self.release_start_value * (1.0 - t);
                self.stage_pos += 1.0;

                if self.stage_pos >= self.release_samples {
                    self.stage = AdsrStage::Idle;
                    self.value = 0.0;
                }
            }
        }

        self.value
    }

    /// Returns true if the envelope is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.stage != AdsrStage::Idle
    }

    /// Returns the current stage.
    #[must_use]
    pub fn stage(&self) -> AdsrStage {
        self.stage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_detector() {
        // Use shorter attack/release for faster test
        // 0.5ms attack at 48kHz = 24 samples time constant
        // 5ms release at 48kHz = 240 samples time constant
        let mut detector = EnvelopeDetector::new(0.5, 5.0, 48000.0, EnvelopeMode::Peak);

        // Feed a loud signal - need ~5 time constants (120 samples) to reach ~99%
        for _ in 0..200 {
            detector.process(1.0);
        }

        assert!(
            detector.current() > 0.9,
            "Expected > 0.9, got {}",
            detector.current()
        );

        // Feed silence, envelope should decay
        // With 5ms release (~240 samples), need ~5 time constants (1200 samples) to reach < 1%
        for _ in 0..2000 {
            detector.process(0.0);
        }

        assert!(
            detector.current() < 0.1,
            "Expected < 0.1, got {}",
            detector.current()
        );
    }

    #[test]
    fn test_adsr() {
        let mut env = AdsrEnvelope::new(10.0, 10.0, 0.5, 10.0, 1000.0);

        env.trigger();

        // Attack phase
        for _ in 0..10 {
            env.process();
        }
        assert!(env.value > 0.9);

        // Decay to sustain
        for _ in 0..10 {
            env.process();
        }
        assert!((env.value - 0.5).abs() < 0.1);

        // Release
        env.release();
        for _ in 0..10 {
            env.process();
        }
        assert!(env.value < 0.1);
    }
}
