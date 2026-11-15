import { describe, expect, it } from 'vitest';
import { fraction, MIN_ELAPSED_MS, observe, remainingSeconds, startRate } from './progress';

/** 10 items per second, one reading per second. */
function steady(readings: number, perSecond = 10) {
  let s = startRate(0, 0);
  for (let i = 1; i <= readings; i++) s = observe(s, i * perSecond, i * 1000);
  return s;
}

describe('remainingSeconds', () => {
  it('gives no estimate before the rate is stable', () => {
    expect(remainingSeconds(null, 5, 100)).toBeNull();
    expect(remainingSeconds(startRate(0, 0), 0, 100)).toBeNull();
    expect(remainingSeconds(steady(2), 20, 100)).toBeNull();
  });
  it('needs enough elapsed time, not only enough samples', () => {
    let s = startRate(0, 0);
    for (let i = 1; i <= 6; i++) s = observe(s, i, i * 100);
    expect(s.lastAt).toBeLessThan(MIN_ELAPSED_MS);
    expect(remainingSeconds(s, 6, 100)).toBeNull();
  });
  it('estimates from a steady rate', () => {
    expect(remainingSeconds(steady(5), 50, 100)).toBe(5);
  });
  it('smooths a slow-down instead of jumping', () => {
    const s = observe(steady(5), 51, 6000);
    const left = remainingSeconds(s, 51, 100);
    expect(left).toBeGreaterThan(5);
    expect(left).toBeLessThan(20);
  });
  it('gives nothing when finished or stalled', () => {
    expect(remainingSeconds(steady(5), 100, 100)).toBeNull();
    const stalled = observe(steady(5), 50, 9000);
    expect(stalled.lastDone).toBe(50);
  });
  it('starts over when progress goes backwards', () => {
    const s = observe(steady(5), 3, 7000);
    expect(s.samples).toBe(0);
    expect(remainingSeconds(s, 3, 100)).toBeNull();
  });
});

describe('fraction', () => {
  it('is clamped and never NaN', () => {
    expect(fraction(5, 10)).toBe(0.5);
    expect(fraction(0, 0)).toBe(0);
    expect(fraction(20, 10)).toBe(1);
    expect(fraction(-1, 10)).toBe(0);
  });
});
