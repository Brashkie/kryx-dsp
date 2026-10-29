//! A biquad filter — second-order IIR, the workhorse of audio EQ and filtering.
//!
//! Coefficients follow the well-known RBJ "Audio EQ Cookbook" formulas; the
//! filter runs in Transposed Direct Form II (two state variables, numerically
//! friendly). Pure math, no Kryx types — [`crate::stages::BiquadStage`] wraps it
//! as a pipeline stage that keeps the state across frames.

use std::f32::consts::PI;

/// A second-order IIR (biquad) filter with persistent state.
///
/// Build one with [`low_pass`](Self::low_pass) or [`high_pass`](Self::high_pass),
/// then feed samples through [`process`](Self::process). State carries across
/// calls, so a stream can be filtered frame by frame; [`reset`](Self::reset)
/// clears it.
#[derive(Debug, Clone)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    /// A low-pass filter at `cutoff_hz` with resonance `q` (0.707 ≈ Butterworth).
    pub fn low_pass(sample_rate: f32, cutoff_hz: f32, q: f32) -> Self {
        let (cw, alpha) = Self::intermediates(sample_rate, cutoff_hz, q);
        let b0 = (1.0 - cw) / 2.0;
        let b1 = 1.0 - cw;
        let b2 = (1.0 - cw) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cw;
        let a2 = 1.0 - alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    /// A high-pass filter at `cutoff_hz` with resonance `q`.
    pub fn high_pass(sample_rate: f32, cutoff_hz: f32, q: f32) -> Self {
        let (cw, alpha) = Self::intermediates(sample_rate, cutoff_hz, q);
        let b0 = (1.0 + cw) / 2.0;
        let b1 = -(1.0 + cw);
        let b2 = (1.0 + cw) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cw;
        let a2 = 1.0 - alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    /// A band-pass filter centered at `center_hz` (constant 0 dB peak gain).
    pub fn band_pass(sample_rate: f32, center_hz: f32, q: f32) -> Self {
        let (cw, alpha) = Self::intermediates(sample_rate, center_hz, q);
        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cw;
        let a2 = 1.0 - alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    /// A notch (band-reject) filter that removes `center_hz`.
    pub fn notch(sample_rate: f32, center_hz: f32, q: f32) -> Self {
        let (cw, alpha) = Self::intermediates(sample_rate, center_hz, q);
        let b0 = 1.0;
        let b1 = -2.0 * cw;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cw;
        let a2 = 1.0 - alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    /// A peaking EQ: boost or cut `gain_db` around `center_hz` with bandwidth `q`.
    pub fn peaking(sample_rate: f32, center_hz: f32, q: f32, gain_db: f32) -> Self {
        let (cw, alpha) = Self::intermediates(sample_rate, center_hz, q);
        let a = 10.0_f32.powf(gain_db / 40.0);
        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cw;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cw;
        let a2 = 1.0 - alpha / a;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    /// A low-shelf: boost or cut `gain_db` below `corner_hz`.
    pub fn low_shelf(sample_rate: f32, corner_hz: f32, q: f32, gain_db: f32) -> Self {
        let (cw, alpha) = Self::intermediates(sample_rate, corner_hz, q);
        let a = 10.0_f32.powf(gain_db / 40.0);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        let b0 = a * ((a + 1.0) - (a - 1.0) * cw + two_sqrt_a_alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cw);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cw - two_sqrt_a_alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cw + two_sqrt_a_alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cw);
        let a2 = (a + 1.0) + (a - 1.0) * cw - two_sqrt_a_alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    /// A high-shelf: boost or cut `gain_db` above `corner_hz`.
    pub fn high_shelf(sample_rate: f32, corner_hz: f32, q: f32, gain_db: f32) -> Self {
        let (cw, alpha) = Self::intermediates(sample_rate, corner_hz, q);
        let a = 10.0_f32.powf(gain_db / 40.0);
        let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
        let b0 = a * ((a + 1.0) + (a - 1.0) * cw + two_sqrt_a_alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cw);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cw - two_sqrt_a_alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cw + two_sqrt_a_alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cw);
        let a2 = (a + 1.0) - (a - 1.0) * cw - two_sqrt_a_alpha;
        Self::normalized(b0, b1, b2, a0, a1, a2)
    }

    fn intermediates(sample_rate: f32, cutoff_hz: f32, q: f32) -> (f32, f32) {
        let w0 = 2.0 * PI * cutoff_hz / sample_rate;
        let cw = w0.cos();
        let sw = w0.sin();
        let alpha = sw / (2.0 * q.max(1.0e-4));
        (cw, alpha)
    }

    fn normalized(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Filter one sample, advancing the state.
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    /// Filter a whole buffer in place.
    pub fn process(&mut self, buf: &mut [f32]) {
        for x in buf {
            *x = self.process_sample(*x);
        }
    }

    /// Clear the filter state (as if starting a fresh stream).
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_pass_passes_dc() {
        let mut f = Biquad::low_pass(48_000.0, 1_000.0, 0.707);
        let mut buf = vec![1.0f32; 2_000];
        f.process(&mut buf);
        // A constant (DC) passes a low-pass at unity gain once settled.
        assert!((buf[buf.len() - 1] - 1.0).abs() < 1e-3);
    }

    #[test]
    fn low_pass_attenuates_high_frequency() {
        let sr = 48_000.0;
        let mut f = Biquad::low_pass(sr, 1_000.0, 0.707);
        let mut buf: Vec<f32> = (0..2_000)
            .map(|n| (2.0 * PI * 12_000.0 * n as f32 / sr).sin())
            .collect();
        f.process(&mut buf);
        // 12 kHz is far above a 1 kHz cutoff → strongly attenuated.
        let peak_out = buf[500..].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak_out < 0.2, "peak {peak_out} should be heavily attenuated");
    }

    #[test]
    fn high_pass_blocks_dc() {
        let mut f = Biquad::high_pass(48_000.0, 1_000.0, 0.707);
        let mut buf = vec![1.0f32; 2_000];
        f.process(&mut buf);
        // DC is blocked by a high-pass → settles toward zero.
        assert!(buf[buf.len() - 1].abs() < 1e-2);
    }

    #[test]
    fn reset_clears_state() {
        let mut f = Biquad::low_pass(48_000.0, 1_000.0, 0.707);
        f.process(&mut [1.0, 1.0, 1.0]);
        f.reset();
        // After reset the first output equals b0 * x (no carried state).
        let y = f.process_sample(1.0);
        assert!((y - f.b0).abs() < 1e-7);
    }

    /// Steady-state amplitude of a filter at frequency `hz` (drives a sine,
    /// measures the peak of the tail once the transient has passed).
    fn response_at(mut f: Biquad, sr: f32, hz: f32) -> f32 {
        let buf: Vec<f32> = (0..8_000)
            .map(|n| (2.0 * PI * hz * n as f32 / sr).sin())
            .collect();
        let mut out = buf;
        f.process(&mut out);
        out[4_000..].iter().fold(0.0f32, |m, s| m.max(s.abs()))
    }

    #[test]
    fn band_pass_passes_center_rejects_extremes() {
        let sr = 48_000.0;
        let center = response_at(Biquad::band_pass(sr, 1_000.0, 1.0), sr, 1_000.0);
        let low = response_at(Biquad::band_pass(sr, 1_000.0, 1.0), sr, 60.0);
        let high = response_at(Biquad::band_pass(sr, 1_000.0, 1.0), sr, 12_000.0);
        assert!(center > 0.9, "center {center} should pass near unity");
        assert!(low < 0.3 && high < 0.3, "extremes should be attenuated");
    }

    #[test]
    fn notch_rejects_center_passes_elsewhere() {
        let sr = 48_000.0;
        let at_center = response_at(Biquad::notch(sr, 1_000.0, 1.0), sr, 1_000.0);
        let away = response_at(Biquad::notch(sr, 1_000.0, 1.0), sr, 8_000.0);
        assert!(at_center < 0.2, "center {at_center} should be rejected");
        assert!(away > 0.8, "away {away} should pass");
    }

    #[test]
    fn peaking_boosts_at_center() {
        let sr = 48_000.0;
        let boosted = response_at(Biquad::peaking(sr, 1_000.0, 1.0, 6.0), sr, 1_000.0);
        // +6 dB ≈ ×1.995 at the center frequency.
        assert!(boosted > 1.8 && boosted < 2.1, "boost {boosted} ≈ ×2");
    }

    #[test]
    fn low_shelf_lifts_dc() {
        let mut f = Biquad::low_shelf(48_000.0, 1_000.0, 0.707, 6.0);
        let mut buf = vec![1.0f32; 4_000];
        f.process(&mut buf);
        // DC sits well inside a low shelf → lifted by ~+6 dB (×1.995).
        assert!((buf[buf.len() - 1] - 1.995).abs() < 0.1);
    }

    #[test]
    fn high_shelf_leaves_dc() {
        let mut f = Biquad::high_shelf(48_000.0, 1_000.0, 0.707, 6.0);
        let mut buf = vec![1.0f32; 4_000];
        f.process(&mut buf);
        // DC is below a high shelf → unchanged at unity.
        assert!((buf[buf.len() - 1] - 1.0).abs() < 0.05);
    }
}
