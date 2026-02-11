//! Main WASM audio processor.

use amdusias_core::{AudioBuffer, SampleRate};
use amdusias_dsp::{BiquadFilter, Compressor, FilterType, Limiter, Processor, Reverb};
use wasm_bindgen::prelude::*;

/// The main audio processor for WebAssembly.
///
/// This struct runs in the AudioWorklet thread and processes audio
/// in real-time.
#[wasm_bindgen]
pub struct AmdusiasProcessor {
    /// Sample rate.
    sample_rate: f32,
    /// Buffer size (typically 128 for AudioWorklet).
    buffer_size: usize,
    /// High-pass filter for DC offset removal.
    dc_filter: BiquadFilter,
    /// Low-pass filter for anti-aliasing.
    lowpass: BiquadFilter,
    /// Compressor.
    compressor: Compressor,
    /// Reverb.
    reverb: Reverb,
    /// Output limiter.
    limiter: Limiter,
    /// Master gain.
    master_gain: f32,
    /// Reverb send level.
    reverb_send: f32,
}

#[wasm_bindgen]
impl AmdusiasProcessor {
    /// Creates a new processor.
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            buffer_size: 128, // AudioWorklet default
            dc_filter: BiquadFilter::new(FilterType::Highpass, 20.0, 0.707, sample_rate),
            lowpass: BiquadFilter::new(FilterType::Lowpass, 20000.0, 0.707, sample_rate),
            compressor: Compressor::new(sample_rate),
            reverb: Reverb::new(0.5, 0.5, 0.3, sample_rate),
            limiter: Limiter::new(-0.3, 5.0, 50.0, sample_rate),
            master_gain: 1.0,
            reverb_send: 0.3,
        }
    }

    /// Processes a block of audio.
    ///
    /// Input and output are interleaved stereo float arrays.
    #[wasm_bindgen]
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        let frames = input.len().min(output.len()) / 2;

        for frame in 0..frames {
            let in_l = input[frame * 2];
            let in_r = input[frame * 2 + 1];

            // DC removal
            let dc_l = self.dc_filter.process_sample(in_l);
            let dc_r = self.dc_filter.process_sample(in_r);

            // Mono for effects processing
            let mono = (dc_l + dc_r) * 0.5;

            // Compression
            let compressed = self.compressor.process_sample(mono);

            // Reverb (wet/dry handled internally)
            let reverb_out = self.reverb.process(compressed * self.reverb_send);

            // Mix dry + wet
            let mixed_l = dc_l + reverb_out;
            let mixed_r = dc_r + reverb_out;

            // Apply master gain
            let gained_l = mixed_l * self.master_gain;
            let gained_r = mixed_r * self.master_gain;

            // Limiting
            let limited_l = self.limiter.process_sample(gained_l);
            let limited_r = self.limiter.process_sample(gained_r);

            output[frame * 2] = limited_l;
            output[frame * 2 + 1] = limited_r;
        }

        true // Keep processor alive
    }

    /// Sets the master gain in dB.
    #[wasm_bindgen]
    pub fn set_master_gain_db(&mut self, gain_db: f32) {
        self.master_gain = 10.0_f32.powf(gain_db / 20.0);
    }

    /// Sets the reverb mix (0.0 to 1.0).
    #[wasm_bindgen]
    pub fn set_reverb_mix(&mut self, mix: f32) {
        self.reverb.set_mix(mix.clamp(0.0, 1.0));
    }

    /// Sets the reverb room size (0.0 to 1.0).
    #[wasm_bindgen]
    pub fn set_reverb_room_size(&mut self, size: f32) {
        self.reverb.set_room_size(size);
    }

    /// Sets the compressor threshold in dB.
    #[wasm_bindgen]
    pub fn set_compressor_threshold(&mut self, threshold_db: f32) {
        self.compressor.set_threshold(threshold_db);
    }

    /// Sets the compressor ratio.
    #[wasm_bindgen]
    pub fn set_compressor_ratio(&mut self, ratio: f32) {
        self.compressor.set_ratio(ratio);
    }

    /// Returns the current gain reduction in dB (for metering).
    #[wasm_bindgen]
    pub fn get_gain_reduction_db(&self) -> f32 {
        self.compressor.gain_reduction_db()
    }

    /// Resets all processors.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.dc_filter.reset();
        self.lowpass.reset();
        self.compressor.reset();
        self.reverb.reset();
        self.limiter.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let proc = AmdusiasProcessor::new(48000.0);
        assert_eq!(proc.sample_rate, 48000.0);
    }

    #[test]
    fn test_process_silence() {
        let mut proc = AmdusiasProcessor::new(48000.0);
        let input = [0.0_f32; 256];
        let mut output = [0.0_f32; 256];

        proc.process(&input, &mut output);

        // Output should be near-silent
        let max = output.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max < 0.001);
    }
}
