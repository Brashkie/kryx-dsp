//! @kryxjs/dsp — audio DSP processing for the Kryx ecosystem.
//
// The Processing layer: sample-level math (gain, mix, clamp, peak, rms) and a
// biquad filter, backed by a pure-Rust core through N-API. The Stage / Pipeline
// integration lives in the Rust core for composing pipelines natively.

export { gain, mix, clamp, peak, rms } from './samples.js'
export { Biquad } from './biquad.js'
export { Resampler } from './resampler.js'
export { native } from './native.js'

/** Version of the `@kryxjs/dsp` package. */
export const VERSION = '0.1.0'
