//! Sample-rate conversion — a streaming linear resampler.
//!
//! [`LinearResampler`] converts audio from one sample rate to another by linear
//! interpolation, keeping fractional-position state across buffers so a stream
//! can be resampled frame by frame with no clicks at the seams. Pure math, no
//! Kryx types — [`crate::stages::ResampleStage`] wraps it as a pipeline stage.
//!
//! Linear interpolation is cheap and click-free but not alias-free; high-quality
//! (windowed-sinc / polyphase) resampling is a later milestone. For large ratio
//! changes, filter first (e.g. a [`crate::biquad::Biquad`] low-pass) when
//! downsampling to tame aliasing.

/// A streaming linear resampler. Build with [`new`](Self::new), then feed
/// buffers through [`process`](Self::process); state carries across calls.
#[derive(Debug, Clone)]
pub struct LinearResampler {
    /// Input samples consumed per output sample (`from / to`).
    step: f64,
    /// Read position in the virtual index space `[-1, len)`, where `-1` is the
    /// previous buffer's last sample.
    pos: f64,
    /// Last sample of the previous buffer (virtual index `-1`).
    prev: f32,
    /// Whether `prev` holds a real sample yet.
    primed: bool,
}

impl LinearResampler {
    /// A resampler from `from_hz` to `to_hz`. Equal rates give a pass-through.
    pub fn new(from_hz: f32, to_hz: f32) -> Self {
        let step = (from_hz as f64 / (to_hz as f64).max(1.0e-9)).max(1.0e-9);
        Self {
            step,
            pos: 0.0,
            prev: 0.0,
            primed: false,
        }
    }

    /// Output samples per input sample (`to / from`). Useful to size buffers.
    pub fn ratio(&self) -> f64 {
        1.0 / self.step
    }

    /// Resample a buffer. Returns the output samples produced from this chunk;
    /// the fractional boundary is carried into the next call.
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let n = input.len();
        if n == 0 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity((n as f64 * self.ratio()) as usize + 1);
        // Produce while both interpolation taps (floor(pos), floor(pos)+1) are
        // available inside this buffer. The last fractional sample is emitted on
        // the next call, where input[n-1] becomes the virtual index -1 (`prev`).
        while self.pos < n as f64 - 1.0 {
            let i = self.pos.floor();
            let frac = (self.pos - i) as f32;
            let a = self.sample(input, i as i64);
            let b = self.sample(input, i as i64 + 1);
            out.push(a + (b - a) * frac);
            self.pos += self.step;
        }
        // Shift the reference frame: this buffer's last sample becomes `prev`.
        self.prev = input[n - 1];
        self.primed = true;
        self.pos -= n as f64;
        out
    }

    /// Clear state (as if starting a fresh stream); keeps the rate ratio.
    pub fn reset(&mut self) {
        self.pos = 0.0;
        self.prev = 0.0;
        self.primed = false;
    }

    /// Read virtual index `k`: `-1` is `prev`, `0..n` indexes `input`, out of
    /// range clamps to the nearest edge.
    fn sample(&self, input: &[f32], k: i64) -> f32 {
        if k < 0 {
            if self.primed {
                self.prev
            } else {
                input[0]
            }
        } else {
            let k = k as usize;
            if k < input.len() {
                input[k]
            } else {
                input[input.len() - 1]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_rates_pass_through() {
        let mut r = LinearResampler::new(48_000.0, 48_000.0);
        let out = r.process(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        // step == 1: output tracks input (one-sample boundary latency).
        assert!(out.len() >= 4);
        for (i, &v) in out.iter().enumerate() {
            assert!((v - i as f32).abs() < 1e-5, "out[{i}]={v}");
        }
    }

    #[test]
    fn upsampling_doubles_count_roughly() {
        // 1 kHz → 2 kHz: about twice as many samples.
        let mut r = LinearResampler::new(1_000.0, 2_000.0);
        let input: Vec<f32> = (0..1_000).map(|n| n as f32).collect();
        let out = r.process(&input);
        assert!(
            (out.len() as i64 - 2_000).abs() < 4,
            "expected ~2000, got {}",
            out.len()
        );
    }

    #[test]
    fn downsampling_halves_count_roughly() {
        // 2 kHz → 1 kHz: about half as many samples.
        let mut r = LinearResampler::new(2_000.0, 1_000.0);
        let input: Vec<f32> = (0..1_000).map(|n| n as f32).collect();
        let out = r.process(&input);
        assert!(
            (out.len() as i64 - 500).abs() < 4,
            "expected ~500, got {}",
            out.len()
        );
    }

    #[test]
    fn interpolates_midpoints_when_doubling() {
        // A clean ramp upsampled 2× should yield half-integer midpoints.
        let mut r = LinearResampler::new(1_000.0, 2_000.0);
        let out = r.process(&[0.0, 2.0, 4.0, 6.0, 8.0]);
        // Expect values near 0, 1, 2, 3, 4, ... (step 1 over the ramp of step 2).
        for w in out.windows(2) {
            assert!((w[1] - w[0] - 1.0).abs() < 0.05, "step {:?}", w);
        }
    }

    #[test]
    fn seams_are_continuous_across_buffers() {
        // Feeding a ramp in two chunks must match feeding it whole.
        let full: Vec<f32> = (0..200).map(|n| n as f32).collect();
        let mut whole = LinearResampler::new(3_000.0, 2_000.0);
        let out_whole = whole.process(&full);

        let mut split = LinearResampler::new(3_000.0, 2_000.0);
        let mut out_split = split.process(&full[..100]);
        out_split.extend(split.process(&full[100..]));

        assert_eq!(out_whole.len(), out_split.len());
        for (a, b) in out_whole.iter().zip(&out_split) {
            assert!((a - b).abs() < 1e-4, "seam mismatch {a} vs {b}");
        }
    }

    #[test]
    fn empty_input_yields_empty() {
        let mut r = LinearResampler::new(48_000.0, 44_100.0);
        assert!(r.process(&[]).is_empty());
    }
}
