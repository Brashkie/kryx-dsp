//! Register the DSP stages into a core `StageRegistry`, so pipelines can be
//! assembled from configuration (names + params) — the Phase 6 plugin surface.

use std::sync::Arc;

use kc_core::pipeline::{StageConfig, StageRegistry};

use crate::stages::{
    BiquadStage, DcBlockerStage, DelayStage, GainStage, LimiterStage, ResampleStage, SoftClipStage,
};

/// Register every DSP stage into `reg` under its `dsp.*` name.
///
/// After this, a caller can build these from config without knowing the types:
///
/// - `dsp.gain` — params: `factor` (float, default 1.0)
/// - `dsp.filter` — params: `kind` (`low_pass` | `high_pass` | `band_pass` |
///   `notch` | `peaking` | `low_shelf` | `high_shelf`, default low_pass),
///   `sample_rate` (float, default 48000), `cutoff` (float, default 1000),
///   `q` (float, default 0.707), `gain_db` (float, default 0.0 — peaking/shelf)
/// - `dsp.resample` — params: `from` (float, default 48000), `to` (float,
///   default 48000)
pub fn register_dsp(reg: &mut StageRegistry) {
    reg.register("dsp.gain", |cfg: &StageConfig| {
        let factor = cfg.float("factor").unwrap_or(1.0) as f32;
        Ok(Arc::new(GainStage::new(factor)) as _)
    });

    reg.register("dsp.filter", |cfg: &StageConfig| {
        let sr = cfg.float("sample_rate").unwrap_or(48_000.0) as f32;
        let cutoff = cfg.float("cutoff").unwrap_or(1_000.0) as f32;
        let q = cfg.float("q").unwrap_or(0.707) as f32;
        let gain_db = cfg.float("gain_db").unwrap_or(0.0) as f32;
        let stage = match cfg.str("kind").unwrap_or("low_pass") {
            "high_pass" => BiquadStage::high_pass(sr, cutoff, q),
            "band_pass" => BiquadStage::band_pass(sr, cutoff, q),
            "notch" => BiquadStage::notch(sr, cutoff, q),
            "peaking" => BiquadStage::peaking(sr, cutoff, q, gain_db),
            "low_shelf" => BiquadStage::low_shelf(sr, cutoff, q, gain_db),
            "high_shelf" => BiquadStage::high_shelf(sr, cutoff, q, gain_db),
            _ => BiquadStage::low_pass(sr, cutoff, q),
        };
        Ok(Arc::new(stage) as _)
    });

    reg.register("dsp.resample", |cfg: &StageConfig| {
        let from = cfg.float("from").unwrap_or(48_000.0) as f32;
        let to = cfg.float("to").unwrap_or(48_000.0) as f32;
        let quality = cfg.int("quality").unwrap_or(16).max(2) as u32;
        let stage = match cfg.str("kind").unwrap_or("linear") {
            "sinc" => ResampleStage::sinc(from, to, quality),
            _ => ResampleStage::linear(from, to),
        };
        Ok(Arc::new(stage) as _)
    });

    reg.register("dsp.dcblock", |cfg: &StageConfig| {
        let r = cfg.float("r").unwrap_or(0.995) as f32;
        Ok(Arc::new(DcBlockerStage::new(r)) as _)
    });

    reg.register("dsp.delay", |cfg: &StageConfig| {
        let max = cfg.int("max_delay").unwrap_or(48_000) as usize;
        let delay = cfg.int("delay").unwrap_or(2_400) as usize;
        let feedback = cfg.float("feedback").unwrap_or(0.3) as f32;
        let mix = cfg.float("mix").unwrap_or(0.5) as f32;
        Ok(Arc::new(DelayStage::new(max, delay, feedback, mix)) as _)
    });

    reg.register("dsp.limiter", |cfg: &StageConfig| {
        let threshold = cfg.float("threshold").unwrap_or(0.9) as f32;
        let release = cfg.float("release").unwrap_or(0.999) as f32;
        Ok(Arc::new(LimiterStage::new(threshold, release)) as _)
    });

    reg.register("dsp.softclip", |cfg: &StageConfig| {
        let threshold = cfg.float("threshold").unwrap_or(1.0) as f32;
        Ok(Arc::new(SoftClipStage::new(threshold)) as _)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_dsp_stages() {
        let mut reg = StageRegistry::new();
        register_dsp(&mut reg);
        for name in [
            "dsp.gain",
            "dsp.filter",
            "dsp.resample",
            "dsp.dcblock",
            "dsp.delay",
            "dsp.limiter",
            "dsp.softclip",
        ] {
            assert!(reg.contains(name), "missing {name}");
        }
    }

    #[test]
    fn builds_sinc_resample_and_effects() {
        let mut reg = StageRegistry::new();
        register_dsp(&mut reg);
        let sinc = reg
            .build(
                "dsp.resample",
                &StageConfig::new().set("kind", "sinc").set("quality", 16),
            )
            .unwrap();
        assert_eq!(sinc.name(), "dsp.resample");
        assert_eq!(reg.build("dsp.dcblock", &StageConfig::new()).unwrap().name(), "dsp.dcblock");
        assert_eq!(reg.build("dsp.delay", &StageConfig::new()).unwrap().name(), "dsp.delay");
        assert_eq!(reg.build("dsp.limiter", &StageConfig::new()).unwrap().name(), "dsp.limiter");
        assert_eq!(reg.build("dsp.softclip", &StageConfig::new()).unwrap().name(), "dsp.softclip");
    }

    #[test]
    fn builds_each_filter_kind() {
        let mut reg = StageRegistry::new();
        register_dsp(&mut reg);
        for kind in ["low_pass", "high_pass", "band_pass", "notch", "peaking", "low_shelf", "high_shelf"] {
            let cfg = StageConfig::new().set("kind", kind).set("gain_db", 3.0);
            let stage = reg.build("dsp.filter", &cfg).unwrap();
            assert_eq!(stage.name(), "dsp.filter");
        }
    }

    #[test]
    fn builds_a_resample_stage() {
        let mut reg = StageRegistry::new();
        register_dsp(&mut reg);
        let cfg = StageConfig::new().set("from", 48_000.0).set("to", 44_100.0);
        let stage = reg.build("dsp.resample", &cfg).unwrap();
        assert_eq!(stage.name(), "dsp.resample");
    }

    #[test]
    fn builds_a_dsp_stage_from_config() {
        let mut reg = StageRegistry::new();
        register_dsp(&mut reg);
        let cfg = StageConfig::new().set("factor", 0.5f64);
        let stage = reg.build("dsp.gain", &cfg).unwrap();
        assert_eq!(stage.name(), "dsp.gain");
    }

    #[test]
    fn assembles_a_pipeline_from_config() {
        let mut reg = StageRegistry::new();
        register_dsp(&mut reg);
        let p = reg
            .build_pipeline(&[
                ("dsp.filter", StageConfig::new().set("kind", "low_pass").set("cutoff", 2_000.0)),
                ("dsp.gain", StageConfig::new().set("factor", 0.8)),
            ])
            .unwrap();
        assert_eq!(p.stage_names(), vec!["dsp.filter", "dsp.gain"]);
    }
}
