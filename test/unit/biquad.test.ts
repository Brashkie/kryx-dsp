// Unit tests: the Biquad SDK wrapper should build via the native factories and
// forward process/reset. Native contract mocked — no .node needed.

import { beforeEach, describe, expect, it, vi } from 'vitest'

const h = vi.hoisted(() => ({ process: vi.fn(), reset: vi.fn() }))

vi.mock('../../index.js', () => ({
  Biquad: {
    lowPass: vi.fn(() => h),
    highPass: vi.fn(() => h),
  },
}))

import * as addon from '../../index.js'
import { Biquad } from '../../src/biquad.js'

// Re-type the mocked addon for assertions.
const native = addon as unknown as {
  Biquad: { lowPass: ReturnType<typeof vi.fn>; highPass: ReturnType<typeof vi.fn> }
}

describe('Biquad SDK wraps the native filter', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('lowPass passes a default Q when omitted', () => {
    Biquad.lowPass(48_000, 1_000)
    expect(native.Biquad.lowPass).toHaveBeenCalledWith(48_000, 1_000, Math.SQRT1_2)
  })

  it('highPass forwards an explicit Q', () => {
    Biquad.highPass(48_000, 500, 1.2)
    expect(native.Biquad.highPass).toHaveBeenCalledWith(48_000, 500, 1.2)
  })

  it('process and reset delegate to the native instance', () => {
    const out = new Float32Array([0.5])
    h.process.mockReturnValue(out)
    const f = Biquad.lowPass(48_000, 1_000)
    const input = new Float32Array([1])
    expect(f.process(input)).toBe(out)
    expect(h.process).toHaveBeenCalledWith(input)
    f.reset()
    expect(h.reset).toHaveBeenCalled()
  })
})
