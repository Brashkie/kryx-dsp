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

use kryx_dsp::{samples, Biquad as CoreBiquad, LinearResampler};

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
