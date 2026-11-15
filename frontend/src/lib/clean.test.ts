import { describe, expect, it } from 'vitest';

import { FILE_FAILURES, STATUS_KEYS, fileFailureText, limited, parseThresholds, reasonText, thresholdsToForm } from './clean';
import { setLocale, t } from './i18n';
import { toBase64 } from './ipc/app';
import type { Reason } from './ipc/app';

const reasons: Reason[] = [
  { kind: 'mean-below', mean: 12.4, min: 40 },
  { kind: 'too-many-dark-pixels', fraction: 0.82, max: 0.6, level: 30 },
  { kind: 'xmp-rejected' },
  { kind: 'analysis-failed', failure: 'undecodable' }
];

describe('cleaning helpers', () => {
  it('has a message for every status, reason and failure in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const id of Object.values(STATUS_KEYS)) expect(t(id)).not.toBe(id);
      for (const r of reasons) expect(reasonText(r)).not.toMatch(/^clean-/);
      for (const f of FILE_FAILURES) expect(fileFailureText(f)).not.toMatch(/^files-/);
      expect(fileFailureText('something-new')).not.toMatch(/^files-/);
    }
  });

  it('formats reason numbers through Intl', () => {
    setLocale('en');
    expect(reasonText(reasons[1])).toContain('82%');
    setLocale('fr-CA');
    expect(reasonText(reasons[1])).toMatch(/82\s%/);
  });

  it('parses threshold forms and rejects invalid input', () => {
    expect(parseThresholds({ minMean: '40', darkLevel: '30', maxDarkPercent: '60' }, true)).toEqual({
      min_mean_luma: 40,
      dark_level: 30,
      max_dark_fraction: 0.6,
      flag_xmp_rejected: true
    });
    expect(parseThresholds({ minMean: '40,5', darkLevel: '30', maxDarkPercent: '60' }, false)?.min_mean_luma).toBe(40.5);
    expect(parseThresholds({ minMean: '', darkLevel: '30', maxDarkPercent: '60' }, true)).toBeNull();
    expect(parseThresholds({ minMean: '40', darkLevel: '30.5', maxDarkPercent: '60' }, true)).toBeNull();
    expect(parseThresholds({ minMean: '40', darkLevel: '30', maxDarkPercent: '101' }, true)).toBeNull();
    expect(thresholdsToForm({ min_mean_luma: 40, dark_level: 30, max_dark_fraction: 0.6, flag_xmp_rejected: true })).toEqual({
      minMean: '40',
      darkLevel: '30',
      maxDarkPercent: '60'
    });
  });

  it('limits concurrency and keeps result order', async () => {
    let inFlight = 0;
    let peak = 0;
    const tasks = Array.from({ length: 10 }, (_, i) => async () => {
      inFlight++;
      peak = Math.max(peak, inFlight);
      await new Promise((r) => setTimeout(r, 1));
      inFlight--;
      if (i === 3) throw new Error('x');
      return i;
    });
    const out = await limited(tasks, 3);
    expect(peak).toBeLessThanOrEqual(3);
    expect(out[0]).toEqual({ status: 'fulfilled', value: 0 });
    expect(out[3].status).toBe('rejected');
  });

  it('base64-encodes thumbnail bytes', () => {
    expect(toBase64(new Uint8Array([0xff, 0xd8, 0xff]))).toBe('/9j/');
  });

  it('has every bulk-decision text, singular and plural, in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const d of ['accept', 'reject', 'clear'] as const) {
        for (const part of ['title', 'body', 'action']) {
          const id = `clean-bulk-${d}-${part}`;
          expect(t(id, { count: 1 })).not.toBe(id);
          expect(t(id, { count: 1 })).not.toBe(t(id, { count: 3 }));
        }
      }
      expect(t('clean-selected', { count: 2 })).toContain('2');
    }
  });
});
