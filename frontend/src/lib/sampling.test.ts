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

import { selectionLabel, selectionMethodPart, type LabelFormat } from './sampling';

describe('selection label', () => {
  const dict: Record<string, (a: Record<string, unknown>) => string> = {
    'selection-every-minutes': (a) => `Every ${a.n} min`,
    'selection-every-hours': (a) => `Every ${a.n} h`,
    'selection-every-days': (a) => `Every ${a.n} d`,
    'selection-every-seconds': (a) => `Every ${a.n} s`,
    'selection-nth': (a) => `1 in ${a.n}`,
    'selection-uniform': () => 'Evenly spaced',
    'selection-label': (a) => `${a.method} · ${a.photos} photos · ${a.date}`
  };
  const fmt: LabelFormat = {
    t: (id, a = {}) => dict[id](a),
    number: (n) => n.toLocaleString('en'),
    date: (ms, y) => new Date(ms).toLocaleDateString('en', { day: 'numeric', month: 'short', ...(y ? { year: 'numeric' } : {}), timeZone: 'UTC' })
  };
  const at = Date.UTC(2026, 10, 15, 12) / 1000;
  const now = Date.UTC(2026, 10, 20);
  const cfg = (interval: number) => ({ method: 'fixed-interval' as const, interval_seconds: interval, policy: 'closest' as const, include_flagged: true });

  it('names the method, count and day', () => {
    expect(selectionLabel({ config: cfg(300), selected_count: 1033, created_at: at }, fmt, now)).toBe('Every 5 min · 1,033 photos · Nov 15');
    expect(selectionLabel({ config: { method: 'every-nth', n: 4, offset: 0, include_flagged: true }, selected_count: 8, created_at: at }, fmt, now)).toBe('1 in 4 · 8 photos · Nov 15');
    expect(selectionLabel({ config: { method: 'uniform', count: 8, include_flagged: true }, selected_count: 8, created_at: at }, fmt, now)).toContain('Evenly spaced');
  });

  it('picks the largest unit that divides evenly, and adds the year only when it differs', () => {
    expect(selectionMethodPart({ config: cfg(7200) })).toEqual({ kind: 'interval', unit: 'hours', value: 2 });
    expect(selectionMethodPart({ config: cfg(90) })).toEqual({ kind: 'interval', unit: 'seconds', value: 90 });
    expect(selectionMethodPart({ config: cfg(86_400) })).toEqual({ kind: 'interval', unit: 'days', value: 1 });
    expect(selectionLabel({ config: cfg(300), selected_count: 1, created_at: at }, fmt, Date.UTC(2027, 0, 5))).toContain('2026');
  });
});
