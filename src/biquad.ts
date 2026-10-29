// A biquad filter — second-order IIR, the workhorse of audio EQ. State persists
// across `process` calls, so a stream can be filtered buffer by buffer.

import { type NativeBiquad, native } from './native.js'

/** Default resonance (Q): Butterworth response (1/√2), maximally flat passband. */
const DEFAULT_Q = Math.SQRT1_2

export class Biquad {
  private readonly inner: NativeBiquad

  private constructor(inner: NativeBiquad) {
    this.inner = inner
  }

  /** A low-pass filter at `cutoffHz` (Hz) for audio at `sampleRate` (Hz). */
  static lowPass(sampleRate: number, cutoffHz: number, q: number = DEFAULT_Q): Biquad {
    return new Biquad(native.Biquad.lowPass(sampleRate, cutoffHz, q))
  }

  /** A high-pass filter at `cutoffHz` (Hz) for audio at `sampleRate` (Hz). */
  static highPass(sampleRate: number, cutoffHz: number, q: number = DEFAULT_Q): Biquad {
    return new Biquad(native.Biquad.highPass(sampleRate, cutoffHz, q))
  }

  /** A band-pass filter centered at `centerHz` (constant 0 dB peak gain). */
  static bandPass(sampleRate: number, centerHz: number, q: number = DEFAULT_Q): Biquad {
    return new Biquad(native.Biquad.bandPass(sampleRate, centerHz, q))
  }

  /** A notch (band-reject) filter that removes `centerHz`. */
  static notch(sampleRate: number, centerHz: number, q: number = DEFAULT_Q): Biquad {
    return new Biquad(native.Biquad.notch(sampleRate, centerHz, q))
  }

  /** A peaking EQ: boost or cut `gainDb` around `centerHz`. */
  static peaking(sampleRate: number, centerHz: number, q: number, gainDb: number): Biquad {
    return new Biquad(native.Biquad.peaking(sampleRate, centerHz, q, gainDb))
  }

  /** A low-shelf: boost or cut `gainDb` below `cornerHz`. */
  static lowShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): Biquad {
    return new Biquad(native.Biquad.lowShelf(sampleRate, cornerHz, q, gainDb))
  }

  /** A high-shelf: boost or cut `gainDb` above `cornerHz`. */
  static highShelf(sampleRate: number, cornerHz: number, q: number, gainDb: number): Biquad {
    return new Biquad(native.Biquad.highShelf(sampleRate, cornerHz, q, gainDb))
  }

  /** Filter a buffer, advancing the filter state. Returns a new buffer. */
  process(input: Float32Array): Float32Array {
    return this.inner.process(input)
  }

  /** Clear the filter state (as if starting a fresh stream). */
  reset(): void {
    this.inner.reset()
  }
}
