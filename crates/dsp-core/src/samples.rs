//! Sample-level DSP primitives on `f32` audio, pure and dependency-free.
//!
//! These functions know nothing about Kryx, pipelines, or buffers — they are
//! plain math on slices of samples, so they are reusable anywhere and trivial to
//! test. The Kryx integration (wrapping them as pipeline stages) lives in
//! [`crate::stages`]; the math stays separate (Rule 1: the generic doesn't know
//! the specific).

/// Scale every sample by `factor` in place (a linear gain / volume change).
#[inline]
pub fn gain(samples: &mut [f32], factor: f32) {
    for s in samples {
        *s *= factor;
    }
}

/// Mix `src` into `dst` sample-wise (`dst[i] += src[i]`). The shorter length
/// wins, so mixing a short clip into a longer one touches only the overlap.
#[inline]
pub fn mix_into(dst: &mut [f32], src: &[f32]) {
    for (d, s) in dst.iter_mut().zip(src) {
        *d += *s;
    }
}

/// Clamp every sample to `[-1.0, 1.0]`, the normalized full-scale range.
#[inline]
pub fn clamp(samples: &mut [f32]) {
    for s in samples {
        *s = s.clamp(-1.0, 1.0);
    }
}

/// Peak absolute amplitude across the buffer (0.0 for an empty buffer).
pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |m, s| m.max(s.abs()))
}

/// Root-mean-square level across the buffer (0.0 for an empty buffer).
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gain_scales_exactly() {
        let mut s = [0.5, -0.25, 1.0];
        gain(&mut s, 2.0);
        assert_eq!(s, [1.0, -0.5, 2.0]);
    }

    #[test]
    fn mix_sums_over_the_overlap() {
        let mut a = [0.1, 0.2, 0.3];
        mix_into(&mut a, &[0.4, 0.4]); // shorter src
        assert!((a[0] - 0.5).abs() < 1e-6);
        assert!((a[1] - 0.6).abs() < 1e-6);
        assert!((a[2] - 0.3).abs() < 1e-6); // untouched
    }

    #[test]
    fn clamp_bounds_full_scale() {
        let mut s = [2.0, -3.0, 0.5];
        clamp(&mut s);
        assert_eq!(s, [1.0, -1.0, 0.5]);
    }

    #[test]
    fn peak_and_rms() {
        let s = [0.0, 1.0, -1.0, 0.0];
        assert_eq!(peak(&s), 1.0);
        assert!((rms(&s) - (0.5f32).sqrt()).abs() < 1e-6);
        assert_eq!(peak(&[]), 0.0);
        assert_eq!(rms(&[]), 0.0);
    }
}
