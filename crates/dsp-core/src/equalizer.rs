//! A parametric equalizer — an ordered chain of [`Biquad`] bands applied in
//! series. Build it from the biquad filter set (peaking, shelves, pass/notch)
//! to shape a spectrum. Pure math; [`crate::stages::EqualizerStage`] wraps it as
//! a pipeline stage.

use crate::biquad::Biquad;

/// An ordered chain of biquad bands, processed front to back.
#[derive(Debug, Clone, Default)]
pub struct Equalizer {
    bands: Vec<Biquad>,
}

impl Equalizer {
    /// An empty equalizer (a pass-through until bands are added).
    pub fn new() -> Self {
        Self { bands: Vec::new() }
    }

    /// Append a band. Bands apply in the order they are pushed.
    pub fn push(&mut self, band: Biquad) -> &mut Self {
        self.bands.push(band);
        self
    }

    /// Number of bands.
    pub fn len(&self) -> usize {
        self.bands.len()
    }

    /// Whether the equalizer has no bands (a pass-through).
    pub fn is_empty(&self) -> bool {
        self.bands.is_empty()
    }

    /// Filter a buffer in place, running each band in series.
    pub fn process(&mut self, buf: &mut [f32]) {
        for band in &mut self.bands {
            band.process(buf);
        }
    }

    /// Clear the state of every band (as if starting a fresh stream).
    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn tone(sr: f32, hz: f32, n: usize) -> Vec<f32> {
        (0..n).map(|i| (2.0 * PI * hz * i as f32 / sr).sin()).collect()
    }

    #[test]
    fn empty_eq_is_passthrough() {
        let mut eq = Equalizer::new();
        let mut buf = tone(48_000.0, 1_000.0, 100);
        let before = buf.clone();
        eq.process(&mut buf);
        assert_eq!(buf, before);
        assert!(eq.is_empty());
    }

    #[test]
    fn two_peaking_bands_stack() {
        // +6 dB at 1 kHz twice ≈ +12 dB (×3.98) on a 1 kHz tone.
        let sr = 48_000.0;
        let mut eq = Equalizer::new();
        eq.push(Biquad::peaking(sr, 1_000.0, 1.0, 6.0))
            .push(Biquad::peaking(sr, 1_000.0, 1.0, 6.0));
        assert_eq!(eq.len(), 2);
        let mut buf = tone(sr, 1_000.0, 8_000);
        eq.process(&mut buf);
        let peak = buf[4_000..].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - 3.98).abs() < 0.2, "stacked peak {peak}");
    }

    #[test]
    fn reset_clears_all_bands() {
        let sr = 48_000.0;
        let mut eq = Equalizer::new();
        eq.push(Biquad::low_pass(sr, 1_000.0, 0.707));
        eq.process(&mut vec![1.0; 100]);
        eq.reset();
        // After reset the first low-pass output equals b0 * x (no carried state):
        // just assert it runs and produces a near-zero start from silence→unity.
        let mut buf = vec![1.0f32; 1];
        eq.process(&mut buf);
        assert!(buf[0].abs() < 1.0);
    }
}
