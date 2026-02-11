# Amdusias

> *"He is a Duke Great and Strong, appearing at first like a Unicorn, but at the request of the Exorcist he standeth before him in Human Shape, causing Trumpets, and all manner of Musical Instruments to be heard, but not soon or immediately."*
> — Lesser Key of Solomon

**Amdusias** is a professional-grade audio engine for DAW applications, written in [Sigil](https://github.com/Daemoniorum-LLC/sigil-lang) with zero external audio dependencies.

## Features

- **Zero Dependencies** - No JUCE, no cpal, pure Sigil implementation
- **Professional Latency** - <5ms with exclusive mode (WASAPI/CoreAudio/ALSA)
- **Cross-Platform** - Native (LLVM) + WebAssembly (AudioWorklet)
- **Full DSP Suite** - Filters, dynamics, delay, reverb, convolution
- **Audio Graph** - Node-based routing with automatic PDC
- **Siren Engine** - Multi-sample instruments with articulations
- **Polycultural Sound** - Native support for global tuning systems (Shruti, Maqam, sacred frequencies)

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        amdusias                                 │
├─────────────────────────────────────────────────────────────────┤
│  amdusias-graph     │  amdusias-dsp      │  amdusias-siren     │
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
| `amdusias-siren` | Enchanting multi-sample instruments with articulations |
| `amdusias-web` | WebAssembly bindings for browser |

## Quick Start

```sigil
invoke amdusias·prelude·*;

// Create an audio graph
≔ Δgraph = AudioGraph·new(48000.0, 512);

// Add nodes
≔ input = graph.add_node(InputNode·new(2));
≔ gain = graph.add_node(GainNode·new(0.8));
≔ output = graph.add_node(OutputNode·new(2));

// Connect nodes
graph.connect(input, 0, gain, 0)?;
graph.connect(gain, 0, output, 0)?;

// Compile ∀ processing
graph.compile()?;
```

## DSP Examples

### Biquad Filter

```sigil
invoke amdusias·dsp·{BiquadFilter, FilterType, Processor};

≔ Δfilter = BiquadFilter·new(
    FilterType·Lowpass,
    1000.0,  // Frequency
    0.707,   // Q
    48000.0, // Sample rate
);

// Process samples
∀ sample ∈ samples.iter_mut() {
    *sample = filter.process_sample(*sample);
}
```

### Compressor

```sigil
invoke amdusias·dsp·{Compressor, Processor};

≔ Δcomp = Compressor·new(48000.0);
comp.set_threshold(-20.0);
comp.set_ratio(4.0);
comp.set_attack(10.0, 48000.0);
comp.set_release(100.0, 48000.0);

∀ sample ∈ samples.iter_mut() {
    *sample = comp.process_sample(*sample);
}
```

## Polycultural Sound

Sigil provides native support for global tuning systems:

```sigil
// 22-Shruti Indian tuning
≔ sa = shruti_freq(1)              // 256.0 Hz (Sa - tonic)

// Arabic quarter-tones (24-TET)
≔ rast = arabic_quarter_freq(0)    // 440.0 Hz

// Sacred frequencies
≔ om = sacred_freq("om")           // 136.1 Hz
≔ solfeggio = sacred_freq("528")   // 528.0 Hz (DNA repair)

// Chakra frequencies
≔ heart = chakra_freq("heart")     // 639.0 Hz
```

## Siren (Multi-Sample Engine)

```sigil
invoke amdusias·siren·{GuitarInstrument, InstrumentPlayer, Articulation};

// Create a 6-string guitar
≔ guitar = GuitarInstrument·standard_6_string("strat", "Stratocaster");
≔ Δplayer = InstrumentPlayer·new(guitar.base, 48000.0);

// Trigger a note with palm mute articulation
player.note_on_with_articulation(64, 100, Articulation·PalmMute);

// Process audio
player.process(&Δoutput_buffer);
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
# Install Sigil compiler
cargo install sigil-parser

# Type check
sigil check .

# Build all crates
sigil build --release

# Build for WebAssembly
sigil build --target wasm

# Run tests
sigil test

# Build with LLVM (production performance)
cargo install sigil-parser --features llvm
sigil compile . -o amdusias --lto
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
