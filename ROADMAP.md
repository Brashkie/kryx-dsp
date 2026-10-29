# Roadmap

`@kryxjs/dsp` reaches `0.1.0` with the Processing-layer essentials: sample math,
a full biquad filter set, streaming linear resampling, and pipeline stages that
plug into `@kryxjs/core` (directly and via config through `StageRegistry`).

What comes next is grouped into horizons. As with the rest of Kryx, each item
earns its place by a measured need, not by default — "small scope, grow by proven
need."

## H1 — Completing the DSP core

- Higher-quality resampling: windowed-sinc / polyphase, alias-aware, alongside
  the current linear path (keep linear as the cheap option).
- More filters: all-pass, and parametric EQ chains built from the biquad set.
- Gain helpers in dB, soft-clip / limiter, DC blocker.
- Frame-rate-agnostic stages that read the stream's sample rate from context
  instead of fixed constructor params.

## H2 — Performance

- SIMD-friendly inner loops (let the Rust auto-vectorizer work; measure before
  hand-writing anything).
- Zero-copy sample passing across N-API where it beats the current copy.
- A benchmark suite (criterion) for the filters and the resampler, with numbers
  in the README — no "faster than X" claims without reproducible benchmarks.

## H3 — Reach & effects

- More effects: delay / echo, a simple compressor, a noise gate.
- A WASM target so the same DSP runs in the browser.
- Richer diagnostics and a small CLI for offline processing.
- Fuzzing + property tests on the resampler and filter stability.

The public API (the SDK surface and the `dsp.*` stage names) is considered stable
for `0.1.x`; internals may change freely as these land.
