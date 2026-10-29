// Sample-level DSP functions, operating on `Float32Array` audio. Each returns a
// new buffer (the native side copies), so inputs are never mutated.

import { native } from './native.js'

/** Scale every sample by `factor` (linear gain; 1 = unchanged, 0.5 = −6 dB). */
export function gain(input: Float32Array, factor: number): Float32Array {
  return native.gain(input, factor)
}

/** Mix `src` into `dst` sample-wise over the overlap (shorter length wins). */
export function mix(dst: Float32Array, src: Float32Array): Float32Array {
  return native.mix(dst, src)
}

/** Clamp every sample to the normalized full-scale range `[-1, 1]`. */
export function clamp(input: Float32Array): Float32Array {
  return native.clamp(input)
}

/** Peak absolute amplitude across the buffer (0 for an empty buffer). */
export function peak(input: Float32Array): number {
  return native.peak(input)
}

/** Root-mean-square level across the buffer (0 for an empty buffer). */
export function rms(input: Float32Array): number {
  return native.rms(input)
}
