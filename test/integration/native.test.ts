// Integration tests: run the REAL native addon through the SDK. Requires a
// debug build first (`npm run build:debug`), which produces index.js + the .node.

import { describe, expect, it } from 'vitest'
import { Biquad, Resampler, VERSION, clamp, gain, mix, peak, rms } from '../../src/index.js'

/** Peak of a filtered sine tail — steady-state amplitude at `hz`. */
function responseAt(f: Biquad, sr: number, hz: number): number {
  const buf = new Float32Array(8_000)
  for (let n = 0; n < buf.length; n++) buf[n] = Math.sin((2 * Math.PI * hz * n) / sr)
  const out = f.process(buf)
  let peakOut = 0
  for (let i = 4_000; i < out.length; i++) peakOut = Math.max(peakOut, Math.abs(out[i]))
  return peakOut
}

describe('@kryxjs/dsp native integration', () => {
  it('gain scales samples exactly', () => {
    const out = gain(new Float32Array([0.5, -0.25, 1.0]), 2)
    expect(out[0]).toBeCloseTo(1.0, 6)
    expect(out[1]).toBeCloseTo(-0.5, 6)
    expect(out[2]).toBeCloseTo(2.0, 6)
  })

  it('mix sums over the overlap (shorter src)', () => {
    const out = mix(new Float32Array([0.1, 0.2, 0.3]), new Float32Array([0.4, 0.4]))
    expect(out[0]).toBeCloseTo(0.5, 6)
    expect(out[1]).toBeCloseTo(0.6, 6)
    expect(out[2]).toBeCloseTo(0.3, 6) // untouched
  })

  it('clamp bounds to full scale', () => {
    const out = clamp(new Float32Array([2, -3, 0.5]))
    expect(Array.from(out)).toEqual([1, -1, 0.5])
  })

  it('peak and rms', () => {
    const s = new Float32Array([0, 1, -1, 0])
    expect(peak(s)).toBeCloseTo(1, 6)
    expect(rms(s)).toBeCloseTo(Math.SQRT1_2, 5)
  })

  it('low-pass passes DC near unity once settled', () => {
    const f = Biquad.lowPass(48_000, 1_000, Math.SQRT1_2)
    const out = f.process(new Float32Array(2_000).fill(1))
    expect(out[out.length - 1]).toBeCloseTo(1, 2)
  })

  it('high-pass blocks DC', () => {
    const f = Biquad.highPass(48_000, 1_000, Math.SQRT1_2)
    const out = f.process(new Float32Array(2_000).fill(1))
    expect(Math.abs(out[out.length - 1])).toBeLessThan(0.05)
  })

  it('filter state carries across process calls', () => {
    const f = Biquad.lowPass(48_000, 1_000, Math.SQRT1_2)
    f.process(new Float32Array(1_000).fill(1))
    const out = f.process(new Float32Array(1_000).fill(1))
    expect(out[out.length - 1]).toBeCloseTo(1, 2)
  })

  it('band-pass passes its center, rejects extremes', () => {
    const sr = 48_000
    expect(responseAt(Biquad.bandPass(sr, 1_000, 1), sr, 1_000)).toBeGreaterThan(0.9)
    expect(responseAt(Biquad.bandPass(sr, 1_000, 1), sr, 60)).toBeLessThan(0.3)
  })

  it('notch rejects its center', () => {
    const sr = 48_000
    expect(responseAt(Biquad.notch(sr, 1_000, 1), sr, 1_000)).toBeLessThan(0.2)
    expect(responseAt(Biquad.notch(sr, 1_000, 1), sr, 8_000)).toBeGreaterThan(0.8)
  })

  it('peaking boosts ~+6 dB at its center', () => {
    const boosted = responseAt(Biquad.peaking(48_000, 1_000, 1, 6), 48_000, 1_000)
    expect(boosted).toBeGreaterThan(1.8)
    expect(boosted).toBeLessThan(2.1)
  })

  it('resampler downsamples to about half the samples', () => {
    const r = new Resampler(2_000, 1_000)
    const input = new Float32Array(1_000).map((_, i) => i)
    const out = r.process(input)
    expect(Math.abs(out.length - 500)).toBeLessThan(4)
  })

  it('resampler keeps seams continuous across buffers', () => {
    const full = new Float32Array(200).map((_, i) => i)
    const whole = new Resampler(3_000, 2_000).process(full)
    const split = new Resampler(3_000, 2_000)
    const a = split.process(full.slice(0, 100))
    const b = split.process(full.slice(100))
    const joined = new Float32Array(a.length + b.length)
    joined.set(a, 0)
    joined.set(b, a.length)
    expect(joined.length).toBe(whole.length)
    for (let i = 0; i < whole.length; i++) {
      expect(Math.abs(whole[i] - joined[i])).toBeLessThan(1e-3)
    }
  })

  it('exposes a VERSION', () => {
    expect(VERSION).toBeTruthy()
  })
})
