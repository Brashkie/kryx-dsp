//! Audio effects — small, stateful DSP blocks: DC blocker, delay/echo, limiter,
//! and a stateless soft-clip. Pure math, no Kryx types; [`crate::stages`] wraps
//! them as pipeline stages.

/// A DC blocker: a one-pole high-pass that removes the DC offset (0 Hz) while
/// passing everything else, via `y[n] = x[n] - x[n-1] + R * y[n-1]`.
#[derive(Debug, Clone)]
pub struct DcBlocker {
    r: f32,
    x1: f32,
    y1: f32,
}

impl DcBlocker {
    /// A DC blocker with pole `r` (0.995 is a good default; closer to 1.0 passes
    /// lower frequencies).
    pub fn new(r: f32) -> Self {
        Self { r, x1: 0.0, y1: 0.0 }
    }

    /// Process one sample, advancing state.
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = x - self.x1 + self.r * self.y1;
        self.x1 = x;
        self.y1 = y;
        y
    }

    /// Process a buffer in place.
    pub fn process(&mut self, buf: &mut [f32]) {
        for x in buf {
            *x = self.process_sample(*x);
        }
    }

    /// Clear state.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }
}

impl Default for DcBlocker {
    fn default() -> Self {
        Self::new(0.995)
    }
}

/// A delay line with feedback and a dry/wet mix (an echo).
#[derive(Debug, Clone)]
pub struct Delay {
    buf: Vec<f32>,
    idx: usize,
    delay: usize,
    feedback: f32,
    mix: f32,
}

impl Delay {
    /// A delay of `delay_samples` (capped at `max_delay_samples`), with
    /// `feedback` (0..1, how much of the echo re-circulates) and `mix`
    /// (0 = dry only, 1 = wet only).
    pub fn new(max_delay_samples: usize, delay_samples: usize, feedback: f32, mix: f32) -> Self {
        let cap = max_delay_samples.max(1);
        Self {
            buf: vec![0.0; cap],
            idx: 0,
            delay: delay_samples.min(cap - 1).max(1),
            feedback: feedback.clamp(0.0, 0.999),
            mix: mix.clamp(0.0, 1.0),
        }
    }

    /// Process one sample, advancing state.
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let cap = self.buf.len();
        let read = (self.idx + cap - self.delay) % cap;
        let d = self.buf[read];
        let y = x * (1.0 - self.mix) + d * self.mix;
        self.buf[self.idx] = x + d * self.feedback;
        self.idx = (self.idx + 1) % cap;
        y
    }

    /// Process a buffer in place.
    pub fn process(&mut self, buf: &mut [f32]) {
        for x in buf {
            *x = self.process_sample(*x);
        }
    }

    /// Clear the delay line.
    pub fn reset(&mut self) {
        self.buf.iter_mut().for_each(|s| *s = 0.0);
        self.idx = 0;
    }
}

/// A peak limiter: holds output below `threshold` by reducing gain when the
/// signal's envelope exceeds it, recovering at `release`.
#[derive(Debug, Clone)]
pub struct Limiter {
    threshold: f32,
    release: f32,
    env: f32,
}

impl Limiter {
    /// A limiter with the given linear `threshold` (e.g. 0.5) and `release`
    /// coefficient (0..1, closer to 1.0 recovers more slowly).
    pub fn new(threshold: f32, release: f32) -> Self {
        Self {
            threshold: threshold.max(1.0e-6),
            release: release.clamp(0.0, 0.99999),
            env: 0.0,
        }
    }

    /// Process one sample, advancing the envelope.
    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        self.env = x.abs().max(self.env * self.release);
        let gain = if self.env > self.threshold {
            self.threshold / self.env
        } else {
            1.0
        };
        x * gain
    }

    /// Process a buffer in place.
    pub fn process(&mut self, buf: &mut [f32]) {
        for x in buf {
            *x = self.process_sample(*x);
        }
    }

    /// Clear state.
    pub fn reset(&mut self) {
        self.env = 0.0;
    }
}

/// Stateless soft clip: `y = threshold * tanh(x / threshold)`. Smoothly
/// saturates toward `±threshold` instead of hard-clipping.
pub fn soft_clip(samples: &mut [f32], threshold: f32) {
    let t = threshold.max(1.0e-6);
    for s in samples {
        *s = t * (*s / t).tanh();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn dc_blocker_removes_dc_passes_tone() {
        let mut b = DcBlocker::default();
        let mut dc = vec![1.0f32; 4_000];
        b.process(&mut dc);
        assert!(dc[dc.len() - 1].abs() < 1e-2);

        let mut b = DcBlocker::default();
        let mut tone: Vec<f32> =
            (0..4_000).map(|n| (2.0 * PI * 1_000.0 * n as f32 / 48_000.0).sin()).collect();
        b.process(&mut tone);
        let peak = tone[1_000..].iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!((peak - 1.0).abs() < 0.05, "tone peak {peak}");
    }

    #[test]
    fn delay_impulse_echoes() {
        let mut d = Delay::new(16, 3, 0.5, 0.5);
        let mut buf = vec![0.0f32; 13];
        buf[0] = 1.0;
        d.process(&mut buf);
        assert!((buf[0] - 0.5).abs() < 1e-6); // dry half
        assert!((buf[3] - 0.5).abs() < 1e-6); // first echo
        assert!((buf[6] - 0.25).abs() < 1e-6); // decayed
        assert!((buf[9] - 0.125).abs() < 1e-6);
    }

    #[test]
    fn limiter_caps_above_threshold_passes_below() {
        let mut l = Limiter::new(0.5, 0.999);
        let mut loud = vec![0.9f32; 2_000];
        l.process(&mut loud);
        assert!((loud[loud.len() - 1] - 0.5).abs() < 1e-3);

        let mut l = Limiter::new(0.5, 0.999);
        let mut quiet = vec![0.2f32; 2_000];
        l.process(&mut quiet);
        assert!((quiet[quiet.len() - 1] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn soft_clip_is_linear_small_saturates_large() {
        let mut s = [0.1, 5.0, -5.0];
        soft_clip(&mut s, 1.0);
        assert!((s[0] - 0.0997).abs() < 1e-3);
        assert!(s[1] < 1.0 && s[1] > 0.99);
        assert!(s[2] > -1.0 && s[2] < -0.99);
    }
}
