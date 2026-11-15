import { describe, expect, it } from 'vitest';

import { ADJUST_FAILURES, adjustFailureText, brightnessChart, colourChart, formatStops, median, pathChart } from './adjust';
import { setLocale, t } from './i18n';
import type { FlickerPoint, StabilizationPoint } from './ipc/app';

const point = (i: number, before: number | null, after: number | null): FlickerPoint => ({
  sequence_index: i,
  before,
  trend: before,
  after,
  red_before: 0.2,
  red_after: 0.1,
  blue_before: -0.3,
  blue_after: -0.3
});

describe('adjust helpers', () => {
  it('has text for every refusal and every page string in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      const texts = new Set(ADJUST_FAILURES.map((f) => adjustFailureText(f)));
      expect(texts.size).toBe(ADJUST_FAILURES.length);
      expect(adjustFailureText('new-thing')).toBe(adjustFailureText(null));
      expect(t('error-export', { reason: 'adjustment-mismatch' })).not.toBe(t('error-export', { reason: 'other' }));
      expect(t('adjust-frames-option', { count: 1 })).not.toBe(t('adjust-frames-option', { count: 15 }));
      expect(t('nav-adjust')).not.toBe('nav-adjust');
    }
  });

  it('shows brightness and colour relative to the sequence median', () => {
    setLocale('en');
    const points = [point(0, -3, -3), point(1, -2, -2.9), point(2, -4, -3.1), point(3, null, null)];
    expect(median(points.map((p) => p.before))).toBe(-3);
    const b = brightnessChart(points);
    expect(b.xs).toEqual([1, 2, 3, 4]);
    expect(b.series.map((s) => s.key)).toEqual(['before', 'trend', 'after']);
    expect(b.series[0].values).toEqual([0, 1, -1, null]);
    expect(b.series[2].strong).toBe(true);
    expect(b.series[1].dashed).toBe(true);
    const red = colourChart(points, 'red');
    expect(red.series[0].values[0]).toBeCloseTo(0);
    expect(red.series[1].values[0]).toBeCloseTo(-0.1);
  });

  it('charts the camera path and where the frames end up', () => {
    const points: StabilizationPoint[] = [
      { sequence_index: 0, path: [0, 0], smoothed: [0.001, 0], output: [0.001, 0], measured: false },
      { sequence_index: 1, path: [0.01, -0.02], smoothed: [0.002, 0], output: [0.002, -0.01], measured: true }
    ];
    expect(pathChart(points, 0).series[0].values).toEqual([0, 0.01]);
    expect(pathChart(points, 1).series[1].values).toEqual([0, -0.01]);
  });

  it('formats stops with their sign', () => {
    setLocale('en');
    expect(formatStops(0.35)).toBe('+0.35 EV');
    expect(formatStops(-1)).toBe('-1 EV');
    expect(formatStops(0)).toBe('0 EV');
  });
});
