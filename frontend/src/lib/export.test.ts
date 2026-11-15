import { describe, expect, it } from 'vitest';

import { CODECS, EXPORT_FAILURES, QUALITIES, durationSeconds, exportFailureText, formatBytes, outputFrames, supports } from './export';
import { setLocale, t } from './i18n';

describe('export helpers', () => {
  it('matches the core duration maths', () => {
    expect(outputFrames(24, 1)).toBe(24);
    expect(outputFrames(24, 2)).toBe(12);
    expect(outputFrames(3, 0.5)).toBe(6);
    expect(outputFrames(7, 1.5)).toBe(5);
    expect(outputFrames(0, 1)).toBe(0);
    expect(durationSeconds(300, 30, 2)).toBe(5);
    expect(durationSeconds(24, 12, 1)).toBe(2);
  });

  it('has text for every codec, quality and failure in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const c of CODECS) expect(t(`export-codec-${c}`)).not.toBe(`export-codec-${c}`);
      for (const q of QUALITIES) expect(t(`export-quality-${q}`)).not.toBe(`export-quality-${q}`);
      const texts = new Set(EXPORT_FAILURES.map((f) => exportFailureText(f)));
      expect(texts.size).toBe(EXPORT_FAILURES.length);
      expect(exportFailureText('something-new')).toBe(exportFailureText(null));
    }
  });

  it('offers a tier only when the encoder is present', () => {
    const info = { available: true, version: '9.0.2', lgpl: true, av1: true, h264: false, hevc: true };
    expect(supports(info, 'av1')).toBe(true);
    expect(supports(info, 'h264')).toBe(false);
    expect(supports({ ...info, available: false }, 'av1')).toBe(false);
    expect(supports(null, 'av1')).toBe(false);
  });

  it('formats file sizes', () => {
    setLocale('en');
    expect(formatBytes(512)).toContain('512');
    expect(formatBytes(2_500_000)).toContain('2.5');
  });
});
