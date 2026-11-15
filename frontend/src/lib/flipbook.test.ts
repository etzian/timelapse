import { describe, expect, it } from 'vitest';

import { advance, clampIndex, frameIntervalMs, loopWindow, planLoad, windowIndexes, wrapIndex } from './flipbook';
import type { FrameState } from './flipbook';

describe('flipbook timing', () => {
  it('derives the frame interval from fps and speed', () => {
    expect(frameIntervalMs(25)).toBe(40);
    expect(frameIntervalMs(10, 2)).toBe(50);
    expect(frameIntervalMs(0)).toBe(1000);
  });
});

describe('flipbook window', () => {
  it('lists the playhead, then ahead, then behind, in order', () => {
    expect(windowIndexes(10, 100, 3, 2)).toEqual([10, 11, 12, 13, 9, 8]);
  });
  it('wraps past the end and before the start', () => {
    expect(windowIndexes(98, 100, 3, 2)).toEqual([98, 99, 0, 1, 97, 96]);
    expect(windowIndexes(0, 100, 1, 2)).toEqual([0, 1, 99, 98]);
  });
  it('never repeats a frame in a short sequence', () => {
    expect(windowIndexes(1, 3).sort()).toEqual([0, 1, 2]);
    expect(windowIndexes(0, 0)).toEqual([]);
  });
  it('wraps and clamps indexes', () => {
    expect(wrapIndex(4, 1, 5)).toBe(0);
    expect(wrapIndex(0, -1, 5)).toBe(4);
    expect(wrapIndex(0, 1, 0)).toBe(-1);
    expect(clampIndex(9, 5)).toBe(4);
    expect(clampIndex(-3, 5)).toBe(0);
  });
});

describe('flipbook loading plan', () => {
  it('loads the nearest missing frames in a batch', () => {
    expect(planLoad(0, 100, [0, 1], 3, 5, 1).load).toEqual([2, 3, 4]);
  });
  it('evicts frames that left the window and keeps the others', () => {
    const plan = planLoad(50, 100, [10, 50, 51, 49], 4, 5, 2);
    expect(plan.evict).toEqual([10]);
    expect(plan.load).toEqual([52, 53, 54, 55]);
  });
  it('asks for nothing once the window is full', () => {
    expect(planLoad(0, 3, [0, 1, 2]).load).toEqual([]);
  });
});

describe('flipbook playback', () => {
  const states = (m: Record<number, FrameState>) => (i: number) => m[i];
  it('steps in order and loops', () => {
    const s = states({ 0: 'ready', 1: 'ready', 2: 'ready' });
    expect(advance(0, 3, s)).toEqual({ index: 1, waiting: false });
    expect(advance(2, 3, s)).toEqual({ index: 0, waiting: false });
  });
  it('waits for a frame that is loading instead of skipping it', () => {
    expect(advance(0, 3, states({ 1: 'loading', 2: 'ready' }))).toEqual({ index: 0, waiting: true });
    expect(advance(0, 3, states({}))).toEqual({ index: 0, waiting: true });
  });
  it('goes on over a failed frame (placeholder)', () => {
    expect(advance(0, 3, states({ 1: 'failed' }))).toEqual({ index: 1, waiting: false });
  });
  it('handles an empty sequence', () => {
    expect(advance(0, 0, states({}))).toEqual({ index: 0, waiting: false });
  });
});

describe('loop window', () => {
  it('keeps at most 60 frames, centred when possible', () => {
    expect(loopWindow(500, 1000)).toEqual({ start: 470, length: 60 });
  });
  it('sticks to the edges', () => {
    expect(loopWindow(3, 1000)).toEqual({ start: 0, length: 60 });
    expect(loopWindow(999, 1000)).toEqual({ start: 940, length: 60 });
  });
  it('uses a short sequence whole and handles an empty one', () => {
    expect(loopWindow(4, 10)).toEqual({ start: 0, length: 10 });
    expect(loopWindow(0, 0)).toEqual({ start: 0, length: 0 });
  });
});
