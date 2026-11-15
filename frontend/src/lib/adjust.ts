/**
 * Presentation helpers for the Phase 8 Adjust page. Data values from the core (settings,
 * refusal reasons, chart points) map to Fluent ids and chart series here; nothing in this
 * file is user-visible text by itself.
 */
import { formatNumber, t } from './i18n';
import type { DeflickerConfig, FlickerPoint, StabilizationPoint, StabilizeConfig } from './ipc/app';

export const DEFAULT_DEFLICKER: DeflickerConfig = { strength: 0.8, window: 15, colour: true };
export const DEFAULT_STABILIZE: StabilizeConfig = { smoothing: 31, max_crop: 0.05 };

/** Choices offered (the core clamps anything else). */
export const STRENGTHS: readonly number[] = [0.25, 0.5, 0.8, 1];
export const WINDOWS: readonly number[] = [5, 9, 15, 31, 61, 121];
export const SMOOTHINGS: readonly number[] = [9, 15, 31, 61, 121];
export const CROPS: readonly number[] = [0.01, 0.02, 0.03, 0.04, 0.05];

/** Reasons of the core's `Error::Adjust { reason }`. */
export const ADJUST_FAILURES = ['busy', 'empty-run', 'unknown-adjustment', 'adjustment-mismatch', 'unknown-frame'] as const;

export function adjustFailureText(reason: string | null): string {
  const known = reason !== null && (ADJUST_FAILURES as readonly string[]).includes(reason);
  return t('error-adjust', { reason: known ? reason : 'other' });
}

/** A value in stops, signed, e.g. "+0.35 EV". */
export function formatStops(v: number): string {
  return t('adjust-stops', { value: formatNumber(v, { maximumFractionDigits: 2, signDisplay: 'exceptZero' }) });
}

/** A fraction of the frame as a percentage, e.g. "2.4 %". */
export function formatFrame(v: number): string {
  return formatNumber(v, { style: 'percent', maximumFractionDigits: 1, signDisplay: 'exceptZero' });
}

export interface ChartData {
  xs: number[];
  series: { key: string; label: string; color: string; dashed?: boolean; strong?: boolean; values: (number | null)[] }[];
}

/** Median of the present values (0 when there are none). */
export function median(values: (number | null)[]): number {
  const v = values.filter((x): x is number => x !== null && Number.isFinite(x)).sort((a, b) => a - b);
  if (v.length === 0) return 0;
  const m = Math.floor(v.length / 2);
  return v.length % 2 ? v[m] : (v[m - 1] + v[m]) / 2;
}

/** Each value minus `base`, so 0 reads as "typical for this sequence". */
const relative = (values: (number | null)[], base: number) => values.map((v) => (v === null ? null : v - base));

const BEFORE = 'var(--app-chart-before)';
const AFTER = 'var(--app-chart-after)';
const TREND = 'var(--app-chart-trend)';

/** Brightness before, its trend, and after, in stops from the sequence's median. */
export function brightnessChart(points: FlickerPoint[]): ChartData {
  const base = median(points.map((p) => p.before));
  return {
    xs: points.map((p) => p.sequence_index + 1),
    series: [
      { key: 'before', label: t('adjust-series-before'), color: BEFORE, values: relative(points.map((p) => p.before), base) },
      { key: 'trend', label: t('adjust-series-trend'), color: TREND, dashed: true, values: relative(points.map((p) => p.trend), base) },
      { key: 'after', label: t('adjust-series-after'), color: AFTER, strong: true, values: relative(points.map((p) => p.after), base) }
    ]
  };
}

/** One colour's balance against green, before and after, in stops from its median. */
export function colourChart(points: FlickerPoint[], channel: 'red' | 'blue'): ChartData {
  const before = points.map((p) => p[`${channel}_before`]);
  const base = median(before);
  return {
    xs: points.map((p) => p.sequence_index + 1),
    series: [
      { key: 'before', label: t('adjust-series-before'), color: BEFORE, values: relative(before, base) },
      { key: 'after', label: t('adjust-series-after'), color: AFTER, strong: true, values: relative(points.map((p) => p[`${channel}_after`]), base) }
    ]
  };
}

/** The camera path on one axis as measured, and where the frames end up (fraction). */
export function pathChart(points: StabilizationPoint[], axis: 0 | 1): ChartData {
  return {
    xs: points.map((p) => p.sequence_index + 1),
    series: [
      { key: 'before', label: t('adjust-series-path'), color: BEFORE, values: points.map((p) => p.path[axis]) },
      { key: 'after', label: t('adjust-series-stabilized'), color: AFTER, strong: true, values: points.map((p) => p.output[axis]) }
    ]
  };
}
