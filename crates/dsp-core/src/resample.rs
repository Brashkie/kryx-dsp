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

// ─── High-quality windowed-sinc resampler ────────────────────────────────────

/// Modified Bessel function of the first kind, order 0 (series expansion).
fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let mut k = 1.0;
    loop {
        term *= (x * x) / (4.0 * k * k);
        sum += term;
        if term < 1.0e-12 * sum {
            break;
        }
        k += 1.0;
    }
    sum
}

#[inline]
fn sinc(x: f64) -> f64 {
    if x.abs() < 1.0e-9 {
        1.0
    } else {
        let px = std::f64::consts::PI * x;
        px.sin() / px
    }
}

/// A streaming high-quality resampler using a Kaiser-windowed-sinc kernel,
/// evaluated through a precomputed polyphase table with linear phase
/// interpolation. Unlike [`LinearResampler`], it is anti-aliased: when
/// downsampling it attenuates content above the new Nyquist frequency.
///
/// `quality` is the number of sinc zero-crossings per side (taps = `2 * quality`);
/// 16 is a good default. Higher is sharper and costlier. State carries across
/// `process` calls, so a stream stays seamless.
#[derive(Debug, Clone)]
pub struct SincResampler {
    zc: usize,
    phases: usize,
    taps: usize,
    step: f64,
    /// Polyphase coefficient table, `(phases + 1) * taps`, each phase row
    /// normalized to unity DC gain.
    table: Vec<f32>,
    /// Working buffer: `taps - 1` left-context samples followed by pending input.
    buf: Vec<f32>,
    /// Read position (input-sample coords) into `buf`.
    pos: f64,
}

impl SincResampler {
    /// A resampler from `from_hz` to `to_hz` with the given `quality`
    /// (zero-crossings per side; clamped to at least 2). Equal rates still filter.
    pub fn new(from_hz: f32, to_hz: f32, quality: u32) -> Self {
        let zc = (quality as usize).max(2);
        let phases = 512usize;
        let beta = 9.0f64;
        let taps = 2 * zc;
        let step = (from_hz as f64 / (to_hz as f64).max(1.0e-9)).max(1.0e-9);
        let cutoff = (to_hz as f64 / from_hz as f64).min(1.0).max(1.0e-9);

        let i0_beta = bessel_i0(beta);
        let mut table = vec![0.0f32; (phases + 1) * taps];
        for p in 0..=phases {
            let frac = p as f64 / phases as f64;
            let mut acc = 0.0f64;
            let row_start = p * taps;
            for m in 0..taps {
                let off = frac + (zc as f64 - 1.0) - m as f64;
                let u = off / zc as f64;
                let w = if u.abs() >= 1.0 {
                    0.0
                } else {
                    bessel_i0(beta * (1.0 - u * u).sqrt()) / i0_beta
                };
                let h = cutoff * sinc(cutoff * off) * w;
                table[row_start + m] = h as f32;
                acc += h;
            }
            // Normalize the phase row to unity DC gain.
            if acc.abs() > 1.0e-12 {
                for m in 0..taps {
                    table[row_start + m] = (table[row_start + m] as f64 / acc) as f32;
                }
            }
        }

        Self {
            zc,
            phases,
            taps,
            step,
            table,
            buf: vec![0.0; taps - 1],
            pos: (zc - 1) as f64,
        }
    }

    /// Output samples per input sample (`to / from`).
    pub fn ratio(&self) -> f64 {
        1.0 / self.step
    }

    /// Resample a buffer, returning the samples produced from this chunk.
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        self.buf.extend_from_slice(input);
        let len = self.buf.len();
        let mut out = Vec::with_capacity((input.len() as f64 * self.ratio()) as usize + 1);

        while self.pos + (self.zc as f64) < len as f64 {
            let n = self.pos.floor() as usize;
            let frac = self.pos - n as f64;
            let fp = frac * self.phases as f64;
            let pi = fp.floor() as usize;
            let a = (fp - pi as f64) as f32;
            let r0 = pi * self.taps;
            let r1 = (pi + 1) * self.taps;
            let base = n - (self.zc - 1);

            let mut y = 0.0f32;
            for m in 0..self.taps {
                let c = self.table[r0 + m] * (1.0 - a) + self.table[r1 + m] * a;
                y += self.buf[base + m] * c;
            }
            out.push(y);
            self.pos += self.step;
        }

        // Drop consumed samples, keeping `taps - 1` of left context.
        let keep = self.pos.floor() as usize;
        if keep > self.zc - 1 {
            let drop = keep - (self.zc - 1);
            self.buf.drain(0..drop);
            self.pos -= drop as f64;
        }
        out
    }

    /// Clear state (fresh stream); keeps the rate ratio and kernel.
    pub fn reset(&mut self) {
        self.buf.clear();
        self.buf.resize(self.taps - 1, 0.0);
        self.pos = (self.zc - 1) as f64;
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

    // ── SincResampler ──

    use std::f32::consts::PI;

    #[test]
    fn sinc_passes_dc_and_counts() {
        let mut up = SincResampler::new(24_000.0, 48_000.0, 16);
        let out = up.process(&vec![1.0; 1_000]);
        assert!((out.len() as i64 - 2_000).abs() < 40);
        assert!((out[out.len() - 1] - 1.0).abs() < 1e-2, "DC {}", out[out.len() - 1]);
    }

    #[test]
    fn sinc_preserves_in_band_tone() {
        // 1 kHz through 48k→24k stays near unity amplitude.
        let mut r = SincResampler::new(48_000.0, 24_000.0, 16);
        let sig: Vec<f32> = (0..4_000).map(|n| (2.0 * PI * 1_000.0 * n as f32 / 48_000.0).sin()).collect();
        let out = r.process(&sig);
        let peak = out[200..].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - 1.0).abs() < 0.05, "peak {peak}");
    }

    #[test]
    fn sinc_rejects_above_new_nyquist() {
        // 20 kHz when downsampling 48k→24k (new Nyquist 12k) must be killed.
        let mut r = SincResampler::new(48_000.0, 24_000.0, 16);
        let sig: Vec<f32> = (0..4_000).map(|n| (2.0 * PI * 20_000.0 * n as f32 / 48_000.0).sin()).collect();
        let out = r.process(&sig);
        let peak = out[200..].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak < 0.05, "aliased energy {peak} should be near zero");
    }

    #[test]
    fn sinc_seams_are_continuous() {
        let full: Vec<f32> = (0..600).map(|n| (2.0 * PI * 997.0 * n as f32 / 48_000.0).sin()).collect();
        let mut whole = SincResampler::new(48_000.0, 44_100.0, 16);
        let out_whole = whole.process(&full);
        let mut split = SincResampler::new(48_000.0, 44_100.0, 16);
        let mut out_split = split.process(&full[..300]);
        out_split.extend(split.process(&full[300..]));
        let n = out_whole.len().min(out_split.len());
        for i in 0..n {
            assert!((out_whole[i] - out_split[i]).abs() < 1e-5, "seam at {i}");
        }
    }
}
