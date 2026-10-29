# Changelog

## [0.1.0] — 2026-10-29

First stable release of the Kryx Processing layer. Consolidates M1–M3 and makes
the package github-ready.

### Summary
- Sample math: `gain`, `mix`, `clamp`, `peak`, `rms`.
- Biquad filters (RBJ): low/high-pass, band-pass, notch, peaking, low/high-shelf,
  with state that carries across frames.
- Streaming linear resampling (`Resampler`) with click-free buffer seams.
- Pipeline stages for `@kryxjs/core` (`GainStage`, `BiquadStage`, `ResampleStage`)
  and `register_dsp` to build them from config via `StageRegistry`.
- TypeScript SDK over N-API; pure-Rust core, zero runtime deps.

### Added (0.1.0 release engineering)
- Apache-2.0 `LICENSE` + `NOTICE`, bilingual `README.md` / `README.es.md` with
  badges, `ROADMAP.md`, and community files (CONTRIBUTING, CODE_OF_CONDUCT,
  SECURITY, issue/PR templates, dependabot).
- Dual CJS + ESM package (`.cjs` / `.mjs` / `.d.cts` / `.d.mts`), Biome
  (lint + format), `rust-toolchain.toml`, `.gitattributes`, `.npmignore`.
- CI (`ci.yml`) and release (`release.yml`) workflows, per-platform npm
  sub-packages for 7 targets via the `preartifacts` hook, `scripts/` helpers
  (`create-npm-dirs`, `fix-dts`, `sync-versions`).
- Configuration mirrors the `@kryxjs/core` (kryx-core) molde so the whole Kryx
  ecosystem stays consistent.

## [0.1.0-alpha.1] — 2026-10-28 (M3)

Resampling and a full biquad filter set.

### Added
- `resample::LinearResampler` — streaming linear sample-rate conversion with
  fractional-position state, so buffers join without clicks. `process`, `reset`,
  `ratio`.
- `biquad::Biquad` gains `band_pass`, `notch`, `peaking`, `low_shelf`,
  `high_shelf` (RBJ cookbook), alongside the existing low/high-pass.
- `stages::ResampleStage` (`dsp.resample`) and matching `BiquadStage`
  constructors for every filter kind.
- `registry`: `dsp.resample` (`from` / `to`), and `dsp.filter` now accepts
  `kind` = low_pass | high_pass | band_pass | notch | peaking | low_shelf |
  high_shelf, plus a `gain_db` param.
- N-API + SDK: `Resampler` class and the new `Biquad` factories
  (`bandPass`, `notch`, `peaking`, `lowShelf`, `highShelf`).
- Tests at every layer (Rust core, SDK unit with mocks, SDK integration over the
  real `.node`), including resampler seam-continuity.

## [0.1.0-alpha.0] — 2026-10-28 (M2)

N-API bindings and the TypeScript SDK — the package becomes usable from Node.

### Added
- Workspace layout: `crates/dsp-core` (pure Rust) + `crates/dsp-node` (N-API cdylib).
- Native bindings for the sample math (`gain`, `mix`, `clamp`, `peak`, `rms`) and
  the `Biquad` filter (`lowPass` / `highPass` factories, `process`, `reset`),
  all over `Float32Array`; plus `coreVersion()`.
- TypeScript SDK (`@kryxjs/dsp`): `gain` / `mix` / `clamp` / `peak` / `rms`, the
  `Biquad` class, and `VERSION`. Dual CJS + ESM via tsup, NodeNext types.
- Tooling: tsup (external native loader), napi bindings, vitest unit tests (mock
  the native contract) + integration tests (real `.node`).
  _(0.1.0 aligns all tooling with the kryx-core molde — see that entry.)_

### Notes
- M1's Rust core is unchanged; its sources moved under `crates/dsp-core/`.
- JS↔native calls are coarse (whole buffers), per the Kryx layer-5 principle.

## [0.1.0-M1] — 2026-10-28

First milestone: the DSP Rust core and its Kryx integration.

### Added
- `samples`: `gain`, `mix_into`, `clamp`, `peak`, `rms` — pure `f32` math.
- `biquad::Biquad`: RBJ-cookbook low-pass / high-pass IIR filter with persistent
  state and `reset`.
- `stages::GainStage` and `stages::BiquadStage` — implement `kc_core::pipeline::Stage`;
  interpret `MediaBuffer` bytes as little-endian `f32` samples, preserve the pts.
- `register_dsp(&mut StageRegistry)` — registers `dsp.gain` and `dsp.filter`,
  buildable from `StageConfig` (plugs into core 0.7.0's stage registry).
- `bytes_to_f32` / `f32_to_bytes` — alignment-safe LE sample codecs.
- 16 tests (samples, biquad DC/attenuation, stages through a real `Pipeline`,
  registry + config-assembled pipeline).
