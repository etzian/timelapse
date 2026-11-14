import { describe, expect, it } from 'vitest';

import { setLocale, t } from './i18n';
import { CONFIDENCE_KEYS, GRANULARITY_KEYS, SOURCE_KEYS, barPercent, formatBucket, formatDuration } from './timeline';

describe('timeline helpers', () => {
  it('maps every data value to a message present in both locales', () => {
    const ids = [...Object.values(SOURCE_KEYS), ...Object.values(CONFIDENCE_KEYS), ...Object.values(GRANULARITY_KEYS)];
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const id of ids) expect(t(id)).not.toBe(id);
    }
  });

  it('reports a backfill pass with plurals in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      const none = t('timeline-backfill-result', { resolved: 3, unresolved: 0 });
      const some = t('timeline-backfill-result', { resolved: 3, unresolved: 2 });
      expect(none).toContain('3');
      expect(none).not.toContain(';');
      expect(some).toContain('2');
    }
  });

  it('labels buckets in UTC at each granularity', () => {
    setLocale('en');
    expect(formatBucket('2024-01', 'month')).toContain('2024');
    expect(formatBucket('2024-01-15', 'day')).toContain('15');
    expect(formatBucket('2024-01-15T23', 'hour')).toContain('15');
    expect(formatBucket('garbage', 'day')).toBe('garbage');
  });

  it('picks the largest sensible unit for a duration', () => {
    setLocale('en');
    expect(formatDuration(250_200)).toContain('day');
    expect(formatDuration(4_200)).toContain('hour');
    expect(formatDuration(600)).toContain('minute');
    expect(formatDuration(5)).toContain('second');
  });

  it('scales bars to the largest bucket and never hides a non-empty one', () => {
    expect(barPercent(5, 10)).toBe(50);
    expect(barPercent(1, 1000)).toBe(1);
    expect(barPercent(0, 0)).toBe(0);
  });
});
