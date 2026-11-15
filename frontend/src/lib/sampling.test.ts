import { describe, expect, it } from 'vitest';

import { buildConfig, defaultForm, estimatedDurationSeconds, nextDay } from './sampling';

describe('sampling helpers', () => {
  it('builds each method from the form', () => {
    expect(buildConfig({ ...defaultForm(), interval: '2', unit: 'hours' })).toEqual({
      method: 'fixed-interval',
      interval_seconds: 7200,
      policy: 'closest',
      tolerance_seconds: null,
      from: null,
      to: null,
      include_flagged: true
    });
    expect(buildConfig({ ...defaultForm(), interval: '1,5', unit: 'minutes', tolerance: '0,5' })).toMatchObject({
      interval_seconds: 90,
      tolerance_seconds: 30
    });
    expect(buildConfig({ ...defaultForm(), method: 'every-nth', n: '4', offset: '' })).toMatchObject({ method: 'every-nth', n: 4, offset: 0 });
    expect(buildConfig({ ...defaultForm(), method: 'uniform', count: '12' })).toMatchObject({ method: 'uniform', count: 12 });
  });

  it('turns an inclusive end date into the exclusive bound', () => {
    expect(nextDay('2024-02-28')).toBe('2024-02-29');
    expect(nextDay('2024-12-31')).toBe('2025-01-01');
    expect(buildConfig({ ...defaultForm(), from: '2024-01-10', to: '2024-01-15' })).toMatchObject({ from: '2024-01-10', to: '2024-01-16' });
  });

  it('rejects invalid input', () => {
    expect(buildConfig({ ...defaultForm(), interval: '0' })).toBeNull();
    expect(buildConfig({ ...defaultForm(), interval: 'abc' })).toBeNull();
    expect(buildConfig({ ...defaultForm(), tolerance: '-1' })).toBeNull();
    expect(buildConfig({ ...defaultForm(), method: 'every-nth', n: '0' })).toBeNull();
    expect(buildConfig({ ...defaultForm(), method: 'every-nth', n: '2.5' })).toBeNull();
    expect(buildConfig({ ...defaultForm(), method: 'uniform', count: '' })).toBeNull();
    expect(buildConfig({ ...defaultForm(), from: '2024-02-01', to: '2024-01-01' })).toBeNull();
    expect(buildConfig({ ...defaultForm(), from: '2024-13-01' })).toBeNull();
  });

  it('estimates duration from FPS and the playback multiplier separately', () => {
    expect(estimatedDurationSeconds(300, 30, 1)).toBe(10);
    expect(estimatedDurationSeconds(300, 30, 2)).toBe(5);
    expect(estimatedDurationSeconds(300, 30, 0.5)).toBe(20);
    expect(estimatedDurationSeconds(300, 0, 1)).toBe(0);
  });
});
