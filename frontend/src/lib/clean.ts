/**
 * Presentation helpers for the Phase 3 cleaning page. Data values from the core
 * (reason kinds, statuses, failure kinds) map to Fluent ids here; nothing in this
 * file is user-visible text.
 */
import { formatNumber, t } from './i18n';
import type { ImageStatus, Reason, Thresholds } from './ipc/app';

export const STATUS_KEYS: Record<ImageStatus, string> = {
  eligible: 'clean-status-eligible',
  flagged: 'clean-status-flagged',
  rejected: 'clean-status-rejected',
  quarantined: 'clean-status-quarantined'
};

export const STATUS_VARIANT: Record<ImageStatus, 'success' | 'warning' | 'error' | 'info'> = {
  eligible: 'success',
  flagged: 'warning',
  rejected: 'error',
  quarantined: 'info'
};

/** Failure kinds a file operation can report (`crates/core/src/files`). */
export const FILE_FAILURES = [
  'unknown-image',
  'not-rejected',
  'not-quarantined',
  'unsafe-path',
  'source-unreadable',
  'changed-since-import',
  'destination-exists',
  'move-failed',
  'verify-failed',
  'source-in-use',
  'record-failed',
  'interrupted',
  'inconsistent'
] as const;

export function fileFailureText(kind: string): string {
  return (FILE_FAILURES as readonly string[]).includes(kind) ? t(`files-failure-${kind}`) : t('files-failure-other');
}

/** Analysis failure kinds (`crates/core/src/clean/analysis.rs`). */
export function analysisFailureText(kind: string): string {
  return kind === 'unreadable' ? t('clean-failure-unreadable') : t('clean-failure-undecodable');
}

const percent = (v: number) => formatNumber(v, { style: 'percent', maximumFractionDigits: 0 });
const luma = (v: number) => formatNumber(v, { maximumFractionDigits: 0 });

/** One reason as a sentence, numbers through `Intl`. */
export function reasonText(r: Reason): string {
  switch (r.kind) {
    case 'mean-below':
      return t('clean-reason-mean-below', { mean: luma(r.mean), min: luma(r.min) });
    case 'too-many-dark-pixels':
      return t('clean-reason-too-many-dark-pixels', { fraction: percent(r.fraction), max: percent(r.max) });
    case 'xmp-rejected':
      return t('clean-reason-xmp-rejected');
    case 'analysis-failed':
      return t('clean-reason-analysis-failed', { failure: analysisFailureText(r.failure) });
  }
}

export { percent as formatPercent, luma as formatLuma };

/** Form values (strings, as typed) → thresholds, or null when any field is invalid. */
export function parseThresholds(form: { minMean: string; darkLevel: string; maxDarkPercent: string }, flagXmp: boolean): Thresholds | null {
  const n = (s: string) => (s.trim() === '' ? Number.NaN : Number(s.replace(',', '.')));
  const minMean = n(form.minMean);
  const darkLevel = n(form.darkLevel);
  const maxDark = n(form.maxDarkPercent);
  const inRange = (v: number, hi: number) => Number.isFinite(v) && v >= 0 && v <= hi;
  if (!inRange(minMean, 255) || !inRange(darkLevel, 255) || !Number.isInteger(darkLevel) || !inRange(maxDark, 100)) return null;
  return { min_mean_luma: minMean, dark_level: darkLevel, max_dark_fraction: maxDark / 100, flag_xmp_rejected: flagXmp };
}

export function thresholdsToForm(th: Thresholds): { minMean: string; darkLevel: string; maxDarkPercent: string } {
  return {
    minMean: String(th.min_mean_luma),
    darkLevel: String(th.dark_level),
    maxDarkPercent: String(Math.round(th.max_dark_fraction * 1000) / 10)
  };
}

/** Run `tasks` with at most `limit` in flight (thumbnail loading). */
export async function limited<T>(tasks: (() => Promise<T>)[], limit: number): Promise<PromiseSettledResult<T>[]> {
  const results: PromiseSettledResult<T>[] = new Array(tasks.length);
  let next = 0;
  async function worker(): Promise<void> {
    while (next < tasks.length) {
      const i = next++;
      try {
        results[i] = { status: 'fulfilled', value: await tasks[i]() };
      } catch (reason) {
        results[i] = { status: 'rejected', reason };
      }
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, tasks.length) }, worker));
  return results;
}
