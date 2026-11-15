/**
 * Presentation helpers for the Phase 2 timeline. Data values from the core
 * (`timestamp_source`, `timestamp_confidence`, bucket keys) map to Fluent ids
 * and `Intl` formats here; nothing in this file is user-visible text.
 */
import { formatDate, formatNumber, t } from './i18n';
import type { Confidence, Granularity, TimestampSource } from './ipc/app';

export const SOURCE_KEYS: Record<TimestampSource, string> = {
  exif_datetime_original: 'timeline-source-exif-original',
  exif_datetime_digitized: 'timeline-source-exif-digitized',
  filename: 'timeline-source-filename',
  filesystem: 'timeline-source-filesystem'
};

export const SOURCE_ORDER: readonly TimestampSource[] = [
  'exif_datetime_original',
  'exif_datetime_digitized',
  'filename',
  'filesystem'
];

export const CONFIDENCE_KEYS: Record<Confidence, string> = {
  high: 'timeline-confidence-high',
  medium: 'timeline-confidence-medium',
  low: 'timeline-confidence-low'
};

export const CONFIDENCE_VARIANT: Record<Confidence, 'success' | 'info' | 'warning'> = {
  high: 'success',
  medium: 'info',
  low: 'warning'
};

export const GRANULARITY_KEYS: Record<Granularity, string> = {
  hour: 'timeline-granularity-hour',
  day: 'timeline-granularity-day',
  week: 'timeline-granularity-week',
  month: 'timeline-granularity-month'
};

/**
 * Normalised instants are UTC, and a clock with no recorded zone was *assumed*
 * UTC: showing them in UTC shows exactly what was recorded, without inventing a zone.
 */
const UTC = { timeZone: 'UTC' } as const;

export function formatInstant(epochMs: number): string {
  return formatDate(epochMs, { ...UTC, dateStyle: 'medium', timeStyle: 'medium' });
}

/** A distribution bucket key (`YYYY-MM-DDTHH`, `YYYY-MM-DD`, `YYYY-MM`; a week's Monday) as a label. */
export function formatBucket(key: string, granularity: Granularity): string {
  const at = Date.parse(
    granularity === 'month' ? `${key}-01T00:00:00Z` : granularity === 'day' || granularity === 'week' ? `${key}T00:00:00Z` : `${key}:00:00Z`
  );
  if (Number.isNaN(at)) return key;
  if (granularity === 'week') return t('timeline-week-of', { date: formatDate(at, { ...UTC, dateStyle: 'medium' }) });
  if (granularity === 'month') return formatDate(at, { ...UTC, year: 'numeric', month: 'long' });
  if (granularity === 'day') return formatDate(at, { ...UTC, dateStyle: 'medium' });
  return formatDate(at, { ...UTC, dateStyle: 'medium', timeStyle: 'short' });
}

/** A duration in the largest sensible unit, e.g. "2.9 days", "70 minutes". */
export function formatDuration(seconds: number): string {
  const [value, unit] =
    seconds >= 86_400
      ? [seconds / 86_400, 'day']
      : seconds >= 3_600
        ? [seconds / 3_600, 'hour']
        : seconds >= 60
          ? [seconds / 60, 'minute']
          : [seconds, 'second'];
  return formatNumber(value, { style: 'unit', unit, unitDisplay: 'long', maximumFractionDigits: 1 });
}

/** Bar width (0–100) of `count` relative to the largest bucket. */
export function barPercent(count: number, max: number): number {
  return max > 0 ? Math.max(1, Math.round((count / max) * 100)) : 0;
}
