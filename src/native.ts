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
}

/** A resampler instance as exposed by the native addon. */
export interface NativeResampler {
  process(input: Float32Array): Float32Array
  reset(): void
}

/** The `Resampler` constructor the native addon exports. */
export interface NativeResamplerCtor {
  new (fromHz: number, toHz: number): NativeResampler
}

/** The full surface of the native addon. */
export interface NativeBindings {
  gain(input: Float32Array, factor: number): Float32Array
  mix(dst: Float32Array, src: Float32Array): Float32Array
  clamp(input: Float32Array): Float32Array
  peak(input: Float32Array): number
  rms(input: Float32Array): number
  coreVersion(): string
  Biquad: NativeBiquadCtor
  Resampler: NativeResamplerCtor
}

export const native = addon as unknown as NativeBindings
