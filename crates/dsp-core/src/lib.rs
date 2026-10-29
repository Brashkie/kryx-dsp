//! # kryx-dsp — audio DSP processing for the Kryx ecosystem
//!
//! The Processing layer of Kryx: sample-level audio math exposed as pipeline
//! stages that plug into `@kryxjs/core`. It is the first real consumer of the
//! core's `Stage` / `Pipeline` / `StageRegistry` surface — building it validates
//! that API against a genuine workload.
//!
//! Three parts, cleanly separated:
//!
//! - [`samples`] — pure, dependency-free math on `f32` buffers (gain, mix,
//!   clamp, peak, rms). Knows nothing about Kryx.
//! - [`biquad`] — a second-order IIR filter (RBJ cookbook coefficients).
//! - [`stages`] — [`GainStage`](stages::GainStage) and
//!   [`BiquadStage`](stages::BiquadStage), which implement the core `Stage`
//!   trait, plus [`register_dsp`] to make them available by name in a
//!   `StageRegistry`.
//!
//! Audio frames are interpreted as **little-endian `f32` samples**. That
//! interpretation is the DSP layer's responsibility; core stays byte-agnostic.
//!
//! ```no_run
//! use kc_core::pipeline::{StageRegistry, StageConfig};
//! use kryx_dsp::register_dsp;
//!
//! let mut reg = StageRegistry::new();
//! register_dsp(&mut reg);
//!
//! // A low-pass → gain chain, described by config.
//! let pipeline = reg.build_pipeline(&[
//!     ("dsp.filter", StageConfig::new().set("cutoff", 3_000.0)),
//!     ("dsp.gain", StageConfig::new().set("factor", 0.8)),
//! ]).unwrap();
//! ```

#![forbid(unsafe_code)]

pub mod biquad;
pub mod registry;
pub mod resample;
pub mod samples;
pub mod stages;

pub use biquad::Biquad;
pub use registry::register_dsp;
pub use resample::LinearResampler;
pub use stages::{BiquadStage, GainStage, ResampleStage};

/// Version of this crate, mirrored from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// ─── Byte ↔ sample helpers ───────────────────────────────────────────────────

/// Read a byte slice as little-endian `f32` samples.
///
/// Alignment-safe (reads four bytes at a time), so it works on any `MediaBuffer`
/// payload. Trailing bytes that don't complete a sample are ignored.
pub fn bytes_to_f32(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Write `f32` samples as little-endian bytes.
pub fn f32_to_bytes(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 4);
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_sample_roundtrip() {
        let s = [0.5f32, -0.25, 1.0, -1.0];
        let bytes = f32_to_bytes(&s);
        assert_eq!(bytes.len(), 16);
        assert_eq!(bytes_to_f32(&bytes), s);
    }

    #[test]
    fn trailing_partial_sample_ignored() {
        // 6 bytes = one f32 + 2 leftover → one sample.
        let bytes = [0u8, 0, 128, 63, 1, 2]; // 1.0f32 then junk
        assert_eq!(bytes_to_f32(&bytes), vec![1.0]);
    }

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
