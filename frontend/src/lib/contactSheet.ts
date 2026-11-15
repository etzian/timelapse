/**
 * Presentation helpers for Phase 7 contact sheets and the timeline's periods. Data values
 * from the core (sources, layouts, statuses, refusal reasons) map to Fluent ids here, and
 * the texts printed on the PDF are put together here from Fluent; nothing in this file is
 * user-visible text by itself.
 */
import { formatNumber, t } from './i18n';
import type { Granularity, SheetLabels, SheetLayout, SheetRecord, SheetSource } from './ipc/app';

export type SourceKind = SheetSource['kind'];
export const SOURCE_KINDS: readonly SourceKind[] = ['all', 'period', 'flagged', 'quarantined', 'sampling', 'export'];

export const MIN_CELLS = 1;
export const MAX_CELLS = 12;
export const GRID_OPTIONS: readonly number[] = Array.from({ length: MAX_CELLS }, (_, i) => i + 1);

export const DEFAULT_LAYOUT: SheetLayout = {
  page_size: 'a4',
  orientation: 'portrait',
  columns: 4,
  rows: 6,
  show_filename: true,
  show_timestamp: true
};

/** Reasons of the core's `Error::ContactSheet { reason }` and of a failed sheet. */
export const SHEET_FAILURES = ['busy', 'empty', 'too-many', 'invalid-period', 'unknown-export', 'write-failed', 'output-exists', 'interrupted'] as const;

export function sheetFailureText(reason: string | null): string {
  const known = reason !== null && (SHEET_FAILURES as readonly string[]).includes(reason);
  return t('error-contact-sheet', { reason: known ? reason : 'other' });
}

/** A period bound as the `datetime-local` input shows it (`YYYY-MM-DDTHH:MM`, UTC). */
export type PeriodBound = string;

/** `YYYY-MM-DDTHH:MM` of a UTC instant. */
export function toBound(epochMs: number): PeriodBound {
  return new Date(epochMs).toISOString().slice(0, 16);
}

/** The UTC instant of a bound, or NaN. */
export function boundMs(bound: string): number {
  return /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(bound) ? Date.parse(`${bound}:00Z`) : Number.NaN;
}

/** `[from, to)` covered by a distribution bucket (keys are UTC, see `DistributionBucket`). */
export function bucketPeriod(key: string, granularity: Granularity): { from: PeriodBound; to: PeriodBound } | null {
  const start =
    granularity === 'hour' ? `${key}:00:00Z` : granularity === 'month' ? `${key}-01T00:00:00Z` : `${key}T00:00:00Z`;
  const at = Date.parse(start);
  if (Number.isNaN(at)) return null;
  const d = new Date(at);
  if (granularity === 'hour') d.setUTCHours(d.getUTCHours() + 1);
  else if (granularity === 'day') d.setUTCDate(d.getUTCDate() + 1);
  else if (granularity === 'week') d.setUTCDate(d.getUTCDate() + 7);
  else d.setUTCMonth(d.getUTCMonth() + 1);
  return { from: toBound(at), to: toBound(d.getTime()) };
}

/** The next finer granularity to zoom into, or null at the finest. */
export function finer(g: Granularity): Granularity | null {
  return g === 'month' ? 'day' : g === 'week' ? 'day' : g === 'day' ? 'hour' : null;
}

/** A period source for the core: the bounds are prefixes of the normalised times. */
export function periodSource(from: PeriodBound, to: PeriodBound): SheetSource | null {
  const a = boundMs(from);
  const b = boundMs(to);
  return Number.isNaN(a) || Number.isNaN(b) || a >= b ? null : { kind: 'period', from, to };
}

/** One line saying what a sheet shows, e.g. in the history or on the PDF. */
export function sourceText(source: SheetSource, formatBound: (b: string) => string = (b) => b): string {
  switch (source.kind) {
    case 'period':
      return t('sheets-source-period-line', { from: formatBound(source.from), to: formatBound(source.to) });
    case 'sampling':
      return t('sheets-source-sampling-line', { id: source.run });
    case 'export':
      return t('sheets-source-export-line', { id: source.export });
    default:
      return t(`sheets-source-${source.kind}`);
  }
}

/** The texts printed on every page, in the current language. */
export function sheetLabels(source: SheetSource, photos: number, formatBound?: (b: string) => string): SheetLabels {
  return {
    title: t('sheets-pdf-title'),
    source: t('sheets-pdf-source', { source: sourceText(source, formatBound), count: photos, photos: formatNumber(photos) }),
    page: t('sheets-pdf-page', { page: '{page}', pages: '{pages}' }),
    note: t('sheets-pdf-note'),
    unreadable: t('sheets-pdf-unreadable')
  };
}

export function sheetStatusVariant(s: SheetRecord['status']): 'success' | 'info' | 'default' | 'error' {
  return s === 'completed' ? 'success' : s === 'running' ? 'info' : s === 'cancelled' ? 'default' : 'error';
}

/** The layout clamped to the grid bounds the core accepts. */
export function clampLayout(l: SheetLayout): SheetLayout {
  const clamp = (n: number) => Math.min(MAX_CELLS, Math.max(MIN_CELLS, Math.round(n) || MIN_CELLS));
  return { ...l, columns: clamp(l.columns), rows: clamp(l.rows) };
}
