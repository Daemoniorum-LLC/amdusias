# Amdusias TDD Roadmap to Production

**Goal:** Professional audio engine with <5ms latency, zero external dependencies, targeting both native and web platforms.

**Philosophy:** Test-Driven Development (TDD) - every feature starts with a failing test.

---

## Current Status

| Crate | Unit Tests | Integration | Coverage | Production Ready |
|-------|-----------|-------------|----------|------------------|
| amdusias-core | ✅ 54 passing | ❌ | ~80% | ⚠️ |
| amdusias-dsp | ✅ 81 passing | ❌ | ~75% | ⚠️ |
| amdusias-graph | ✅ 108 passing | ❌ | ~85% | ⚠️ |
| amdusias-hal | ✅ 130 passing (Linux) | ❌ | ~90% | ⚠️ |
| amdusias-siren | ✅ 187 passing | ❌ | ~90% | ⚠️ |
| amdusias-web | ✅ 2 passing | ❌ | ~10% | ❌ |

**Total:** 563 tests passing (Linux), 4 doc tests ignored

*Platform-specific HAL tests: ALSA (31 tests), WASAPI (~25 tests), CoreAudio (~27 tests)*

### Phase Completion
- ✅ **Phase 1: Core Primitives** - 54 tests covering buffers, SIMD, queues, schedulers
- ✅ **Phase 2: DSP Primitives** - 81 tests covering filters, dynamics, delay, reverb
- ✅ **Phase 3: HAL Abstractions** - 99 tests covering config, device, stream, error types
- ✅ **Phase 3b: Platform Backends** - ~83 tests (ALSA/WASAPI/CoreAudio) covering backends
- ✅ **Phase 4: Audio Graph** - 108 tests covering topology, nodes, connections, processor
- ✅ **Phase 5: Siren (Sample Instrument Engine)** - 187 tests covering samples, zones, articulations, instruments, voices, guitar, drums

---

## Phase 0: Foundation (Current)
**Status: ✅ COMPLETE**

- [x] Scaffold all 7 crates
- [x] Fix initial compilation errors
- [x] Fix failing tests (delay overflow, envelope timing)
- [x] Establish baseline test coverage

---

## Phase 1: Core Primitives
**Target: 100% test coverage for amdusias-core**

### 1.1 Lock-Free Queue (RED → GREEN → REFACTOR)

```rust
// Write test FIRST
#[test]
fn test_spsc_queue_concurrent() {
    // Producer thread pushes 10000 items
    // Consumer thread pops 10000 items
    // Verify no data loss, no corruption
}

#[test]
fn test_spsc_queue_memory_ordering() {
    // Verify Acquire/Release semantics
    // Test under MIRI for undefined behavior
}

#[test]
fn test_spsc_queue_capacity_power_of_two() {
    // Verify mask optimization works correctly
}
```

**Implementation Tasks:**
- [ ] Test: Concurrent push/pop with std::thread
- [ ] Test: Memory ordering with loom or MIRI
- [ ] Test: Wrap-around edge cases
- [ ] Impl: AtomicUsize indices with proper ordering
- [ ] Bench: Compare with crossbeam-channel

### 1.2 SIMD Operations (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_simd_gain_matches_scalar() {
    let input = random_buffer(1024);
    let scalar_result = apply_gain_scalar(&input, 0.5);
    let simd_result = apply_gain_simd(&input, 0.5);
    assert_buffers_equal(&scalar_result, &simd_result, 1e-6);
}

#[test]
fn test_simd_rms_accuracy() {
    // Known input with calculable RMS
    let input = vec![1.0, -1.0, 1.0, -1.0];
    assert_eq!(calculate_rms(&input), 1.0);
}

#[test]
#[cfg(target_arch = "x86_64")]
fn test_avx2_available_uses_avx2() {
    // Verify AVX2 path is taken when available
}
```

**Implementation Tasks:**
- [ ] Test: Gain application correctness
- [ ] Test: RMS calculation accuracy
- [ ] Test: Peak detection
- [ ] Test: Buffer mixing
- [ ] Impl: Runtime CPU feature detection
- [ ] Impl: AVX2 intrinsics (x86_64)
- [ ] Impl: NEON intrinsics (aarch64)
- [ ] Bench: SIMD vs scalar performance

### 1.3 Audio Buffer (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_buffer_alignment() {
    let buffer = AudioBuffer::<2>::new(1024);
    assert!(buffer.as_ptr() as usize % 32 == 0); // 32-byte aligned for AVX
}

#[test]
fn test_buffer_no_allocations_in_process() {
    // Use custom allocator to detect allocations
    let buffer = AudioBuffer::<2>::new(256);
    let _guard = AllocationGuard::deny();
    buffer.apply_gain(0.5); // Should not allocate
}

#[test]
fn test_buffer_channel_interleaving() {
    let mut buffer = AudioBuffer::<2>::new(4);
    buffer.set(0, 0, 1.0); // L channel, sample 0
    buffer.set(1, 0, 2.0); // R channel, sample 0
    assert_eq!(buffer.interleaved(), &[1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
}
```

**Implementation Tasks:**
- [ ] Test: 32-byte alignment for AVX
- [ ] Test: Zero allocations during processing
- [ ] Test: Interleaved/planar conversion
- [ ] Impl: Custom aligned allocator
- [ ] Impl: const generics for channels

### 1.4 Event Scheduler (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_scheduler_sample_accurate() {
    let mut scheduler = EventScheduler::new();
    scheduler.schedule(Event::NoteOn(60, 100), 100); // Sample 100
    scheduler.schedule(Event::NoteOff(60), 200);     // Sample 200

    let events = scheduler.drain_until(150);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sample, 100);
}

#[test]
fn test_scheduler_ordering() {
    // Events at same sample maintain insertion order
}

#[test]
fn test_scheduler_no_heap_allocation_in_drain() {
    // drain_until should not allocate
}
```

**Implementation Tasks:**
- [ ] Test: Sample-accurate timing
- [ ] Test: Event ordering guarantees
- [ ] Test: No allocations in hot path
- [ ] Impl: Priority queue without heap allocation
- [ ] Impl: Pre-allocated event pool

---

## Phase 2: DSP Primitives
**Target: 100% test coverage for amdusias-dsp**

### 2.1 Biquad Filter (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_biquad_lowpass_frequency_response() {
    let filter = BiquadFilter::lowpass(1000.0, 0.707, 48000.0);

    // Test at various frequencies
    assert!(filter.magnitude_at(100.0) > 0.99);   // Passband
    assert!(filter.magnitude_at(1000.0) > 0.70);  // Cutoff (-3dB)
    assert!(filter.magnitude_at(10000.0) < 0.1);  // Stopband
}

#[test]
fn test_biquad_stability() {
    // Filter should remain stable with extreme parameters
    let filter = BiquadFilter::lowpass(20.0, 10.0, 48000.0);
    let mut output = 0.0;
    for _ in 0..10000 {
        output = filter.process(1.0);
    }
    assert!(output.is_finite());
}

#[test]
fn test_biquad_transposed_direct_form_2() {
    // Verify implementation matches reference
}
```

**Implementation Tasks:**
- [ ] Test: Frequency response accuracy
- [ ] Test: Stability with extreme Q values
- [ ] Test: Coefficient calculation vs reference
- [ ] Impl: All filter types (LP, HP, BP, Notch, Peak, Shelf)
- [ ] Impl: Frequency response calculation
- [ ] Bench: Filter chain performance

### 2.2 Dynamics (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_compressor_ratio() {
    let mut comp = Compressor::new(-20.0, 4.0, 10.0, 100.0, 48000.0);

    // Signal at -10dB (10dB above threshold)
    // Should be reduced to -17.5dB (10/4 = 2.5dB above threshold)
    let output = process_constant_signal(&mut comp, db_to_linear(-10.0), 1000);
    assert_approx_eq!(linear_to_db(output), -17.5, 0.5);
}

#[test]
fn test_limiter_never_exceeds_ceiling() {
    let mut limiter = Limiter::new(-0.3, 5.0, 50.0, 48000.0);

    // Process random loud signal
    for _ in 0..100000 {
        let input = rand::random::<f32>() * 10.0 - 5.0; // -5 to +5
        let output = limiter.process(input);
        assert!(output.abs() <= 0.966 + 0.001); // -0.3dB ceiling
    }
}

#[test]
fn test_limiter_lookahead_prevents_overshoot() {
    // Sudden transient should not cause overshoot
}
```

**Implementation Tasks:**
- [ ] Test: Compression ratio accuracy
- [ ] Test: Limiter ceiling guarantee
- [ ] Test: Lookahead effectiveness
- [ ] Test: Attack/release timing
- [ ] Impl: Soft-knee compression
- [ ] Impl: True-peak limiting with oversampling
- [ ] Bench: Real-time performance

### 2.3 Reverb (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_reverb_decay_time() {
    let reverb = Reverb::new(0.5, 0.5, 1.0, 48000.0);

    // Measure RT60 (time to decay 60dB)
    let rt60 = measure_rt60(&reverb);
    assert!(rt60 > 0.5 && rt60 < 3.0); // Reasonable room
}

#[test]
fn test_reverb_no_metallic_artifacts() {
    // Comb filter delays should be mutually prime
}

#[test]
fn test_reverb_stereo_width() {
    // Verify L/R decorrelation
}
```

**Implementation Tasks:**
- [ ] Test: Decay time measurement
- [ ] Test: Frequency response (no ringing)
- [ ] Test: Stereo decorrelation
- [ ] Impl: Improved diffusion network
- [ ] Impl: Modulated delays for chorus
- [ ] Impl: Early reflections

---

## Phase 3: Hardware Abstraction Layer
**Target: Working audio I/O on Linux, macOS, Windows**

### 3.1 ALSA Backend (Linux)

```rust
#[test]
#[cfg(target_os = "linux")]
fn test_alsa_device_enumeration() {
    let devices = AlsaBackend::enumerate_devices();
    assert!(!devices.is_empty(), "No ALSA devices found");
}

#[test]
#[cfg(target_os = "linux")]
fn test_alsa_stream_open_close() {
    let backend = AlsaBackend::new();
    let stream = backend.open_stream(StreamConfig::default()).unwrap();
    stream.close().unwrap();
}

#[test]
#[cfg(target_os = "linux")]
fn test_alsa_callback_timing() {
    // Verify callback is called with correct buffer size
    // Measure actual latency
}
```

**Implementation Tasks:**
- [ ] Test: Device enumeration
- [ ] Test: Stream open/close
- [ ] Test: Callback timing accuracy
- [ ] Test: Error recovery
- [ ] Impl: ALSA bindings (raw libc)
- [ ] Impl: PipeWire support
- [ ] Impl: JACK support (optional)

### 3.2 WASAPI Backend (Windows)

```rust
#[test]
#[cfg(target_os = "windows")]
fn test_wasapi_exclusive_mode() {
    // Exclusive mode should provide lowest latency
}

#[test]
#[cfg(target_os = "windows")]
fn test_wasapi_shared_mode_fallback() {
    // Fallback when exclusive mode unavailable
}
```

**Implementation Tasks:**
- [ ] Test: Exclusive vs shared mode
- [ ] Test: Device change handling
- [ ] Impl: COM bindings
- [ ] Impl: Exclusive mode low-latency

### 3.3 CoreAudio Backend (macOS)

```rust
#[test]
#[cfg(target_os = "macos")]
fn test_coreaudio_aggregate_device() {
    // Test combining input/output devices
}

#[test]
#[cfg(target_os = "macos")]
fn test_coreaudio_sample_rate_conversion() {
    // System may force sample rate conversion
}
```

**Implementation Tasks:**
- [ ] Test: AudioUnit setup
- [ ] Test: Aggregate device support
- [ ] Impl: AudioUnit callbacks
- [ ] Impl: kAudioUnitProperty handling

---

## Phase 4: Audio Graph
**Target: Modular routing with automatic latency compensation**

### 4.1 Graph Topology (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_graph_topological_sort() {
    let mut graph = AudioGraph::new();
    let a = graph.add_node(GainNode::new(1.0));
    let b = graph.add_node(GainNode::new(1.0));
    let c = graph.add_node(GainNode::new(1.0));

    graph.connect(a, 0, b, 0);
    graph.connect(b, 0, c, 0);

    let order = graph.processing_order();
    assert_eq!(order, vec![a, b, c]);
}

#[test]
fn test_graph_cycle_detection() {
    let mut graph = AudioGraph::new();
    let a = graph.add_node(GainNode::new(1.0));
    let b = graph.add_node(GainNode::new(1.0));

    graph.connect(a, 0, b, 0);
    assert!(graph.connect(b, 0, a, 0).is_err()); // Would create cycle
}

#[test]
fn test_graph_latency_compensation() {
    // Parallel paths with different latencies should be aligned
}
```

**Implementation Tasks:**
- [ ] Test: Topological sorting
- [ ] Test: Cycle detection
- [ ] Test: PDC calculation
- [ ] Impl: Kahn's algorithm
- [ ] Impl: Automatic latency alignment
- [ ] Impl: Lock-free graph updates

### 4.2 Graph Processing (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_graph_process_no_allocation() {
    let graph = build_test_graph();
    let _guard = AllocationGuard::deny();
    graph.process(&mut buffer); // Must not allocate
}

#[test]
fn test_graph_parallel_processing() {
    // Independent branches should process in parallel
}
```

**Implementation Tasks:**
- [ ] Test: Zero allocations in process
- [ ] Test: Parallel branch processing
- [ ] Impl: Work-stealing scheduler
- [ ] Impl: Buffer pool

---

## Phase 5: Realistic Sound Engine (Siren)
**Target: Guitar Pro-quality playback**

### 5.1 Sample Playback (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_sample_pitch_accuracy() {
    let sample = Sample::from_file("test/guitar_a4.wav");
    let player = SamplePlayer::new(sample);

    // Play A4 (440Hz)
    player.trigger(69, 100); // MIDI note 69 = A4

    let output = collect_output(&player, 4096);
    let detected_pitch = detect_pitch(&output);
    assert_approx_eq!(detected_pitch, 440.0, 1.0); // Within 1Hz
}

#[test]
fn test_velocity_layers() {
    let instrument = Instrument::load("test/piano.sfz");

    // Different velocities should select different samples
    let soft = instrument.find_zone(60, 30);
    let loud = instrument.find_zone(60, 127);
    assert_ne!(soft.sample_id, loud.sample_id);
}

#[test]
fn test_round_robin() {
    // Same note repeated should cycle through variations
}
```

**Implementation Tasks:**
- [ ] Test: Pitch accuracy across range
- [ ] Test: Velocity layer selection
- [ ] Test: Round-robin variation
- [ ] Impl: High-quality resampling (sinc)
- [ ] Impl: SFZ/SF2 parser
- [ ] Impl: Sample streaming from disk

### 5.2 Articulations (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_palm_mute_frequency_content() {
    let guitar = GuitarInstrument::new();

    let normal = guitar.play(60, 100, Articulation::Sustain);
    let muted = guitar.play(60, 100, Articulation::PalmMute);

    let normal_brightness = calculate_brightness(&normal);
    let muted_brightness = calculate_brightness(&muted);

    assert!(muted_brightness < normal_brightness * 0.5);
}

#[test]
fn test_bend_pitch_accuracy() {
    let guitar = GuitarInstrument::new();

    // Bend up 200 cents (whole tone)
    let bent = guitar.play(60, 100, Articulation::Bend { cents: 200 });

    let pitch = detect_pitch(&bent);
    assert_approx_eq!(pitch, 261.63 * 1.122, 1.0); // C4 + whole tone
}

#[test]
fn test_slide_smooth_pitch() {
    // Slide should produce smooth pitch transition
}
```

**Implementation Tasks:**
- [ ] Test: Each articulation type
- [ ] Test: Articulation transitions
- [ ] Impl: Per-articulation sample selection
- [ ] Impl: Real-time pitch bend
- [ ] Impl: Vibrato LFO

### 5.3 Guitar-Specific (RED → GREEN → REFACTOR)

```rust
#[test]
fn test_per_string_physics() {
    let guitar = GuitarInstrument::standard_tuning();

    // Same pitch on different strings should sound different
    let e2_6th = guitar.play_string(6, 0, 100); // Open E2
    let e2_5th = guitar.play_string(5, 7, 100); // E2 on 5th string

    // Spectral centroid should differ
    let centroid_6th = spectral_centroid(&e2_6th);
    let centroid_5th = spectral_centroid(&e2_5th);
    assert_ne!(centroid_6th, centroid_5th);
}

#[test]
fn test_amp_simulation() {
    let amp = AmpModel::new(AmpType::Crunch);

    // Soft input = clean, loud input = distorted
    let clean_thd = measure_thd(amp.process(0.1));
    let dist_thd = measure_thd(amp.process(0.9));

    assert!(dist_thd > clean_thd * 5.0);
}

#[test]
fn test_cabinet_impulse_response() {
    // Cabinet IR should be applied correctly
}
```

**Implementation Tasks:**
- [ ] Test: Per-string modeling
- [ ] Test: Amp distortion character
- [ ] Test: Cabinet IR convolution
- [ ] Impl: String sympathetic resonance
- [ ] Impl: Pickup position modeling
- [ ] Impl: Amp tone stack EQ

---

## Phase 6: Web/WASM Target
**Target: <10ms latency in browser**

### 6.1 AudioWorklet Integration (RED → GREEN → REFACTOR)

```rust
#[wasm_bindgen_test]
async fn test_worklet_initialization() {
    let ctx = web_sys::AudioContext::new().unwrap();
    let processor = AmdusiasProcessor::new(&ctx).await.unwrap();
    assert!(processor.is_initialized());
}

#[wasm_bindgen_test]
async fn test_worklet_latency() {
    let ctx = web_sys::AudioContext::new().unwrap();
    let latency = ctx.base_latency() + ctx.output_latency();
    assert!(latency < 0.010); // <10ms
}

#[wasm_bindgen_test]
async fn test_message_port_communication() {
    // Parameters sent via MessagePort should apply
}
```

**Implementation Tasks:**
- [ ] Test: Worklet registration
- [ ] Test: Latency measurement
- [ ] Test: MessagePort reliability
- [ ] Impl: SharedArrayBuffer double-buffer
- [ ] Impl: Parameter smoothing in worklet
- [ ] Impl: Web Workers for non-realtime tasks

### 6.2 WASM Performance (RED → GREEN → REFACTOR)

```rust
#[wasm_bindgen_test]
fn test_wasm_simd_enabled() {
    assert!(is_wasm_simd_available());
}

#[wasm_bindgen_test]
fn test_process_128_samples_under_3ms() {
    let processor = AmdusiasProcessor::new();
    let start = performance_now();
    processor.process_128();
    let elapsed = performance_now() - start;
    assert!(elapsed < 3.0); // Must complete in <3ms (128 samples @ 44.1kHz)
}
```

**Implementation Tasks:**
- [ ] Test: WASM SIMD availability
- [ ] Test: Processing time budget
- [ ] Impl: wasm32-simd128 optimizations
- [ ] Impl: Memory management without GC pressure
- [ ] Bench: Chrome vs Firefox vs Safari

---

## Phase 7: Integration Testing
**Target: Full system validation**

### 7.1 End-to-End Tests

```rust
#[test]
fn test_midi_to_audio_pipeline() {
    let engine = AmdusiasEngine::new();
    let midi = load_midi_file("test/bach_prelude.mid");

    engine.load_instrument("test/piano.sfz");
    let audio = engine.render(midi);

    // Audio should not be silent
    assert!(audio.rms() > 0.01);
    // Audio should not clip
    assert!(audio.peak() < 1.0);
}

#[test]
fn test_real_time_streaming() {
    let engine = AmdusiasEngine::new();

    // Simulate real-time with callback timing
    for _ in 0..1000 {
        let start = Instant::now();
        engine.process_block(256);
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_micros(5000)); // <5ms for 256 samples @ 48kHz
    }
}
```

### 7.2 Stress Tests

```rust
#[test]
fn test_max_polyphony() {
    let engine = AmdusiasEngine::new();

    // Trigger 64 simultaneous notes
    for note in 36..100 {
        engine.note_on(note, 100);
    }

    // Should not crash or exceed time budget
    engine.process_block(256);
}

#[test]
fn test_parameter_automation() {
    // Rapid parameter changes should not cause artifacts
}

#[test]
fn test_memory_stability() {
    // Run for extended period, check for leaks
}
```

---

## Phase 8: Performance Benchmarks
**Target: Quantifiable performance metrics**

### Benchmark Suite

```rust
#[bench]
fn bench_biquad_filter_1024_samples(b: &mut Bencher) {
    let mut filter = BiquadFilter::lowpass(1000.0, 0.707, 48000.0);
    let mut buffer = vec![0.5; 1024];

    b.iter(|| {
        filter.process_block(&mut buffer);
    });
}

#[bench]
fn bench_reverb_process(b: &mut Bencher) {
    let mut reverb = Reverb::new(0.5, 0.5, 0.5, 48000.0);

    b.iter(|| {
        for _ in 0..256 {
            black_box(reverb.process(0.5));
        }
    });
}

#[bench]
fn bench_full_graph_process(b: &mut Bencher) {
    let graph = build_typical_graph(); // 10 nodes, 5 effects
    let mut buffer = AudioBuffer::<2>::new(256);

    b.iter(|| {
        graph.process(&mut buffer);
    });
}
```

### Performance Targets

| Operation | Target | Measured |
|-----------|--------|----------|
| Biquad (1024 samples) | <5μs | TBD |
| Reverb (256 samples) | <50μs | TBD |
| Full graph (10 nodes) | <500μs | TBD |
| Voice allocation | <1μs | TBD |
| Sample interpolation | <10μs/voice | TBD |

---

## Phase 9: Production Hardening

### 9.1 Error Handling

- [ ] All `unwrap()` replaced with proper error handling
- [ ] Custom error types with context
- [ ] Graceful degradation (e.g., reduce polyphony under load)

### 9.2 Documentation

- [ ] Rustdoc for all public APIs
- [ ] Architecture decision records (ADRs)
- [ ] Integration guide for Orpheus

### 9.3 CI/CD

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - run: cargo test --all-features
      - run: cargo bench --no-run
      - run: cargo clippy -- -D warnings
      - run: cargo miri test  # Undefined behavior check

  wasm:
    runs-on: ubuntu-latest
    steps:
      - run: wasm-pack test --headless --chrome
```

### 9.4 Release Criteria

- [ ] 100% test coverage on core, dsp, graph
- [ ] All benchmarks meet targets
- [ ] No clippy warnings
- [ ] No MIRI errors
- [ ] Cross-platform CI green
- [ ] Security audit passed
- [ ] Memory leak check passed (valgrind)

---

## Milestones

| Milestone | Target Date | Deliverable |
|-----------|-------------|-------------|
| M1: Core Complete | +2 weeks | Lock-free queue, SIMD, buffers tested |
| M2: DSP Complete | +4 weeks | All processors with tests |
| M3: HAL Working | +6 weeks | Audio I/O on Linux |
| M4: Graph Working | +8 weeks | Modular routing with PDC |
| M5: Siren Alpha | +12 weeks | Basic guitar playback |
| M6: Web Working | +14 weeks | Browser demo |
| M7: Siren Beta | +18 weeks | Full articulation support |
| M8: Production | +24 weeks | Release candidate |

---

## TDD Workflow

For each feature:

1. **RED**: Write a failing test that defines the expected behavior
2. **GREEN**: Write the minimum code to make the test pass
3. **REFACTOR**: Improve the code while keeping tests green

```bash
# TDD cycle
cargo test --lib -p amdusias-core -- test_new_feature  # RED: fails
# ... implement feature ...
cargo test --lib -p amdusias-core -- test_new_feature  # GREEN: passes
# ... refactor ...
cargo test --lib -p amdusias-core                       # All tests still pass
cargo clippy -p amdusias-core                          # No warnings
cargo bench -p amdusias-core -- bench_new_feature      # Performance acceptable
```

---

## Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p amdusias-core

# With output
cargo test -- --nocapture

# Single test
cargo test test_biquad_lowpass

# Benchmarks
cargo bench

# Coverage (requires cargo-llvm-cov)
cargo llvm-cov --html

# MIRI (undefined behavior check)
cargo +nightly miri test

# WASM tests
wasm-pack test --headless --chrome
```
