// A parametric equalizer — an ordered chain of biquad bands applied in series.
// Build it fluently from the biquad filter set to shape a spectrum.

import { type NativeEqualizer, native } from './native.js'

/** Default resonance (Q): Butterworth response (1/√2). */
const DEFAULT_Q = Math.SQRT1_2

export class Equalizer {
  private readonly inner: NativeEqualizer = new native.Equalizer()

  /** Number of bands currently in the chain. */
  get length(): number {
    return this.inner.length
  }

  /** Append a low-pass band. Returns `this` for chaining. */
  addLowPass(sampleRate: number, cutoffHz: number, q: number = DEFAULT_Q): this {
    this.inner.addLowPass(sampleRate, cutoffHz, q)
    return this
  }

  /** Append a high-pass band. */
  addHighPass(sampleRate: number, cutoffHz: number, q: number = DEFAULT_Q): this {
    this.inner.addHighPass(sampleRate, cutoffHz, q)
    return this
  }

  /** Append a band-pass band. */
  addBandPass(sampleRate: number, centerHz: number, q: number = DEFAULT_Q): this {
    this.inner.addBandPass(sampleRate, centerHz, q)
    return this
  }

  /** Append a notch band. */
  addNotch(sampleRate: number, centerHz: number, q: number = DEFAULT_Q): this {
    this.inner.addNotch(sampleRate, centerHz, q)
    return this
  }

  /** Append an all-pass band (phase shift only). */
  addAllPass(sampleRate: number, centerHz: number, q: number = DEFAULT_Q): this {
    this.inner.addAllPass(sampleRate, centerHz, q)
    return this
  }

  /** Append a peaking band (`gainDb` boost/cut around `centerHz`). */
  addPeaking(sampleRate: number, centerHz: number, q: number, gainDb: number): this {
    this.inner.addPeaking(sampleRate, centerHz, q, gainDb)
    return this
  }

  /** Append a low-shelf band. */
  addLowShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): this {
    this.inner.addLowShelf(sampleRate, cornerHz, q, gainDb)
    return this
  }

  /** Append a high-shelf band. */
  addHighShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): this {
    this.inner.addHighShelf(sampleRate, cornerHz, q, gainDb)
    return this
  }

  /** Filter a buffer through every band in series. Returns a new buffer. */
  process(input: Float32Array): Float32Array {
    return this.inner.process(input)
  }

  /** Clear the state of every band. */
  reset(): void {
    this.inner.reset()
  }
}
