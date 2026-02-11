# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **BREAKING**: Migrated entire codebase from Rust to Sigil
- Updated CI workflow for Sigil compiler (sigil-parser from crates.io)
- Updated documentation for Sigil syntax and commands

## [0.1.0] - 2025-02-11

### Added

- Initial open source release
- **amdusias-core**: Lock-free primitives, SIMD buffers (AVX2/NEON), real-time scheduling
- **amdusias-hal**: Hardware abstraction for WASAPI, CoreAudio, ALSA, PipeWire
- **amdusias-dsp**: DSP primitives (biquad, compressor, limiter, reverb, delay, convolution)
- **amdusias-graph**: Audio graph with automatic latency compensation (PDC)
- **amdusias-siren**: Multi-sample instrument engine with articulations
- **amdusias-web**: WebAssembly bindings with AudioWorklet integration
- Professional latency targets (<5ms @ 256 samples)
- Cross-platform support: Native + WebAssembly
