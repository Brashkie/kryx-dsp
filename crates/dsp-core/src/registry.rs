//! Register the DSP stages into a core `StageRegistry`, so pipelines can be
//! assembled from configuration (names + params) — the Phase 6 plugin surface.

use std::sync::Arc;

use kc_core::pipeline::{StageConfig, StageRegistry};

use crate::stages::{BiquadStage, GainStage, ResampleStage};

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
        Ok(Arc::new(ResampleStage::new(from, to)) as _)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_dsp_stages() {
        let mut reg = StageRegistry::new();
        register_dsp(&mut reg);
        assert!(reg.contains("dsp.gain"));
        assert!(reg.contains("dsp.filter"));
        assert!(reg.contains("dsp.resample"));
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
