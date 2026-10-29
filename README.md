# @kryxjs/dsp

[![CI](https://github.com/Brashkie/kryx-dsp/actions/workflows/ci.yml/badge.svg)](https://github.com/Brashkie/kryx-dsp/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/@kryxjs/dsp.svg)](https://www.npmjs.com/package/@kryxjs/dsp)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

**English** · [Español](./README.es.md)

Audio DSP processing for the [Kryx](https://github.com/Brashkie) ecosystem — the
**Processing layer**. A pure-Rust core (sample math + biquad filters + resampling,
plus pipeline stages for `@kryxjs/core`) with a thin TypeScript SDK over N-API.

It is the first real consumer of core's `Stage` / `Pipeline` / `StageRegistry`
surface: building it exercises and validates that API against a genuine workload.

## Install

```
npm install @kryxjs/dsp
```

## Quickstart (TypeScript / Node)

```ts
import { gain, peak, rms, Biquad } from '@kryxjs/dsp'

// Sample math — each call returns a new Float32Array (inputs are never mutated).
const quieter = gain(samples, 0.5) // −6 dB
const level = rms(samples)
const headroom = 1 - peak(samples)

// A biquad filter keeps state across calls, so filter a stream buffer by buffer.
const lp = Biquad.lowPass(48_000, 3_000) // sampleRate, cutoffHz, q = 0.707
for (const frame of frames) {
  emit(lp.process(frame))
}
```

### API

- `gain(samples, factor)` — linear gain (volume).
- `mix(dst, src)` — sum `src` into `dst` over the overlap.
- `clamp(samples)` — bound every sample to `[-1, 1]`.
- `peak(samples)` / `rms(samples)` — level metering.
- `Biquad` — second-order IIR filter with `.process(samples)` and `.reset()`.
  Factories: `lowPass` / `highPass` / `bandPass` / `notch` (`sampleRate, freq, q?`)
  and `peaking` / `lowShelf` / `highShelf` (`sampleRate, freq, q, gainDb`).
- `Resampler(fromHz, toHz)` — streaming linear sample-rate conversion with
  `.process(samples)` and `.reset()`; click-free across buffers.

All buffers are `Float32Array`. JS↔native calls stay coarse (whole buffers at a
time) so the crossing cost never outweighs the work.

## Using it from Rust (pipeline stages)

The core crate also exposes the DSP as `Stage`s, so you can compose a native
pipeline and even assemble one from config:

```rust
use kc_core::pipeline::{StageRegistry, StageConfig};
use kryx_dsp::register_dsp;

let mut reg = StageRegistry::new();
register_dsp(&mut reg);

// A low-pass → gain chain, described by config.
let pipeline = reg.build_pipeline(&[
    ("dsp.filter", StageConfig::new().set("cutoff", 3_000.0)),
    ("dsp.gain",   StageConfig::new().set("factor", 0.8)),
]).unwrap();
```

Audio frames are interpreted as **little-endian `f32` samples** — that's the DSP
layer's job; core stays byte-agnostic.

## Dependency on core

The Rust core depends on `kc-core`. In `crates/dsp-core/Cargo.toml` it points at a
local checkout (`path = "../../../kryx-core/crates/kc-core"`), expecting
`kryx-core` and `kryx-dsp` to sit side by side. Switch to a git dependency if you
prefer:

```toml
kc-core = { git = "https://github.com/Brashkie/kryx-core" }
```

## Layout

```
crates/dsp-core   pure Rust: samples, biquad, resample, stages, registry
crates/dsp-node   N-API bindings (cdylib)
src               TypeScript SDK
test              vitest unit (mock) + integration (real .node)
```

## Build & test

```
npm run test:rust            # Rust core (cargo test --workspace --exclude dsp-node)
npm run build:debug          # compile the native addon + TS
npm run test:vitest          # SDK unit + integration (vitest)
npm run typecheck            # tsc --noEmit
npm run lint                 # biome
```

Before committing, format both sides: `npm run format:rust` (cargo fmt) and
`npm run format` (biome).

## Roadmap

`0.1.0` is the first stable release (samples, biquad set, resampling, pipeline
stages + registry, TS SDK). What's next lives in [ROADMAP.md](./ROADMAP.md):
higher-quality (windowed-sinc) resampling, more effects (delay, compressor), and
a WASM target.

Part of the Kryx multimedia ecosystem · Apache-2.0
