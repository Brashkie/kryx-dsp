// Unit tests: the effect SDK wrappers should construct their native counterpart
// and forward process/reset. Native contract mocked — no .node needed.

import { beforeEach, describe, expect, it, vi } from 'vitest'

const h = vi.hoisted(() => {
  const mk = () => {
    const process = vi.fn()
    const reset = vi.fn()
    const ctor = vi.fn(function (this: Record<string, unknown>) {
      this.process = process
      this.reset = reset
    })
    return { process, reset, ctor }
  }
  return { dc: mk(), delay: mk(), limiter: mk() }
})

vi.mock('../../index.js', () => ({
  DcBlocker: h.dc.ctor,
  Delay: h.delay.ctor,
  Limiter: h.limiter.ctor,
}))

import { DcBlocker, Delay, Limiter } from '../../src/effects.js'

describe('effect SDK wrappers delegate to native', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('DcBlocker passes a default pole and forwards process', () => {
    const out = new Float32Array([1])
    h.dc.process.mockReturnValue(out)
    const b = new DcBlocker()
    expect(h.dc.ctor).toHaveBeenCalledWith(0.995)
    const input = new Float32Array([1])
    expect(b.process(input)).toBe(out)
    expect(h.dc.process).toHaveBeenCalledWith(input)
  })

  it('Delay forwards its four params', () => {
    new Delay(16, 3, 0.5, 0.25)
    expect(h.delay.ctor).toHaveBeenCalledWith(16, 3, 0.5, 0.25)
  })

  it('Limiter forwards threshold/release and reset', () => {
    const l = new Limiter(0.5, 0.9)
    expect(h.limiter.ctor).toHaveBeenCalledWith(0.5, 0.9)
    l.reset()
    expect(h.limiter.reset).toHaveBeenCalled()
  })
})
