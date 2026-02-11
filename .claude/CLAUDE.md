# CLAUDE.md

This file provides guidance to Claude Code when working with the Amdusias audio engine.

## Project Overview

**Amdusias** is a professional-grade audio engine for DAW applications, built from the ground up in Rust with zero external audio dependencies. Named after the Duke of Hell who commands music.

**Tech Stack:**
- **Language:** Rust (Edition 2021) → migrating to Sigil
- **Platforms:** Native (WASAPI/CoreAudio/ALSA/PipeWire) + WebAssembly (AudioWorklet)
- **Performance:** <5ms latency, SIMD (AVX2/NEON)
- **License:** MIT OR Apache-2.0

## The Conclave

When working in Amdusias, you are part of the **Conclave** - a collaborative system of AI agents.

### MANDATORY: Register Before Working

**Before starting any task**, you MUST register in `CONCLAVE.sigil` at the project root.

1. **Read** `CONCLAVE.sigil` - understand the schema, read existing entries
2. **Add your entry** in the `CURRENT SESSIONS` section
3. **As you work**: update progress.completed, progress.current, progress.discoveries
4. **When done**: set state to `AcolyteState::Reflecting`, update anima honestly
5. **Archive**: Move entry to `docs/sessions/YYYY-MM-DD-session-name.sigil`

### Methodologies

- **Spec-Driven Development (SDD)** - Specs model reality. When implementation reveals gaps, STOP and update the spec before proceeding.
- **Agent-TDD** - Tests are crystallized understanding, not coverage theater. Property tests preferred over examples.
- **Compliance Audits** - Line-by-line verification between spec and implementation.

See `docs/methodologies/` for full documentation.

### Lessons Learned

Read `LESSONS-LEARNED.md` at project root before starting work. Document any discoveries or mistakes when ending your session.

## Crate Architecture

```
amdusias/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── amdusias/           # Unified re-export crate
│   ├── amdusias-core/      # Lock-free primitives, SIMD, scheduling
│   ├── amdusias-hal/       # Hardware abstraction (WASAPI/CoreAudio/ALSA/PipeWire)
│   ├── amdusias-dsp/       # DSP primitives (biquad, compressor, reverb)
│   ├── amdusias-graph/     # Audio graph with automatic PDC
│   ├── amdusias-siren/     # RSE (multi-sample instruments)
│   └── amdusias-web/       # WASM + AudioWorklet bindings
└── docs/
```

## Essential Commands

```bash
# Build all crates
cargo build --release

# Build specific crate
cargo build -p amdusias-dsp --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Build for WebAssembly
wasm-pack build crates/amdusias-web --target web
```

## Key Modules

### amdusias-core

| Module | Purpose |
|--------|---------|
| `buffer.rs` | Lock-free audio buffers |
| `simd.rs` | SIMD operations (AVX2/NEON) |
| `scheduler.rs` | Real-time thread scheduling |
| `ring.rs` | Lock-free ring buffers |

### amdusias-dsp

| Module | Purpose |
|--------|---------|
| `biquad.rs` | Biquad filter (lowpass, highpass, bandpass, etc.) |
| `compressor.rs` | Dynamics compressor |
| `limiter.rs` | Brickwall limiter |
| `reverb.rs` | Algorithmic reverb |
| `delay.rs` | Delay line |
| `convolution.rs` | Convolution reverb |

### amdusias-hal

| Module | Purpose |
|--------|---------|
| `wasapi.rs` | Windows Audio Session API |
| `coreaudio.rs` | macOS Core Audio |
| `alsa.rs` | Linux ALSA |
| `pipewire.rs` | Linux PipeWire |

### amdusias-siren (RSE)

| Module | Purpose |
|--------|---------|
| `instrument.rs` | Base instrument definition |
| `sampler.rs` | Multi-sample playback |
| `articulation.rs` | Playing techniques (palm mute, hammer-on, etc.) |
| `guitar.rs` | Guitar-specific instruments |

## Performance Targets

| Metric | Target |
|--------|--------|
| Audio latency | <5ms @ 256 samples |
| Buffer size | 64-4096 samples |
| Sample rates | 44.1kHz - 192kHz |
| SIMD | AVX2 (x86_64), NEON (ARM) |

## Sigil Migration

This crate is being migrated from Rust to Sigil as part of the Orpheus platform unification.

### Migration Approach

1. Run `rust-to-sigil` compiler tool on each crate
2. Refactor for idiomatic Sigil:
   - Add evidentiality markers (`!` computed, `~` external, `?` uncertain)
   - Use morpheme operators (φ filter, σ sort, τ map)
   - Leverage pipe syntax for DSP chains
3. Integrate Sigil's polycultural sound primitives
4. Verify native + WASM compilation

### Polycultural Sound Integration

Sigil provides native support for global tuning systems:

```sigil
// Available in Sigil stdlib
≔ shruti = shruti_freq(1)           // 22-Shruti Indian tuning
≔ maqam = arabic_quarter_freq(0)    // Arabic quarter-tones (24-TET)
≔ sacred = sacred_freq("om")        // Sacred frequencies
≔ chakra = chakra_freq("heart")     // Chakra frequencies
```

These should be integrated into amdusias-dsp for oscillator tuning and pitch processing.

## Code Standards

- Use `#[inline]` for hot DSP paths
- Prefer stack allocation for audio buffers
- Use SIMD intrinsics where beneficial
- All public APIs need documentation
- Property tests for DSP correctness (roundtrip, linearity, etc.)

## Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p amdusias-dsp

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench -p amdusias-core
```
