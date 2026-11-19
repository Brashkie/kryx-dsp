// Loads the compiled native addon through napi's generated loader (../index.js),
// which picks the right prebuilt .node for the current platform. Kept external by
// tsup so it is never bundled.
import * as addon from '../index.js'

/** A biquad filter instance as exposed by the native addon. */
export interface NativeBiquad {
  process(input: Float32Array): Float32Array
  reset(): void
}

/** The `Biquad` constructor the native addon exports (factory methods). */
export interface NativeBiquadCtor {
  lowPass(sampleRate: number, cutoffHz: number, q: number): NativeBiquad
  highPass(sampleRate: number, cutoffHz: number, q: number): NativeBiquad
  bandPass(sampleRate: number, centerHz: number, q: number): NativeBiquad
  notch(sampleRate: number, centerHz: number, q: number): NativeBiquad
  peaking(sampleRate: number, centerHz: number, q: number, gainDb: number): NativeBiquad
  lowShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): NativeBiquad
  highShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): NativeBiquad
  allPass(sampleRate: number, centerHz: number, q: number): NativeBiquad
}

/** A parametric equalizer instance as exposed by the native addon. */
export interface NativeEqualizer {
  readonly length: number
  addLowPass(sampleRate: number, cutoffHz: number, q: number): void
  addHighPass(sampleRate: number, cutoffHz: number, q: number): void
  addBandPass(sampleRate: number, centerHz: number, q: number): void
  addNotch(sampleRate: number, centerHz: number, q: number): void
  addAllPass(sampleRate: number, centerHz: number, q: number): void
  addPeaking(sampleRate: number, centerHz: number, q: number, gainDb: number): void
  addLowShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): void
  addHighShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): void
  process(input: Float32Array): Float32Array
  reset(): void
}

/** The `Equalizer` constructor the native addon exports. */
export interface NativeEqualizerCtor {
  new (): NativeEqualizer
}

/** A processing block with buffer-at-a-time state (resampler, effect). */
export interface NativeProcessor {
  process(input: Float32Array): Float32Array
  reset(): void
}

/** The `Resampler` (linear) constructor the native addon exports. */
export interface NativeResamplerCtor {
  new (fromHz: number, toHz: number): NativeProcessor
}

/** The `SincResampler` constructor the native addon exports. */
export interface NativeSincCtor {
  new (fromHz: number, toHz: number, quality?: number): NativeProcessor
}

/** The `DcBlocker` constructor. */
export interface NativeDcBlockerCtor {
  new (r?: number): NativeProcessor
}

/** The `Delay` constructor. */
export interface NativeDelayCtor {
  new (
    maxDelaySamples: number,
    delaySamples: number,
    feedback: number,
    mix: number,
  ): NativeProcessor
}

/** The `Limiter` constructor. */
export interface NativeLimiterCtor {
  new (threshold: number, release: number): NativeProcessor
}

/** The full surface of the native addon. */
export interface NativeBindings {
  gain(input: Float32Array, factor: number): Float32Array
  mix(dst: Float32Array, src: Float32Array): Float32Array
  clamp(input: Float32Array): Float32Array
  peak(input: Float32Array): number
  rms(input: Float32Array): number
  dbToLinear(db: number): number
  linearToDb(linear: number): number
  gainDb(input: Float32Array, db: number): Float32Array
  softClip(input: Float32Array, threshold: number): Float32Array
  coreVersion(): string
  Biquad: NativeBiquadCtor
  Equalizer: NativeEqualizerCtor
  Resampler: NativeResamplerCtor
  SincResampler: NativeSincCtor
  DcBlocker: NativeDcBlockerCtor
  Delay: NativeDelayCtor
  Limiter: NativeLimiterCtor
}

export const native = addon as unknown as NativeBindings
