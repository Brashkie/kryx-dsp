# @kryxjs/dsp

[![CI](https://github.com/Brashkie/kryx-dsp/actions/workflows/ci.yml/badge.svg)](https://github.com/Brashkie/kryx-dsp/actions/workflows/ci.yml)
[![npm](https://img.shields.io/npm/v/@kryxjs/dsp.svg)](https://www.npmjs.com/package/@kryxjs/dsp)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](./LICENSE)

[English](./README.md) · **Español**

Procesamiento de audio DSP para el ecosistema [Kryx](https://github.com/Brashkie) —
la **capa de Processing**. Un núcleo en Rust puro (matemática de muestras + filtros
biquad + resampling, además de stages de pipeline para `@kryxjs/core`) con un SDK
delgado en TypeScript sobre N-API.

Es el primer consumidor real de la superficie `Stage` / `Pipeline` / `StageRegistry`
del core: construirlo ejercita y valida esa API contra una carga de trabajo genuina.

## Instalación

```
npm install @kryxjs/dsp
```

## Inicio rápido (TypeScript / Node)

```ts
import { gain, peak, rms, Biquad } from '@kryxjs/dsp'

// Matemática de muestras — cada llamada devuelve un Float32Array nuevo (no muta la entrada).
const masBajo = gain(samples, 0.5) // −6 dB
const nivel = rms(samples)
const margen = 1 - peak(samples)

// Un filtro biquad mantiene estado entre llamadas: filtra el stream buffer por buffer.
const lp = Biquad.lowPass(48_000, 3_000) // sampleRate, cutoffHz, q = 0.707
for (const frame of frames) {
  emit(lp.process(frame))
}
```

### API

- `gain(samples, factor)` — ganancia lineal (volumen).
- `mix(dst, src)` — suma `src` en `dst` sobre el solapamiento.
- `clamp(samples)` — acota cada muestra a `[-1, 1]`.
- `peak(samples)` / `rms(samples)` — medición de nivel.
- `Biquad` — filtro IIR de segundo orden con `.process(samples)` y `.reset()`.
  Factorías: `lowPass` / `highPass` / `bandPass` / `notch` (`sampleRate, freq, q?`)
  y `peaking` / `lowShelf` / `highShelf` (`sampleRate, freq, q, gainDb`).
- `Resampler(fromHz, toHz)` — conversión de sample-rate lineal en streaming con
  `.process(samples)` y `.reset()`; sin clicks entre buffers.

Todos los buffers son `Float32Array`. Las llamadas JS↔nativo son gruesas (buffer
entero por vez) para que el costo del cruce nunca supere el trabajo hecho.

## Uso desde Rust (stages de pipeline)

El crate del core también expone el DSP como `Stage`s, así podés componer un
pipeline nativo e incluso armarlo desde configuración:

```rust
use kc_core::pipeline::{StageRegistry, StageConfig};
use kryx_dsp::register_dsp;

let mut reg = StageRegistry::new();
register_dsp(&mut reg);

// Una cadena low-pass → gain, descrita por configuración.
let pipeline = reg.build_pipeline(&[
    ("dsp.filter", StageConfig::new().set("cutoff", 3_000.0)),
    ("dsp.gain",   StageConfig::new().set("factor", 0.8)),
]).unwrap();
```

Los frames de audio se interpretan como **muestras `f32` little-endian** — ese es
el trabajo de la capa DSP; el core se mantiene agnóstico a los bytes.

## Dependencia del core

El núcleo Rust depende de `kc-core`. En `crates/dsp-core/Cargo.toml` apunta a un
checkout local (`path = "../../../kryx-core/crates/kc-core"`), asumiendo que
`kryx-core` y `kryx-dsp` están uno al lado del otro. Cambialo a dependencia git si
preferís:

```toml
kc-core = { git = "https://github.com/Brashkie/kryx-core" }
```

## Estructura

```
crates/dsp-core   Rust puro: samples, biquad, resample, stages, registry
crates/dsp-node   bindings N-API (cdylib)
src               SDK TypeScript
test              vitest unit (mock) + integration (.node real)
```

## Compilar y testear

```
npm run test:rust            # núcleo Rust (cargo test --workspace --exclude dsp-node)
npm run build:debug          # compila el addon nativo + TS
npm run test:vitest          # SDK unit + integration (vitest)
npm run typecheck            # tsc --noEmit
npm run lint                 # biome
```

Antes de commitear, formateá ambos lados: `npm run format:rust` (cargo fmt) y
`npm run format` (biome).

## Roadmap

`0.1.0` es la primera versión estable (samples, set de biquad, resampling, stages
de pipeline + registry, SDK TS). Lo que sigue está en [ROADMAP.md](./ROADMAP.md):
resampling de mayor calidad (windowed-sinc), más efectos (delay, compresor) y un
target WASM.

Parte del ecosistema multimedia Kryx · Apache-2.0
