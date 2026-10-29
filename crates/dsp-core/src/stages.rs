//! Pipeline stages that apply DSP to audio frames — the Kryx integration.
//!
//! Each stage implements `kc_core::pipeline::Stage`, so it drops straight into a
//! `Pipeline` or `StreamPipeline`. It interprets a `MediaBuffer`'s bytes as
//! little-endian `f32` samples (that interpretation is the DSP layer's job —
//! core stays byte-agnostic, Rule 2), processes them, and emits a new frame
//! carrying the same presentation timestamp.

use async_trait::async_trait;
use std::sync::Mutex;

use kc_core::buffer::MediaBuffer;
use kc_core::error::Result;
use kc_core::pipeline::{Stage, StageContext};
use kc_core::types::MediaType;

use crate::biquad::Biquad;
use crate::resample::LinearResampler;
use crate::{bytes_to_f32, f32_to_bytes, samples};

/// Build an audio `MediaBuffer` from samples, preserving `src`'s timestamp.
fn audio_frame_like(src: &MediaBuffer, out_samples: &[f32]) -> Result<MediaBuffer> {
    MediaBuffer::builder(MediaType::Audio)
        .data(f32_to_bytes(out_samples))
        .pts(src.pts())
        .build()
}

/// A stage that applies a linear gain (volume) to every sample.
#[derive(Debug)]
pub struct GainStage {
    /// Linear gain factor (1.0 = unchanged, 0.5 = −6 dB, 2.0 = +6 dB).
    pub factor: f32,
}

impl GainStage {
    /// A gain stage with the given linear factor.
    pub fn new(factor: f32) -> Self {
        Self { factor }
    }
}

#[async_trait]
impl Stage for GainStage {
    fn name(&self) -> &'static str {
        "dsp.gain"
    }

    async fn process(&self, buf: MediaBuffer, _ctx: &StageContext) -> Result<Vec<MediaBuffer>> {
        let mut s = bytes_to_f32(buf.data());
        samples::gain(&mut s, self.factor);
        Ok(vec![audio_frame_like(&buf, &s)?])
    }
}

/// A stage that runs a biquad filter across the stream, keeping filter state
/// across frames (so filtering a frame sequence is seamless).
#[derive(Debug)]
pub struct BiquadStage {
    filter: Mutex<Biquad>,
}

impl BiquadStage {
    /// A low-pass biquad stage.
    pub fn low_pass(sample_rate: f32, cutoff_hz: f32, q: f32) -> Self {
        Self {
            filter: Mutex::new(Biquad::low_pass(sample_rate, cutoff_hz, q)),
        }
    }

    /// A high-pass biquad stage.
    pub fn high_pass(sample_rate: f32, cutoff_hz: f32, q: f32) -> Self {
        Self {
            filter: Mutex::new(Biquad::high_pass(sample_rate, cutoff_hz, q)),
        }
    }

    /// A band-pass biquad stage.
    pub fn band_pass(sample_rate: f32, center_hz: f32, q: f32) -> Self {
        Self {
            filter: Mutex::new(Biquad::band_pass(sample_rate, center_hz, q)),
        }
    }

    /// A notch (band-reject) biquad stage.
    pub fn notch(sample_rate: f32, center_hz: f32, q: f32) -> Self {
        Self {
            filter: Mutex::new(Biquad::notch(sample_rate, center_hz, q)),
        }
    }

    /// A peaking-EQ biquad stage (`gain_db` boost/cut around `center_hz`).
    pub fn peaking(sample_rate: f32, center_hz: f32, q: f32, gain_db: f32) -> Self {
        Self {
            filter: Mutex::new(Biquad::peaking(sample_rate, center_hz, q, gain_db)),
        }
    }

    /// A low-shelf biquad stage.
    pub fn low_shelf(sample_rate: f32, corner_hz: f32, q: f32, gain_db: f32) -> Self {
        Self {
            filter: Mutex::new(Biquad::low_shelf(sample_rate, corner_hz, q, gain_db)),
        }
    }

    /// A high-shelf biquad stage.
    pub fn high_shelf(sample_rate: f32, corner_hz: f32, q: f32, gain_db: f32) -> Self {
        Self {
            filter: Mutex::new(Biquad::high_shelf(sample_rate, corner_hz, q, gain_db)),
        }
    }
}

#[async_trait]
impl Stage for BiquadStage {
    fn name(&self) -> &'static str {
        "dsp.filter"
    }

    async fn process(&self, buf: MediaBuffer, _ctx: &StageContext) -> Result<Vec<MediaBuffer>> {
        let mut s = bytes_to_f32(buf.data());
        // Lock is held only for the CPU filtering, never across an await.
        self.filter.lock().expect("biquad mutex").process(&mut s);
        Ok(vec![audio_frame_like(&buf, &s)?])
    }
}

/// A stage that resamples the stream from one sample rate to another, keeping
/// resampler state across frames so the seams stay click-free.
#[derive(Debug)]
pub struct ResampleStage {
    resampler: Mutex<LinearResampler>,
}

impl ResampleStage {
    /// A resample stage from `from_hz` to `to_hz`.
    pub fn new(from_hz: f32, to_hz: f32) -> Self {
        Self {
            resampler: Mutex::new(LinearResampler::new(from_hz, to_hz)),
        }
    }
}

#[async_trait]
impl Stage for ResampleStage {
    fn name(&self) -> &'static str {
        "dsp.resample"
    }

    async fn process(&self, buf: MediaBuffer, _ctx: &StageContext) -> Result<Vec<MediaBuffer>> {
        let s = bytes_to_f32(buf.data());
        // Lock held only for the CPU resampling, never across an await.
        let out = self.resampler.lock().expect("resampler mutex").process(&s);
        // A chunk can legitimately yield zero output (fractional boundary); emit
        // nothing rather than an empty frame.
        if out.is_empty() {
            return Ok(vec![]);
        }
        Ok(vec![audio_frame_like(&buf, &out)?])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kc_core::pipeline::Pipeline;

    fn audio(samples: &[f32]) -> MediaBuffer {
        MediaBuffer::builder(MediaType::Audio)
            .data(f32_to_bytes(samples))
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn gain_stage_doubles_samples_through_a_pipeline() {
        // Runs the stage through a real core Pipeline — the integration that
        // proves a DSP stage drops into Kryx unchanged.
        let p = Pipeline::builder().stage(GainStage::new(2.0)).build().unwrap();
        let out = p.process(audio(&[0.1, -0.2, 0.3])).await.unwrap();
        assert_eq!(out.len(), 1);
        let got = bytes_to_f32(out[0].data());
        assert!((got[0] - 0.2).abs() < 1e-6);
        assert!((got[1] + 0.4).abs() < 1e-6);
        assert!((got[2] - 0.6).abs() < 1e-6);
    }

    #[tokio::test]
    async fn filter_stage_keeps_state_across_frames() {
        let p = Pipeline::builder()
            .stage(BiquadStage::low_pass(48_000.0, 1_000.0, 0.707))
            .build()
            .unwrap();
        // Two DC frames; by the second the filter has settled near unity.
        let _ = p.process(audio(&vec![1.0; 1_000])).await.unwrap();
        let out = p.process(audio(&vec![1.0; 1_000])).await.unwrap();
        let got = bytes_to_f32(out[0].data());
        assert!((got[got.len() - 1] - 1.0).abs() < 1e-2);
    }

    #[tokio::test]
    async fn band_pass_stage_runs_through_a_pipeline() {
        let p = Pipeline::builder()
            .stage(BiquadStage::band_pass(48_000.0, 1_000.0, 1.0))
            .build()
            .unwrap();
        let out = p.process(audio(&vec![1.0; 1_000])).await.unwrap();
        // Band-pass blocks DC → settles toward zero.
        let got = bytes_to_f32(out[0].data());
        assert!(got[got.len() - 1].abs() < 0.1);
    }

    #[tokio::test]
    async fn resample_stage_halves_a_downsampled_frame() {
        let p = Pipeline::builder()
            .stage(ResampleStage::new(2_000.0, 1_000.0))
            .build()
            .unwrap();
        let out = p.process(audio(&(0..1_000).map(|n| n as f32).collect::<Vec<_>>()))
            .await
            .unwrap();
        assert_eq!(out.len(), 1);
        let got = bytes_to_f32(out[0].data());
        assert!((got.len() as i64 - 500).abs() < 4, "got {}", got.len());
    }
}
