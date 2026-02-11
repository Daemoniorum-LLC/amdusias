//! SIMD-optimized audio processing functions.
//!
//! This module provides vectorized implementations of common audio operations.
//! Functions automatically select the best available instruction set at runtime.

use crate::Sample;

/// SIMD lane width for f32 operations.
#[cfg(target_arch = "x86_64")]
pub const SIMD_LANES: usize = 8; // AVX2: 256-bit = 8 x f32

#[cfg(target_arch = "aarch64")]
pub const SIMD_LANES: usize = 4; // NEON: 128-bit = 4 x f32

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub const SIMD_LANES: usize = 4; // Fallback

/// Applies gain to a buffer using SIMD operations.
///
/// # Arguments
///
/// - `samples`: The sample buffer to modify in-place.
/// - `gain`: The gain multiplier to apply.
#[inline]
pub fn apply_gain_simd(samples: &mut [Sample], gain: Sample) {
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: We've verified AVX2 is available.
            unsafe { apply_gain_avx2(samples, gain) };
            return;
        }
    }

    // Scalar fallback
    apply_gain_scalar(samples, gain);
}

/// Scalar implementation of gain application.
#[inline]
fn apply_gain_scalar(samples: &mut [Sample], gain: Sample) {
    for sample in samples.iter_mut() {
        *sample *= gain;
    }
}

/// AVX2 implementation of gain application.
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
#[target_feature(enable = "avx2")]
unsafe fn apply_gain_avx2(samples: &mut [Sample], gain: Sample) {
    use core::arch::x86_64::*;

    let gain_vec = _mm256_set1_ps(gain);
    let chunks = samples.len() / 8;

    let ptr = samples.as_mut_ptr();

    for i in 0..chunks {
        let offset = i * 8;
        let data = _mm256_loadu_ps(ptr.add(offset));
        let result = _mm256_mul_ps(data, gain_vec);
        _mm256_storeu_ps(ptr.add(offset), result);
    }

    // Handle remaining samples
    let remainder_start = chunks * 8;
    for sample in samples[remainder_start..].iter_mut() {
        *sample *= gain;
    }
}

/// Mixes two buffers together using SIMD operations.
///
/// Adds `src` samples to `dst` samples in-place.
#[inline]
pub fn mix_buffers_simd(dst: &mut [Sample], src: &[Sample]) {
    debug_assert_eq!(dst.len(), src.len(), "buffer sizes must match");

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: We've verified AVX2 is available.
            unsafe { mix_buffers_avx2(dst, src) };
            return;
        }
    }

    // Scalar fallback
    mix_buffers_scalar(dst, src);
}

/// Scalar implementation of buffer mixing.
#[inline]
fn mix_buffers_scalar(dst: &mut [Sample], src: &[Sample]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d += *s;
    }
}

/// AVX2 implementation of buffer mixing.
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
#[target_feature(enable = "avx2")]
unsafe fn mix_buffers_avx2(dst: &mut [Sample], src: &[Sample]) {
    use core::arch::x86_64::*;

    let chunks = dst.len() / 8;

    let dst_ptr = dst.as_mut_ptr();
    let src_ptr = src.as_ptr();

    for i in 0..chunks {
        let offset = i * 8;
        let dst_data = _mm256_loadu_ps(dst_ptr.add(offset));
        let src_data = _mm256_loadu_ps(src_ptr.add(offset));
        let result = _mm256_add_ps(dst_data, src_data);
        _mm256_storeu_ps(dst_ptr.add(offset), result);
    }

    // Handle remaining samples
    let remainder_start = chunks * 8;
    for (d, s) in dst[remainder_start..].iter_mut().zip(src[remainder_start..].iter()) {
        *d += *s;
    }
}

/// Finds the peak absolute value in a buffer.
#[inline]
#[must_use]
pub fn find_peak(samples: &[Sample]) -> Sample {
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: We've verified AVX2 is available.
            return unsafe { find_peak_avx2(samples) };
        }
    }

    find_peak_scalar(samples)
}

/// Scalar implementation of peak finding.
#[inline]
fn find_peak_scalar(samples: &[Sample]) -> Sample {
    samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f32, |a, b| a.max(b))
}

/// AVX2 implementation of peak finding.
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
#[target_feature(enable = "avx2")]
unsafe fn find_peak_avx2(samples: &[Sample]) -> Sample {
    use core::arch::x86_64::*;

    let sign_mask = _mm256_set1_ps(-0.0); // Mask for clearing sign bit
    let mut max_vec = _mm256_setzero_ps();

    let chunks = samples.len() / 8;
    let ptr = samples.as_ptr();

    for i in 0..chunks {
        let offset = i * 8;
        let data = _mm256_loadu_ps(ptr.add(offset));
        let abs_data = _mm256_andnot_ps(sign_mask, data); // Absolute value
        max_vec = _mm256_max_ps(max_vec, abs_data);
    }

    // Horizontal max reduction
    let mut max_arr = [0.0_f32; 8];
    _mm256_storeu_ps(max_arr.as_mut_ptr(), max_vec);
    let mut max_val = max_arr.iter().fold(0.0_f32, |a, &b| a.max(b));

    // Handle remaining samples
    let remainder_start = chunks * 8;
    for sample in samples[remainder_start..].iter() {
        max_val = max_val.max(sample.abs());
    }

    max_val
}

/// Calculates RMS (Root Mean Square) of a buffer.
#[inline]
#[must_use]
pub fn calculate_rms(samples: &[Sample]) -> Sample {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_gain() {
        let mut samples = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        apply_gain_simd(&mut samples, 0.5);

        for (i, &sample) in samples.iter().enumerate() {
            assert!((sample - (i + 1) as f32 * 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_mix_buffers() {
        let mut dst = [1.0, 2.0, 3.0, 4.0];
        let src = [0.5, 0.5, 0.5, 0.5];
        mix_buffers_simd(&mut dst, &src);

        assert!((dst[0] - 1.5).abs() < 1e-6);
        assert!((dst[1] - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_find_peak() {
        let samples = [-0.5, 0.3, -0.8, 0.6, -0.2];
        let peak = find_peak(&samples);
        assert!((peak - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_calculate_rms() {
        let samples = [1.0, -1.0, 1.0, -1.0];
        let rms = calculate_rms(&samples);
        assert!((rms - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_simd_gain_matches_scalar() {
        // Verify SIMD and scalar produce identical results
        let original: Vec<Sample> = (0..1024).map(|i| (i as f32) * 0.01 - 5.0).collect();
        let gain = 0.7;

        let mut simd_result = original.clone();
        let mut scalar_result = original.clone();

        apply_gain_simd(&mut simd_result, gain);
        apply_gain_scalar(&mut scalar_result, gain);

        for (i, (simd, scalar)) in simd_result.iter().zip(scalar_result.iter()).enumerate() {
            assert!(
                (simd - scalar).abs() < 1e-6,
                "Mismatch at index {}: SIMD {} vs scalar {}",
                i,
                simd,
                scalar
            );
        }
    }

    #[test]
    fn test_simd_mix_matches_scalar() {
        // Verify SIMD and scalar mixing produce identical results
        let dst_original: Vec<Sample> = (0..1024).map(|i| (i as f32) * 0.01).collect();
        let src: Vec<Sample> = (0..1024).map(|i| (i as f32) * -0.005 + 1.0).collect();

        let mut simd_result = dst_original.clone();
        let mut scalar_result = dst_original.clone();

        mix_buffers_simd(&mut simd_result, &src);
        mix_buffers_scalar(&mut scalar_result, &src);

        for (i, (simd, scalar)) in simd_result.iter().zip(scalar_result.iter()).enumerate() {
            assert!(
                (simd - scalar).abs() < 1e-6,
                "Mismatch at index {}: SIMD {} vs scalar {}",
                i,
                simd,
                scalar
            );
        }
    }

    #[test]
    fn test_simd_peak_matches_scalar() {
        // Verify SIMD and scalar peak finding match
        let samples: Vec<Sample> = (0..1024)
            .map(|i| ((i as f32) * 0.1).sin() * 0.9)
            .collect();

        let simd_peak = find_peak(&samples);
        let scalar_peak = find_peak_scalar(&samples);

        assert!(
            (simd_peak - scalar_peak).abs() < 1e-6,
            "Peak mismatch: SIMD {} vs scalar {}",
            simd_peak,
            scalar_peak
        );
    }

    #[test]
    fn test_apply_gain_various_sizes() {
        // Test with sizes that aren't multiples of SIMD lane width
        for size in [1, 7, 8, 9, 15, 16, 17, 31, 32, 33, 127, 128, 129] {
            let mut samples: Vec<Sample> = (0..size).map(|i| i as f32).collect();
            let expected: Vec<Sample> = (0..size).map(|i| i as f32 * 2.0).collect();

            apply_gain_simd(&mut samples, 2.0);

            for (i, (got, want)) in samples.iter().zip(expected.iter()).enumerate() {
                assert!(
                    (got - want).abs() < 1e-6,
                    "Size {}, index {}: got {}, want {}",
                    size,
                    i,
                    got,
                    want
                );
            }
        }
    }

    #[test]
    fn test_mix_buffers_various_sizes() {
        // Test with sizes that aren't multiples of SIMD lane width
        for size in [1, 7, 8, 9, 15, 16, 17] {
            let mut dst: Vec<Sample> = vec![1.0; size];
            let src: Vec<Sample> = vec![0.5; size];

            mix_buffers_simd(&mut dst, &src);

            for (i, &sample) in dst.iter().enumerate() {
                assert!(
                    (sample - 1.5).abs() < 1e-6,
                    "Size {}, index {}: got {}, want 1.5",
                    size,
                    i,
                    sample
                );
            }
        }
    }

    #[test]
    fn test_find_peak_negative() {
        // Peak should be absolute value
        let samples = [-0.9, -0.5, -0.1];
        let peak = find_peak(&samples);
        assert!((peak - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_find_peak_empty() {
        let samples: [Sample; 0] = [];
        let peak = find_peak(&samples);
        assert_eq!(peak, 0.0);
    }

    #[test]
    fn test_calculate_rms_silence() {
        let samples = [0.0, 0.0, 0.0, 0.0];
        let rms = calculate_rms(&samples);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_calculate_rms_empty() {
        let samples: [Sample; 0] = [];
        let rms = calculate_rms(&samples);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_calculate_rms_sine_wave() {
        // RMS of a sine wave = amplitude / sqrt(2)
        let amplitude = 0.8;
        let samples: Vec<Sample> = (0..10000)
            .map(|i| amplitude * (2.0 * std::f32::consts::PI * i as f32 / 100.0).sin())
            .collect();

        let rms = calculate_rms(&samples);
        let expected = amplitude / 2.0_f32.sqrt();

        assert!(
            (rms - expected).abs() < 0.01,
            "RMS {} should be close to {}",
            rms,
            expected
        );
    }
}
