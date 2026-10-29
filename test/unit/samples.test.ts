// Unit tests: the SDK sample functions should delegate to the native addon
// faithfully. We mock the native contract (../../index.js) — no .node needed.

import { beforeEach, describe, expect, it, vi } from 'vitest'

const h = vi.hoisted(() => ({
  gain: vi.fn(),
  mix: vi.fn(),
  clamp: vi.fn(),
  peak: vi.fn(),
  rms: vi.fn(),
}))

vi.mock('../../index.js', () => h)

import { clamp, gain, mix, peak, rms } from '../../src/samples.js'

describe('samples SDK delegates to native', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('gain forwards args and returns the native result', () => {
    const out = new Float32Array([2])
    h.gain.mockReturnValue(out)
    const input = new Float32Array([1])
    expect(gain(input, 2)).toBe(out)
    expect(h.gain).toHaveBeenCalledWith(input, 2)
  })

  it('mix forwards both buffers', () => {
    const out = new Float32Array([0.5])
    h.mix.mockReturnValue(out)
    const a = new Float32Array([0.1])
    const b = new Float32Array([0.4])
    expect(mix(a, b)).toBe(out)
    expect(h.mix).toHaveBeenCalledWith(a, b)
  })

  it('clamp forwards the buffer', () => {
    const out = new Float32Array([1])
    h.clamp.mockReturnValue(out)
    const input = new Float32Array([2])
    expect(clamp(input)).toBe(out)
    expect(h.clamp).toHaveBeenCalledWith(input)
  })

  it('peak and rms return the native numbers', () => {
    h.peak.mockReturnValue(0.9)
    h.rms.mockReturnValue(0.5)
    const input = new Float32Array([0.9, -0.1])
    expect(peak(input)).toBe(0.9)
    expect(rms(input)).toBe(0.5)
  })
})
