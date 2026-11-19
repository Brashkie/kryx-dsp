// Streaming sample-rate conversion. State carries across `process` calls, so a
// stream can be resampled buffer by buffer with click-free seams.

import { type NativeProcessor, native } from './native.js'

/**
 * Linear resampler — cheap and click-free, but not alias-free. For large
 * downsampling ratios prefer `SincResampler` (anti-aliased).
 */
export class Resampler {
  private readonly inner: NativeProcessor

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

/**
 * High-quality anti-aliased resampler (Kaiser-windowed sinc). When downsampling
 * it attenuates content above the new Nyquist frequency.
 */
export class SincResampler {
  private readonly inner: NativeProcessor

  /**
   * A resampler from `fromHz` to `toHz`. `quality` is the number of sinc
   * zero-crossings per side (default 16; higher is sharper and costlier).
   */
  constructor(fromHz: number, toHz: number, quality = 16) {
    this.inner = new native.SincResampler(fromHz, toHz, quality)
  }

  /** Resample a buffer, returning the samples produced from this chunk. */
  process(input: Float32Array): Float32Array {
    return this.inner.process(input)
  }

  /** Clear state (fresh stream); keeps the rate ratio and kernel. */
  reset(): void {
    this.inner.reset()
  }
}
