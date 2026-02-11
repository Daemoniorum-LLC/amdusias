# Amdusias

> *"He is a Duke Great and Strong, appearing at first like a Unicorn, but at the request of the Exorcist he standeth before him in Human Shape, causing Trumpets, and all manner of Musical Instruments to be heard, but not soon or immediately."*
> — Lesser Key of Solomon

**Amdusias** is a professional-grade audio engine for DAW applications, built from the ground up in Rust with zero external audio dependencies.

## Features

- 🎛️ **Zero Dependencies** - No JUCE, no cpal, pure Rust implementation
- ⚡ **Professional Latency** - <5ms with exclusive mode (WASAPI/CoreAudio/ALSA)
- 🌐 **Cross-Platform** - Native + WebAssembly (AudioWorklet)
- 🎚️ **Full DSP Suite** - Filters, dynamics, delay, reverb, convolution
- 🔀 **Audio Graph** - Node-based routing with automatic PDC
- 🎸 **RSE Competitor** - Multi-sample instruments with articulations

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        amdusias                                 │
├─────────────────────────────────────────────────────────────────┤
│  amdusias-graph     │  amdusias-dsp      │  amdusias-rse       │
│  (Audio routing)    │  (DSP primitives)  │  (Sample engine)    │
├─────────────────────────────────────────────────────────────────┤
│  amdusias-core (lock-free, SIMD, scheduling, buffers)          │
├────────────────────────┬────────────────────────────────────────┤
│  amdusias-hal (native) │  amdusias-web (WASM + Web Audio)      │
│  WASAPI/CoreAudio/ALSA │  AudioWorklet, SharedArrayBuffer      │
└────────────────────────┴────────────────────────────────────────┘
```

## Crates

| Crate | Description |
|-------|-------------|
| `amdusias` | Unified re-export crate |
| `amdusias-core` | Lock-free primitives, SIMD buffers, scheduling |
| `amdusias-hal` | Hardware abstraction (WASAPI, CoreAudio, ALSA, PipeWire) |
| `amdusias-dsp` | DSP primitives (biquad, compressor, limiter, reverb) |
| `amdusias-graph` | Audio graph with automatic latency compensation |
| `amdusias-rse` | Realistic Sound Engine (multi-sample instruments) |
| `amdusias-web` | WebAssembly bindings for browser |

## Quick Start

```rust
use amdusias::prelude::*;

// Create an audio graph
let mut graph = AudioGraph::new(48000.0, 512);

// Add nodes
let input = graph.add_node(InputNode::new(2));
let gain = graph.add_node(GainNode::new(0.8));
let output = graph.add_node(OutputNode::new(2));

// Connect nodes
graph.connect(input, 0, gain, 0)?;
graph.connect(gain, 0, output, 0)?;

// Compile for processing
graph.compile()?;
```

## DSP Examples

### Biquad Filter

```rust
use amdusias::dsp::{BiquadFilter, FilterType, Processor};

let mut filter = BiquadFilter::new(
    FilterType::Lowpass,
    1000.0,  // Frequency
    0.707,   // Q
    48000.0, // Sample rate
);

// Process samples
for sample in samples.iter_mut() {
    *sample = filter.process_sample(*sample);
}
```

### Compressor

```rust
use amdusias::dsp::{Compressor, Processor};

let mut comp = Compressor::new(48000.0);
comp.set_threshold(-20.0);
comp.set_ratio(4.0);
comp.set_attack(10.0, 48000.0);
comp.set_release(100.0, 48000.0);

for sample in samples.iter_mut() {
    *sample = comp.process_sample(*sample);
}
```

## RSE (Realistic Sound Engine)

```rust
use amdusias::rse::{GuitarInstrument, InstrumentPlayer, Articulation};

// Create a 6-string guitar
let guitar = GuitarInstrument::standard_6_string("strat", "Stratocaster");
let mut player = InstrumentPlayer::new(guitar.base, 48000.0);

// Trigger a note with palm mute articulation
player.note_on_with_articulation(64, 100, Articulation::PalmMute);

// Process audio
player.process(&mut output_buffer);
```

## WebAssembly

```javascript
import init, { AmdusiasProcessor } from './amdusias_web.js';

await init();

const ctx = new AudioContext({ sampleRate: 48000 });
await ctx.audioWorklet.addModule('amdusias-worklet.js');

const node = new AudioWorkletNode(ctx, 'amdusias-processor');
node.connect(ctx.destination);
```

## Building

```bash
# Build all crates
cargo build --release

# Build with specific features
cargo build --release --features "native rse"

# Build for WebAssembly
wasm-pack build crates/amdusias-web --target web

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Audio latency | <5ms @ 256 samples |
| Buffer size | 64-4096 samples |
| Sample rates | 44.1kHz - 192kHz |
| SIMD | AVX2 (x86_64), NEON (ARM) |

## License

MIT OR Apache-2.0

## Acknowledgements

Named after Amdusias, the Duke of Hell who commands music and causes instruments to play. Part of the Daemoniorum ecosystem.
