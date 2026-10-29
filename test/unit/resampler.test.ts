// Unit test: the Resampler SDK wrapper should construct the native Resampler
// with the given rates and forward process/reset. Native contract mocked.

import { beforeEach, describe, expect, it, vi } from 'vitest'

const h = vi.hoisted(() => {
  const process = vi.fn()
  const reset = vi.fn()
  const ctor = vi.fn(function (this: Record<string, unknown>) {
    this.process = process
    this.reset = reset
  })
  return { process, reset, ctor }
})

vi.mock('../../index.js', () => ({ Resampler: h.ctor }))

import { Resampler } from '../../src/resampler.js'

describe('Resampler SDK wraps the native resampler', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('constructs the native resampler with the rates', () => {
    new Resampler(48_000, 44_100)
    expect(h.ctor).toHaveBeenCalledWith(48_000, 44_100)
  })

  it('process and reset delegate to the native instance', () => {
    const out = new Float32Array([1, 2])
    h.process.mockReturnValue(out)
    const r = new Resampler(2_000, 1_000)
    const input = new Float32Array([1, 2, 3, 4])
    expect(r.process(input)).toBe(out)
    expect(h.process).toHaveBeenCalledWith(input)
    r.reset()
    expect(h.reset).toHaveBeenCalled()
  })
})
