// Audio effects — stateful blocks backed by the native core. Each keeps state
// across `process` calls, so a stream is processed buffer by buffer.

import { type NativeProcessor, native } from './native.js'

/** Removes the DC offset (0 Hz), passes everything else. */
export class DcBlocker {
  private readonly inner: NativeProcessor

  /** `r` is the pole (default 0.995; closer to 1 passes lower frequencies). */
  constructor(r = 0.995) {
    this.inner = new native.DcBlocker(r)
  }

  /** Process a buffer. Returns a new buffer. */
  process(input: Float32Array): Float32Array {
    return this.inner.process(input)
  }

  /** Clear state. */
  reset(): void {
    this.inner.reset()
  }
}

/** A delay / echo with feedback and dry/wet mix. */
export class Delay {
  private readonly inner: NativeProcessor

  /**
   * `delaySamples` of delay (capped at `maxDelaySamples`), `feedback` (0..1) of
   * the echo re-circulated, and `mix` (0 = dry only, 1 = wet only).
   */
  constructor(maxDelaySamples: number, delaySamples: number, feedback = 0.3, mix = 0.5) {
    this.inner = new native.Delay(maxDelaySamples, delaySamples, feedback, mix)
  }

  /** Process a buffer. Returns a new buffer. */
  process(input: Float32Array): Float32Array {
    return this.inner.process(input)
  }

  /** Clear the delay line. */
  reset(): void {
    this.inner.reset()
  }
}

/** A peak limiter — holds output below `threshold`. */
export class Limiter {
  private readonly inner: NativeProcessor

  /** Linear `threshold` (e.g. 0.5) and `release` (0..1, closer to 1 = slower). */
  constructor(threshold = 0.9, release = 0.999) {
    this.inner = new native.Limiter(threshold, release)
  }

  /** Process a buffer. Returns a new buffer. */
  process(input: Float32Array): Float32Array {
    return this.inner.process(input)
  }

  /** Clear state. */
  reset(): void {
    this.inner.reset()
  }
}
