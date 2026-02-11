//! # Amdusias
//!
//! **Amdusias** is a professional-grade audio engine for DAW applications,
//! named after the demon duke of music from the Ars Goetia.
//!
//! > *"He is a Duke Great and Strong, causing Trumpets, and all manner of
//! > Musical Instruments to be heard, but not soon or immediately."*
//! > — Lesser Key of Solomon
//!
//! ## Features
//!
//! - **Zero-dependency audio** - No JUCE, no cpal, pure Rust
//! - **Professional latency** - <5ms with exclusive mode
//! - **Cross-platform** - Native (WASAPI/CoreAudio/ALSA) + WebAssembly
//! - **Full DSP suite** - Filters, dynamics, delay, reverb, convolution
//! - **Audio graph** - Node-based routing with automatic PDC
//! - **Siren** - Enchanting multi-sample instruments with articulations
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        amdusias                                 │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  amdusias-graph     │  amdusias-dsp      │  amdusias-siren     │
//! │  (Audio routing)    │  (DSP primitives)  │  (Sample engine)    │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  amdusias-core (lock-free, SIMD, scheduling, buffers)          │
//! ├────────────────────────┬────────────────────────────────────────┤
//! │  amdusias-hal (native) │  amdusias-web (WASM + Web Audio)      │
//! │  WASAPI/CoreAudio/ALSA │  AudioWorklet, SharedArrayBuffer      │
//! └────────────────────────┴────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use amdusias::prelude::*;
//!
//! // Create an audio graph
//! let mut graph = AudioGraph::new(48000.0, 512);
//!
//! // Add nodes
//! let input = graph.add_node(InputNode::new(2));
//! let reverb = graph.add_node(ReverbNode::new(0.5, 0.5));
//! let output = graph.add_node(OutputNode::new(2));
//!
//! // Connect nodes
//! graph.connect(input, 0, reverb, 0)?;
//! graph.connect(reverb, 0, output, 0)?;
//!
//! // Compile and run
//! graph.compile()?;
//! let processor = graph.create_processor()?;
//! ```
//!
//! ## Crates
//!
//! | Crate | Description |
//! |-------|-------------|
//! | `amdusias-core` | Lock-free primitives, SIMD buffers, scheduling |
//! | `amdusias-hal` | Hardware abstraction (WASAPI, CoreAudio, ALSA) |
//! | `amdusias-dsp` | DSP primitives (filters, dynamics, reverb) |
//! | `amdusias-graph` | Audio graph with automatic latency compensation |
//! | `amdusias-siren` | Siren: enchanting sample instruments |
//! | `amdusias-web` | WebAssembly bindings for browser |

#![warn(missing_docs)]
#![warn(clippy::all)]

// Re-export core crates
pub use amdusias_core as core;
pub use amdusias_dsp as dsp;
pub use amdusias_graph as graph;

#[cfg(feature = "native")]
pub use amdusias_hal as hal;

#[cfg(feature = "siren")]
pub use amdusias_siren as siren;

#[cfg(feature = "web")]
pub use amdusias_web as web;

/// Prelude module with commonly used types.
pub mod prelude {
    // Core types
    pub use amdusias_core::{AudioBuffer, ChannelLayout, SampleRate, SpscQueue};

    // DSP processors
    pub use amdusias_dsp::{
        BiquadFilter, Compressor, DelayLine, FilterType, Limiter, Processor, Reverb,
    };

    // Graph types
    pub use amdusias_graph::{AudioGraph, AudioNode, Connection, NodeId, NodeInfo};

    // Graph nodes
    pub use amdusias_graph::nodes::{GainNode, InputNode, MixerNode, OutputNode};

    // HAL types (native only)
    #[cfg(feature = "native")]
    pub use amdusias_hal::{AudioBackend, AudioCallback, StreamConfig};

    // Siren types (optional)
    #[cfg(feature = "siren")]
    pub use amdusias_siren::{Articulation, Instrument, InstrumentPlayer};
}

/// Returns the version of the Amdusias engine.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
