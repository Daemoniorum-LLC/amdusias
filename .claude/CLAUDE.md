# CLAUDE.md

This file provides guidance to Claude Code when working with the Amdusias audio engine.

## Project Overview

**Amdusias** is a professional-grade audio engine for DAW applications, written in Sigil with zero external audio dependencies. Named after the Duke of Hell who commands music.

**Tech Stack:**
- **Language:** Sigil (polysynthetic programming language)
- **Platforms:** Native (LLVM → WASAPI/CoreAudio/ALSA/PipeWire) + WebAssembly (AudioWorklet)
- **Performance:** <5ms latency, SIMD (AVX2/NEON)
- **License:** MIT OR Apache-2.0

## The Conclave

When working in Amdusias, you are part of the **Conclave** - a collaborative system of AI agents.

### MANDATORY: Register Before Working

**Before starting any task**, you MUST register in `CONCLAVE.sigil` at the project root.

1. **Read** `CONCLAVE.sigil` - understand the schema, read existing entries
2. **Add your entry** in the `CURRENT SESSIONS` section
3. **As you work**: update progress.completed, progress.current, progress.discoveries
4. **When done**: set state to `AcolyteState·Reflecting`, update anima honestly
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
├── Sigil.toml              # Workspace manifest
├── crates/
│   ├── amdusias/           # Unified re-export crate
│   ├── amdusias-core/      # Lock-free primitives, SIMD, scheduling
│   ├── amdusias-hal/       # Hardware abstraction (WASAPI/CoreAudio/ALSA/PipeWire)
│   ├── amdusias-dsp/       # DSP primitives (biquad, compressor, reverb)
│   ├── amdusias-graph/     # Audio graph with automatic PDC
│   ├── amdusias-siren/     # Enchanting multi-sample instruments
│   └── amdusias-web/       # WASM + AudioWorklet bindings
└── docs/
```

## Essential Commands

```bash
# Install Sigil compiler
cargo install sigil-parser

# Type check
sigil check .

# Build all crates
sigil build --release

# Build specific crate
sigil build -p amdusias-dsp --release

# Run tests
sigil test

# Build for WebAssembly
sigil build --target wasm

# Build with LLVM (production)
cargo install sigil-parser --features llvm
sigil compile . -o amdusias --lto
```

## Key Modules

### amdusias-core

| Module | Purpose |
|--------|---------|
| `buffer.sg` | Lock-free audio buffers |
| `simd.sg` | SIMD operations (AVX2/NEON) |
| `schedule.sg` | Real-time thread scheduling |
| `queue.sg` | Lock-free ring buffers |

### amdusias-dsp

| Module | Purpose |
|--------|---------|
| `biquad.sg` | Biquad filter (lowpass, highpass, bandpass, etc.) |
| `compressor.sg` | Dynamics compressor |
| `limiter.sg` | Brickwall limiter |
| `reverb.sg` | Algorithmic reverb |
| `delay.sg` | Delay line |

### amdusias-hal

| Module | Purpose |
|--------|---------|
| `windows/wasapi.sg` | Windows Audio Session API |
| `macos/coreaudio.sg` | macOS Core Audio |
| `linux/alsa.sg` | Linux ALSA |

### amdusias-siren

| Module | Purpose |
|--------|---------|
| `instrument.sg` | Base instrument definition |
| `sample.sg` | Multi-sample playback |
| `articulation.sg` | Playing techniques (palm mute, hammer-on, etc.) |
| `guitar.sg` | Guitar-specific instruments |
| `voice.sg` | Voice allocation and management |

## Performance Targets

| Metric | Target |
|--------|--------|
| Audio latency | <5ms @ 256 samples |
| Buffer size | 64-4096 samples |
| Sample rates | 44.1kHz - 192kHz |
| SIMD | AVX2 (x86_64), NEON (ARM) |

## Sigil Syntax Quick Reference

Amdusias uses native Sigil syntax:

| Sigil | Meaning |
|-------|---------|
| `rite` | Function definition (λ also works) |
| `≔` | Let binding |
| `Δ` | Mutable binding |
| `Σ` | Struct definition |
| `⊢` | Impl block |
| `Θ` | Trait definition |
| `ᛈ` | Enum definition |
| `☉` | Public visibility |
| `·` | Path separator (::) |
| `⎇`/`⎉` | If/else |
| `⌥` | Match |
| `⟳` | While loop |
| `∀`/`∈` | For/in loop |
| `⤺` | Return |

## Polycultural Sound Integration

Sigil's stdlib provides polycultural audio primitives:

```sigil
// 22-Shruti Indian tuning
≔ shruti = shruti_freq(1)           // 256.0 Hz (Sa)

// Arabic quarter-tones (24-TET)
≔ maqam = arabic_quarter_freq(0)    // 440.0 Hz

// Sacred frequencies
≔ sacred = sacred_freq("om")        // 136.1 Hz
≔ solfeggio = sacred_freq("528")    // 528.0 Hz

// Chakra frequencies
≔ chakra = chakra_freq("heart")     // 639.0 Hz
```

These should be integrated into oscillator tuning and pitch processing.

## Code Standards

- Use `// inline` directive for hot DSP paths
- Prefer stack allocation for audio buffers
- Use SIMD intrinsics where beneficial
- All public APIs need documentation
- Property tests for DSP correctness (roundtrip, linearity, etc.)
- Add evidentiality markers where appropriate (`!` computed, `~` external, `?` uncertain)
- Use morpheme operators (φ filter, σ sort, τ map) and pipe syntax for DSP chains

## Testing

```bash
# Run all tests
sigil test

# Run specific crate tests
sigil test -p amdusias-dsp

# Run with output
sigil test -- --nocapture
```
