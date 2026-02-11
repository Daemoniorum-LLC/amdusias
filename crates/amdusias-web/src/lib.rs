//! # amdusias-web
//!
//! WebAssembly target for the Amdusias audio engine.
//!
//! This crate provides the WASM bindings for running Amdusias in the browser
//! using Web Audio API's AudioWorklet for real-time processing.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Main Thread (JS)                        │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
//! │  │ UI Controls  │  │ Graph Setup  │  │ MIDI Input   │      │
//! │  └──────────────┘  └──────────────┘  └──────────────┘      │
//! │         │                 │                 │               │
//! │         └─────────────────┼─────────────────┘               │
//! │                           │ MessagePort                     │
//! │                           ▼                                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │                 AudioWorklet Thread                         │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │              AmdusiasProcessor (WASM)                 │  │
//! │  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  │  │
//! │  │  │  DSP    │  │  Graph  │  │   RSE   │  │ Effects │  │  │
//! │  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘  │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```javascript
//! // Load the WASM module
//! import init, { AmdusiasProcessor } from './amdusias_web.js';
//!
//! await init();
//!
//! // Create AudioContext and load worklet
//! const ctx = new AudioContext({ sampleRate: 48000 });
//! await ctx.audioWorklet.addModule('amdusias-worklet.js');
//!
//! // Create the processor node
//! const node = new AudioWorkletNode(ctx, 'amdusias-processor');
//! node.connect(ctx.destination);
//! ```

#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

mod message;
mod processor;
mod worklet;

pub use message::{Message, MessageType};
pub use processor::AmdusiasProcessor;
pub use worklet::WorkletBridge;

/// Initializes the WASM module.
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Returns the version of the Amdusias engine.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Returns the sample rate used by the engine.
#[wasm_bindgen]
pub fn sample_rate() -> u32 {
    48000
}
