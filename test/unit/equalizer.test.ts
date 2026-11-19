// Unit test: the Equalizer SDK wrapper should construct the native Equalizer,
// forward each add*/process/reset, and chain. Native contract mocked.

import { beforeEach, describe, expect, it, vi } from 'vitest'

const h = vi.hoisted(() => {
  const inst = {
    length: 0,
    addLowPass: vi.fn(),
    addPeaking: vi.fn(),
    addAllPass: vi.fn(),
    process: vi.fn(),
    reset: vi.fn(),
  }
  const ctor = vi.fn(() => inst)
  return { inst, ctor }
})

vi.mock('../../index.js', () => ({ Equalizer: h.ctor }))

import { Equalizer } from '../../src/equalizer.js'

describe('Equalizer SDK wraps the native chain', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('constructs the native equalizer', () => {
    new Equalizer()
    expect(h.ctor).toHaveBeenCalled()
  })

  it('add* forwards params and returns this for chaining', () => {
    const eq = new Equalizer()
    const ret = eq.addLowPass(48_000, 1_000).addPeaking(48_000, 3_000, 1, 6)
    expect(ret).toBe(eq)
    expect(h.inst.addLowPass).toHaveBeenCalledWith(48_000, 1_000, Math.SQRT1_2)
    expect(h.inst.addPeaking).toHaveBeenCalledWith(48_000, 3_000, 1, 6)
  })

  it('process and reset delegate', () => {
    const out = new Float32Array([1])
    h.inst.process.mockReturnValue(out)
    const eq = new Equalizer()
    const input = new Float32Array([1])
    expect(eq.process(input)).toBe(out)
    expect(h.inst.process).toHaveBeenCalledWith(input)
    eq.reset()
    expect(h.inst.reset).toHaveBeenCalled()
  })
})
