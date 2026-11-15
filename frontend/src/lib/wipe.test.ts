import { describe, expect, it } from 'vitest';

import { createFrameCache } from './frameCache';
import { clampPercent, percentFromPointer, wipeKey } from './wipe';

describe('wipe', () => {
  it('clamps the divider', () => {
    expect(clampPercent(-5)).toBe(0);
    expect(clampPercent(140)).toBe(100);
    expect(clampPercent(NaN)).toBe(50);
  });
  it('moves with the keys and refuses the others', () => {
    expect(wipeKey(50, 'ArrowRight')).toBe(51);
    expect(wipeKey(50, 'ArrowLeft', true)).toBe(40);
    expect(wipeKey(99, 'ArrowRight', true)).toBe(100);
    expect(wipeKey(50, 'Home')).toBe(0);
    expect(wipeKey(50, 'End')).toBe(100);
    expect(wipeKey(50, 'a')).toBeNull();
    expect(wipeKey(50, 'Tab')).toBeNull();
  });
  it('reads the pointer inside and outside the frame', () => {
    expect(percentFromPointer(150, 100, 200)).toBe(25);
    expect(percentFromPointer(50, 100, 200)).toBe(0);
    expect(percentFromPointer(900, 100, 200)).toBe(100);
    expect(percentFromPointer(10, 0, 0)).toBe(50);
  });
});

describe('frame cache', () => {
  it('fetches a key once', async () => {
    const c = createFrameCache<string>();
    let n = 0;
    const f = async () => `v${++n}`;
    expect(await c.get('run1', 'a', f)).toBe('v1');
    expect(await c.get('run1', 'a', f)).toBe('v1');
    expect(n).toBe(1);
  });
  it('drops entries when the run changes', async () => {
    const c = createFrameCache<string>();
    let n = 0;
    await c.get('run1', 'a', async () => `v${++n}`);
    await c.get('run2', 'a', async () => `v${++n}`);
    expect(n).toBe(2);
    expect(c.size).toBe(1);
  });
  it('forgets a failure and stays bounded', async () => {
    const c = createFrameCache<string>(2);
    await expect(c.get('r', 'bad', () => Promise.reject(new Error('x')))).rejects.toThrow();
    await Promise.resolve();
    expect(c.size).toBe(0);
    await c.get('r', 'a', async () => '1');
    await c.get('r', 'b', async () => '2');
    await c.get('r', 'c', async () => '3');
    expect(c.size).toBe(2);
  });
});
