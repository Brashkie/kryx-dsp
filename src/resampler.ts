// Streaming sample-rate conversion. State carries across `process` calls, so a
// stream can be resampled buffer by buffer with click-free seams.
//
// Linear interpolation is cheap and click-free but not alias-free; for large
// downsampling ratios, low-pass first (see `Biquad.lowPass`).

import { type NativeResampler, native } from './native.js'

export class Resampler {
  private readonly inner: NativeResampler

  /** A resampler from `fromHz` to `toHz`. Equal rates give a pass-through. */
  constructor(fromHz: number, toHz: number) {
    this.inner = new native.Resampler(fromHz, toHz)
  }

  /**
   * Resample a buffer, returning the samples produced from this chunk. A chunk
   * may legitimately yield an empty buffer at a fractional boundary.
   */
  process(input: Float32Array): Float32Array {
    return this.inner.process(input)
  }

  /** Clear state (as if starting a fresh stream); keeps the rate ratio. */
  reset(): void {
    this.inner.reset()
  }
}
