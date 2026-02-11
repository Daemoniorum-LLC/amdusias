//! Benchmarks for voice allocation and processing.
//!
//! Run with: cargo bench -p amdusias-siren

use amdusias_siren::{
    sample::{SampleId, SampleZone},
    voice::{VoiceAllocator, VoiceStealingMode},
    Articulation,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Benchmark voice allocation with idle voices available.
fn bench_voice_allocation_idle(c: &mut Criterion) {
    c.bench_function("voice_allocate_idle", |b| {
        let mut allocator = VoiceAllocator::new(64, 48000.0);

        b.iter(|| {
            // Allocate a voice (should find idle voice)
            let voice = allocator.allocate();
            black_box(voice);

            // Reset for next iteration by releasing
            allocator.release_all();
        });
    });
}

/// Benchmark voice allocation with stealing required.
fn bench_voice_allocation_stealing(c: &mut Criterion) {
    let zone = SampleZone::new(SampleId(1), 60);

    c.bench_function("voice_allocate_steal_oldest", |b| {
        let mut allocator = VoiceAllocator::new(8, 48000.0);
        allocator.set_stealing_mode(VoiceStealingMode::Oldest);

        // Fill all voices
        for i in 0..8 {
            if let Some(voice) = allocator.allocate() {
                voice.trigger(60 + i, 100, Articulation::Sustain, &zone, 0);
            }
        }

        b.iter(|| {
            // Now allocation must steal
            let voice = allocator.allocate();
            black_box(voice);
        });
    });
}

/// Benchmark voice processing throughput.
fn bench_voice_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("voice_processing");

    // Test with different sample sizes
    for sample_len in [256, 1024, 4096, 16384].iter() {
        let zone = SampleZone::new(SampleId(1), 60);
        let sample_data: Vec<f32> = vec![0.5; *sample_len];

        group.bench_with_input(
            BenchmarkId::new("mono", sample_len),
            sample_len,
            |b, _| {
                let mut allocator = VoiceAllocator::new(1, 48000.0);
                if let Some(voice) = allocator.allocate() {
                    voice.trigger(60, 100, Articulation::Sustain, &zone, 0);
                }

                b.iter(|| {
                    for voice in allocator.active_voices() {
                        let output = voice.process(&sample_data, 1);
                        black_box(output);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark polyphonic voice processing (chord playback).
fn bench_polyphonic_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("polyphonic_processing");

    // Test with different voice counts
    for voice_count in [4, 8, 16, 32].iter() {
        let zone = SampleZone::new(SampleId(1), 60);
        let sample_data: Vec<f32> = vec![0.5; 4096];

        group.bench_with_input(
            BenchmarkId::new("voices", voice_count),
            voice_count,
            |b, &count| {
                let mut allocator = VoiceAllocator::new(count, 48000.0);

                // Trigger voices
                for i in 0..count {
                    if let Some(voice) = allocator.allocate() {
                        voice.trigger(60 + i as u8, 100, Articulation::Sustain, &zone, 0);
                    }
                }

                b.iter(|| {
                    let mut total = 0.0f32;
                    for voice in allocator.active_voices() {
                        let (l, r) = voice.process(&sample_data, 1);
                        total += l + r;
                    }
                    black_box(total);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark finding a voice by note.
fn bench_find_voice(c: &mut Criterion) {
    let zone = SampleZone::new(SampleId(1), 60);

    c.bench_function("find_voice_by_note", |b| {
        let mut allocator = VoiceAllocator::new(32, 48000.0);

        // Trigger voices with different notes
        for i in 0..32 {
            if let Some(voice) = allocator.allocate() {
                voice.trigger(36 + i, 100, Articulation::Sustain, &zone, 0);
            }
        }

        b.iter(|| {
            // Search for a note in the middle
            let voice = allocator.find_voice(black_box(52));
            black_box(voice);
        });
    });
}

criterion_group!(
    benches,
    bench_voice_allocation_idle,
    bench_voice_allocation_stealing,
    bench_voice_processing,
    bench_polyphonic_processing,
    bench_find_voice,
);

criterion_main!(benches);
