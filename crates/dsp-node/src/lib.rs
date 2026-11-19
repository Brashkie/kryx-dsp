//! N-API bindings for @kryxjs/dsp.
//!
//! A thin, coarse-grained bridge over the pure-Rust `kryx-dsp` core: sample math
//! and the biquad filter, exposed to JavaScript working with `Float32Array`.
//! JS↔native calls stay coarse (whole buffers at a time), so the crossing cost
//! never outweighs the work done — one of the Kryx layer-5 principles.
//!
//! The Stage / Pipeline / StageRegistry integration lives in the core crate and
//! is used to compose pipelines in Rust; it is not surfaced here.

use napi::bindgen_prelude::Float32Array;
use napi_derive::napi;

use kryx_dsp::effects::{self, DcBlocker as CoreDcBlocker, Delay as CoreDelay, Limiter as CoreLimiter};
use kryx_dsp::{
    samples, Biquad as CoreBiquad, Equalizer as CoreEq, LinearResampler, SincResampler as CoreSinc,
};

/// Scale every sample by `factor` (linear gain). Returns a new buffer.
#[napi]
pub fn gain(input: Float32Array, factor: f64) -> Float32Array {
    let mut v = input.as_ref().to_vec();
    samples::gain(&mut v, factor as f32);
    Float32Array::new(v)
}

/// Mix `src` into `dst` sample-wise over the overlap. Returns a new buffer.
#[napi]
pub fn mix(dst: Float32Array, src: Float32Array) -> Float32Array {
    let mut v = dst.as_ref().to_vec();
    samples::mix_into(&mut v, src.as_ref());
    Float32Array::new(v)
}

/// Clamp every sample to the normalized full-scale range `[-1.0, 1.0]`.
#[napi]
pub fn clamp(input: Float32Array) -> Float32Array {
    let mut v = input.as_ref().to_vec();
    samples::clamp(&mut v);
    Float32Array::new(v)
}

/// Peak absolute amplitude across the buffer.
#[napi]
pub fn peak(input: Float32Array) -> f64 {
    samples::peak(input.as_ref()) as f64
}

/// Root-mean-square level across the buffer.
#[napi]
pub fn rms(input: Float32Array) -> f64 {
    samples::rms(input.as_ref()) as f64
}

/// Version of the native `kryx-dsp` core crate.
#[napi]
pub fn core_version() -> String {
    kryx_dsp::VERSION.to_string()
}

/// Convert decibels to a linear amplitude factor.
#[napi]
pub fn db_to_linear(db: f64) -> f64 {
    samples::db_to_linear(db as f32) as f64
}

/// Convert a linear amplitude factor to decibels.
#[napi]
pub fn linear_to_db(linear: f64) -> f64 {
    samples::linear_to_db(linear as f32) as f64
}

/// Apply a gain expressed in decibels to every sample. Returns a new buffer.
#[napi]
pub fn gain_db(input: Float32Array, db: f64) -> Float32Array {
    let mut v = input.as_ref().to_vec();
    samples::gain_db(&mut v, db as f32);
    Float32Array::new(v)
}

/// Soft-clip (saturate) toward `±threshold`. Returns a new buffer.
#[napi]
pub fn soft_clip(input: Float32Array, threshold: f64) -> Float32Array {
    let mut v = input.as_ref().to_vec();
    effects::soft_clip(&mut v, threshold as f32);
    Float32Array::new(v)
}

/// A second-order IIR (biquad) filter with persistent state across `process`
/// calls, so a stream can be filtered buffer by buffer.
#[napi(js_name = "Biquad")]
pub struct Biquad {
    inner: CoreBiquad,
}

#[napi]
impl Biquad {
    /// A low-pass filter at `cutoff_hz` with resonance `q` (0.707 ≈ Butterworth).
    #[napi(factory)]
    pub fn low_pass(sample_rate: f64, cutoff_hz: f64, q: f64) -> Self {
        Self {
            inner: CoreBiquad::low_pass(sample_rate as f32, cutoff_hz as f32, q as f32),
        }
    }

    /// A high-pass filter at `cutoff_hz` with resonance `q`.
    #[napi(factory)]
    pub fn high_pass(sample_rate: f64, cutoff_hz: f64, q: f64) -> Self {
        Self {
            inner: CoreBiquad::high_pass(sample_rate as f32, cutoff_hz as f32, q as f32),
        }
    }

    /// A band-pass filter centered at `center_hz` (constant 0 dB peak gain).
    #[napi(factory)]
    pub fn band_pass(sample_rate: f64, center_hz: f64, q: f64) -> Self {
        Self {
            inner: CoreBiquad::band_pass(sample_rate as f32, center_hz as f32, q as f32),
        }
    }

    /// A notch (band-reject) filter that removes `center_hz`.
    #[napi(factory)]
    pub fn notch(sample_rate: f64, center_hz: f64, q: f64) -> Self {
        Self {
            inner: CoreBiquad::notch(sample_rate as f32, center_hz as f32, q as f32),
        }
    }

    /// A peaking EQ: boost or cut `gain_db` around `center_hz`.
    #[napi(factory)]
    pub fn peaking(sample_rate: f64, center_hz: f64, q: f64, gain_db: f64) -> Self {
        Self {
            inner: CoreBiquad::peaking(
                sample_rate as f32,
                center_hz as f32,
                q as f32,
                gain_db as f32,
            ),
        }
    }

    /// A low-shelf: boost or cut `gain_db` below `corner_hz`.
    #[napi(factory)]
    pub fn low_shelf(sample_rate: f64, corner_hz: f64, q: f64, gain_db: f64) -> Self {
        Self {
            inner: CoreBiquad::low_shelf(
                sample_rate as f32,
                corner_hz as f32,
                q as f32,
                gain_db as f32,
            ),
        }
    }

    /// A high-shelf: boost or cut `gain_db` above `corner_hz`.
    #[napi(factory)]
    pub fn high_shelf(sample_rate: f64, corner_hz: f64, q: f64, gain_db: f64) -> Self {
        Self {
            inner: CoreBiquad::high_shelf(
                sample_rate as f32,
                corner_hz as f32,
                q as f32,
                gain_db as f32,
            ),
        }
    }

    /// An all-pass filter (unity magnitude, phase shift only).
    #[napi(factory)]
    pub fn all_pass(sample_rate: f64, center_hz: f64, q: f64) -> Self {
        Self {
            inner: CoreBiquad::all_pass(sample_rate as f32, center_hz as f32, q as f32),
        }
    }

    /// Filter a buffer, advancing the filter state. Returns a new buffer.
    #[napi]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        let mut v = input.as_ref().to_vec();
        self.inner.process(&mut v);
        Float32Array::new(v)
    }

    /// Clear the filter state (as if starting a fresh stream).
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// A parametric equalizer — a chain of biquad bands applied in series.
#[napi(js_name = "Equalizer")]
pub struct Equalizer {
    inner: CoreEq,
}

#[napi]
impl Equalizer {
    /// An empty equalizer (a pass-through until bands are added).
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { inner: CoreEq::new() }
    }

    /// Number of bands.
    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.inner.len() as u32
    }

    /// Append a low-pass band.
    #[napi]
    pub fn add_low_pass(&mut self, sample_rate: f64, cutoff_hz: f64, q: f64) {
        self.inner.push(CoreBiquad::low_pass(sample_rate as f32, cutoff_hz as f32, q as f32));
    }

    /// Append a high-pass band.
    #[napi]
    pub fn add_high_pass(&mut self, sample_rate: f64, cutoff_hz: f64, q: f64) {
        self.inner.push(CoreBiquad::high_pass(sample_rate as f32, cutoff_hz as f32, q as f32));
    }

    /// Append a band-pass band.
    #[napi]
    pub fn add_band_pass(&mut self, sample_rate: f64, center_hz: f64, q: f64) {
        self.inner.push(CoreBiquad::band_pass(sample_rate as f32, center_hz as f32, q as f32));
    }

    /// Append a notch band.
    #[napi]
    pub fn add_notch(&mut self, sample_rate: f64, center_hz: f64, q: f64) {
        self.inner.push(CoreBiquad::notch(sample_rate as f32, center_hz as f32, q as f32));
    }

    /// Append an all-pass band.
    #[napi]
    pub fn add_all_pass(&mut self, sample_rate: f64, center_hz: f64, q: f64) {
        self.inner.push(CoreBiquad::all_pass(sample_rate as f32, center_hz as f32, q as f32));
    }

    /// Append a peaking band (`gainDb` boost/cut around `centerHz`).
    #[napi]
    pub fn add_peaking(&mut self, sample_rate: f64, center_hz: f64, q: f64, gain_db: f64) {
        self.inner.push(CoreBiquad::peaking(
            sample_rate as f32,
            center_hz as f32,
            q as f32,
            gain_db as f32,
        ));
    }

    /// Append a low-shelf band.
    #[napi]
    pub fn add_low_shelf(&mut self, sample_rate: f64, corner_hz: f64, q: f64, gain_db: f64) {
        self.inner.push(CoreBiquad::low_shelf(
            sample_rate as f32,
            corner_hz as f32,
            q as f32,
            gain_db as f32,
        ));
    }

    /// Append a high-shelf band.
    #[napi]
    pub fn add_high_shelf(&mut self, sample_rate: f64, corner_hz: f64, q: f64, gain_db: f64) {
        self.inner.push(CoreBiquad::high_shelf(
            sample_rate as f32,
            corner_hz as f32,
            q as f32,
            gain_db as f32,
        ));
    }

    /// Filter a buffer through every band in series. Returns a new buffer.
    #[napi]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        let mut v = input.as_ref().to_vec();
        self.inner.process(&mut v);
        Float32Array::new(v)
    }

    /// Clear the state of every band.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// A streaming linear resampler. State carries across `process` calls, so a
/// stream can be resampled buffer by buffer with click-free seams.
#[napi(js_name = "Resampler")]
pub struct Resampler {
    inner: LinearResampler,
}

#[napi]
impl Resampler {
    /// A resampler from `from_hz` to `to_hz`.
    #[napi(constructor)]
    pub fn new(from_hz: f64, to_hz: f64) -> Self {
        Self {
            inner: LinearResampler::new(from_hz as f32, to_hz as f32),
        }
    }

    /// Resample a buffer, returning the samples produced from this chunk. A
    /// chunk may legitimately yield an empty buffer at a fractional boundary.
    #[napi]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        Float32Array::new(self.inner.process(input.as_ref()))
    }

    /// Clear state (as if starting a fresh stream); keeps the rate ratio.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// A high-quality anti-aliased (windowed-sinc) streaming resampler.
#[napi(js_name = "SincResampler")]
pub struct SincResampler {
    inner: CoreSinc,
}

#[napi]
impl SincResampler {
    /// A resampler from `from_hz` to `to_hz`. `quality` = sinc zero-crossings per
    /// side (16 is a good default; higher is sharper and costlier).
    #[napi(constructor)]
    pub fn new(from_hz: f64, to_hz: f64, quality: Option<u32>) -> Self {
        Self {
            inner: CoreSinc::new(from_hz as f32, to_hz as f32, quality.unwrap_or(16)),
        }
    }

    /// Resample a buffer, returning the samples produced from this chunk.
    #[napi]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        Float32Array::new(self.inner.process(input.as_ref()))
    }

    /// Clear state (fresh stream); keeps the rate ratio and kernel.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// A DC blocker — removes the 0 Hz offset, passes everything else.
#[napi(js_name = "DcBlocker")]
pub struct DcBlocker {
    inner: CoreDcBlocker,
}

#[napi]
impl DcBlocker {
    /// A DC blocker with pole `r` (default 0.995).
    #[napi(constructor)]
    pub fn new(r: Option<f64>) -> Self {
        Self {
            inner: CoreDcBlocker::new(r.unwrap_or(0.995) as f32),
        }
    }

    /// Process a buffer. Returns a new buffer.
    #[napi]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        let mut v = input.as_ref().to_vec();
        self.inner.process(&mut v);
        Float32Array::new(v)
    }

    /// Clear state.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// A delay / echo with feedback and dry/wet mix.
#[napi(js_name = "Delay")]
pub struct Delay {
    inner: CoreDelay,
}

#[napi]
impl Delay {
    /// A delay of `delaySamples` (capped at `maxDelaySamples`), with `feedback`
    /// (0..1) and `mix` (0 = dry, 1 = wet).
    #[napi(constructor)]
    pub fn new(max_delay_samples: u32, delay_samples: u32, feedback: f64, mix: f64) -> Self {
        Self {
            inner: CoreDelay::new(
                max_delay_samples as usize,
                delay_samples as usize,
                feedback as f32,
                mix as f32,
            ),
        }
    }

    /// Process a buffer. Returns a new buffer.
    #[napi]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        let mut v = input.as_ref().to_vec();
        self.inner.process(&mut v);
        Float32Array::new(v)
    }

    /// Clear the delay line.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

/// A peak limiter — holds output below `threshold`.
#[napi(js_name = "Limiter")]
pub struct Limiter {
    inner: CoreLimiter,
}

#[napi]
impl Limiter {
    /// A limiter with linear `threshold` and `release` (0..1).
    #[napi(constructor)]
    pub fn new(threshold: f64, release: f64) -> Self {
        Self {
            inner: CoreLimiter::new(threshold as f32, release as f32),
        }
    }

    /// Process a buffer. Returns a new buffer.
    #[napi]
    pub fn process(&mut self, input: Float32Array) -> Float32Array {
        let mut v = input.as_ref().to_vec();
        self.inner.process(&mut v);
        Float32Array::new(v)
    }

    /// Clear state.
    #[napi]
    pub fn reset(&mut self) {
        self.inner.reset();
    }
}
