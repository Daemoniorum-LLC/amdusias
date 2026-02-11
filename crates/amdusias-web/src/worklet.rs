//! AudioWorklet bridge utilities.

use wasm_bindgen::prelude::*;
use crate::message::{Message, MessageType};
use crate::processor::AmdusiasProcessor;

/// Bridge between JavaScript AudioWorklet and WASM processor.
#[wasm_bindgen]
pub struct WorkletBridge {
    processor: AmdusiasProcessor,
}

#[wasm_bindgen]
impl WorkletBridge {
    /// Creates a new worklet bridge.
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            processor: AmdusiasProcessor::new(sample_rate),
        }
    }

    /// Processes audio data.
    ///
    /// Called from the AudioWorkletProcessor's process() method.
    #[wasm_bindgen]
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        self.processor.process(input, output)
    }

    /// Handles a message from the main thread.
    ///
    /// The message should be a JSON-encoded Message struct.
    #[wasm_bindgen]
    pub fn handle_message(&mut self, json: &str) -> Result<(), JsValue> {
        let message: Message = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid message: {}", e)))?;

        match message.msg_type {
            MessageType::Param => {
                if let (Some(param_id), Some(value)) = (message.param_id, message.value) {
                    self.set_param(param_id, value);
                }
            }
            MessageType::NoteOn => {
                // Forward to RSE player when integrated
            }
            MessageType::NoteOff => {
                // Forward to RSE player when integrated
            }
            MessageType::AllNotesOff => {
                // Forward to RSE player when integrated
            }
            MessageType::Transport => {
                // Handle transport commands
            }
            _ => {}
        }

        Ok(())
    }

    /// Sets a parameter value.
    fn set_param(&mut self, param_id: u32, value: f32) {
        use crate::message::params::*;

        match param_id {
            MASTER_GAIN => self.processor.set_master_gain_db(value),
            REVERB_MIX => self.processor.set_reverb_mix(value),
            REVERB_SIZE => self.processor.set_reverb_room_size(value),
            COMP_THRESHOLD => self.processor.set_compressor_threshold(value),
            COMP_RATIO => self.processor.set_compressor_ratio(value),
            _ => {}
        }
    }

    /// Resets the processor.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.processor.reset();
    }

    /// Returns the current gain reduction for metering.
    #[wasm_bindgen]
    pub fn get_gain_reduction_db(&self) -> f32 {
        self.processor.get_gain_reduction_db()
    }
}

/// JavaScript code for the AudioWorklet processor.
///
/// This should be saved as a separate .js file and loaded via
/// `audioContext.audioWorklet.addModule()`.
pub const WORKLET_JS: &str = r#"
class AmdusiasProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.bridge = null;
    this.port.onmessage = this.handleMessage.bind(this);
  }

  async init(wasmModule) {
    const { WorkletBridge } = await wasmModule;
    this.bridge = new WorkletBridge(sampleRate);
  }

  handleMessage(event) {
    if (event.data.type === 'init') {
      this.init(event.data.module);
    } else if (this.bridge) {
      this.bridge.handle_message(JSON.stringify(event.data));
    }
  }

  process(inputs, outputs, parameters) {
    if (!this.bridge) return true;

    const input = inputs[0];
    const output = outputs[0];

    if (input.length === 0 || output.length === 0) return true;

    // Interleave input channels
    const frames = input[0].length;
    const interleaved = new Float32Array(frames * 2);
    for (let i = 0; i < frames; i++) {
      interleaved[i * 2] = input[0]?.[i] ?? 0;
      interleaved[i * 2 + 1] = input[1]?.[i] ?? input[0]?.[i] ?? 0;
    }

    // Process
    const result = new Float32Array(frames * 2);
    this.bridge.process(interleaved, result);

    // De-interleave output
    for (let i = 0; i < frames; i++) {
      output[0][i] = result[i * 2];
      if (output[1]) output[1][i] = result[i * 2 + 1];
    }

    return true;
  }
}

registerProcessor('amdusias-processor', AmdusiasProcessor);
"#;
