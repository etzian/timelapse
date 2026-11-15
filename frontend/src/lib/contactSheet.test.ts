import { describe, expect, it } from 'vitest';

import {
  SHEET_FAILURES,
  SOURCE_KINDS,
  bucketPeriod,
  clampLayout,
  DEFAULT_LAYOUT,
  finer,
  periodSource,
  sheetFailureText,
  sheetLabels,
  sourceText
} from './contactSheet';
import { EXPORT_FAILURES, exportFailureText } from './export';
import { setLocale, t } from './i18n';
import type { SheetSource } from './ipc/app';
import { formatBucket } from './timeline';

const SOURCES: SheetSource[] = [
  { kind: 'all' },
  { kind: 'period', from: '2024-01-15T00:00', to: '2024-01-16T00:00' },
  { kind: 'flagged' },
  { kind: 'quarantined' },
  { kind: 'sampling', run: 3 },
  { kind: 'export', export: 7 }
];

describe('contact sheet helpers', () => {
  it('has text for every source, status, refusal and new export failure in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const k of SOURCE_KINDS) expect(t(`sheets-source-${k}`)).not.toBe(`sheets-source-${k}`);
      const lines = new Set(SOURCES.map((s) => sourceText(s)));
      expect(lines.size).toBe(SOURCES.length);
      for (const s of ['running', 'completed', 'failed', 'cancelled', 'interrupted']) {
        expect(t(`sheets-status-${s}`)).not.toBe(`sheets-status-${s}`);
      }
      const texts = new Set(SHEET_FAILURES.map((f) => sheetFailureText(f)));
      expect(texts.size).toBe(SHEET_FAILURES.length);
      expect(sheetFailureText('something-new')).toBe(sheetFailureText(null));
      for (const f of EXPORT_FAILURES) expect(exportFailureText(f)).not.toBe(exportFailureText(null));
      expect(formatBucket('2024-01-15', 'week')).toContain('2024');
    }
  });

  it('prints translated labels and keeps the page placeholders for the core', () => {
    setLocale('fr-CA');
    const fr = sheetLabels({ kind: 'flagged' }, 1204);
    expect(fr.title).toBe('Planche contact');
    expect(fr.source).toContain('Photographies signalées');
    expect(fr.page).toContain('{page}');
    expect(fr.page).toContain('{pages}');
    expect(fr.page).toContain(' de ');
    setLocale('en');
    const en = sheetLabels({ kind: 'sampling', run: 4 }, 1);
    expect(en.source).toContain('#4');
    expect(en.source).toMatch(/1 photograph$/);
    expect(en.page).toMatch(/of/);
    expect(en.unreadable).not.toBe(fr.unreadable);
  });

  it('agrees pages and photographs separately', () => {
    setLocale('en');
    expect(t('sheets-done', { pages: 1, count: 1, photos: '1', size: '2 kB' })).toBe('1 page, 1 photograph, 2 kB.');
    expect(t('sheets-history-line', { pages: 2, count: 30, photos: '30' })).toBe('2 pages, 30 photographs');
    setLocale('fr-CA');
    expect(t('sheets-done', { pages: 1, count: 1, photos: '1', size: '2 ko' })).toBe('1 page, 1 photographie, 2 ko.');
  });

  it('turns a bucket into the period it covers, in UTC', () => {
    expect(bucketPeriod('2024-01-15T10', 'hour')).toEqual({ from: '2024-01-15T10:00', to: '2024-01-15T11:00' });
    expect(bucketPeriod('2024-01-31', 'day')).toEqual({ from: '2024-01-31T00:00', to: '2024-02-01T00:00' });
    expect(bucketPeriod('2024-12-30', 'week')).toEqual({ from: '2024-12-30T00:00', to: '2025-01-06T00:00' });
    expect(bucketPeriod('2024-12', 'month')).toEqual({ from: '2024-12-01T00:00', to: '2025-01-01T00:00' });
    expect(bucketPeriod('not a key', 'day')).toBeNull();
    expect([finer('month'), finer('week'), finer('day'), finer('hour')]).toEqual(['day', 'day', 'hour', null]);
  });

  it('only accepts a period whose end comes after its start', () => {
    expect(periodSource('2024-01-15T00:00', '2024-01-16T00:00')).toEqual({ kind: 'period', from: '2024-01-15T00:00', to: '2024-01-16T00:00' });
    expect(periodSource('2024-01-16T00:00', '2024-01-15T00:00')).toBeNull();
    expect(periodSource('2024-01-15T00:00', '2024-01-15T00:00')).toBeNull();
    expect(periodSource('', '2024-01-15T00:00')).toBeNull();
  });

  it('clamps the grid as the core does', () => {
    expect(clampLayout({ ...DEFAULT_LAYOUT, columns: 0, rows: 40 })).toMatchObject({ columns: 1, rows: 12 });
    expect(clampLayout({ ...DEFAULT_LAYOUT, columns: Number.NaN, rows: 3 })).toMatchObject({ columns: 1, rows: 3 });
  });
});
