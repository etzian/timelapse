import { describe, expect, it } from 'vitest';

import { CODECS, EXPORT_FAILURES, EXPORT_PRESETS, QUALITIES, RESOLUTIONS, durationSeconds, exportFailureText, formatBytes, matchExportPreset, outputFrames, presetAvailable, summarize, summaryText, supports } from './export';
import type { FfmpegInfo } from './ipc/app';
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

describe('export presets and summary', () => {
  const all: FfmpegInfo = { available: true, av1: true, h264: true, hevc: true } as FfmpegInfo;

  it('matches the preset the values equal, else custom', () => {
    for (const p of EXPORT_PRESETS) expect(matchExportPreset(p)).toBe(p.key);
    expect(matchExportPreset({ codec: 'h264', resolution: '1080', quality: 'high' })).toBe('custom');
  });

  it('treats unknown values as custom', () => {
    expect(matchExportPreset({ codec: 'vp9', resolution: '1080', quality: 'standard' })).toBe('custom');
    expect(matchExportPreset({ codec: '', resolution: '', quality: '' })).toBe('custom');
  });

  it('presets only use known codecs, sizes and qualities', () => {
    for (const p of EXPORT_PRESETS) {
      expect(CODECS).toContain(p.codec);
      expect(QUALITIES).toContain(p.quality);
      expect(RESOLUTIONS.map((r) => r.key)).toContain(p.resolution);
    }
  });

  it('refuses a preset whose codec is unavailable', () => {
    const share = EXPORT_PRESETS.find((p) => p.key === 'share')!;
    expect(presetAvailable(all, share)).toBe(true);
    expect(presetAvailable({ ...all, h264: false }, share)).toBe(false);
    expect(presetAvailable({ ...all, available: false }, share)).toBe(false);
    expect(presetAvailable(null, share)).toBe(false);
  });

  it('summarizes photographs, frames and seconds', () => {
    expect(summarize(300, 30, 2)).toEqual({ photos: 300, frames: 150, seconds: 5 });
    expect(summarize(7, 24, 1.5)).toEqual({ photos: 7, frames: 5, seconds: 5 / 24 });
  });

  it('gives zero frames for zero, negative or invalid input', () => {
    expect(summarize(0, 30, 1)).toEqual({ photos: 0, frames: 0, seconds: 0 });
    expect(summarize(-3, 30, 1).frames).toBe(0);
    expect(summarize(Number.NaN, 30, 1).frames).toBe(0);
    expect(summarize(10, 30, 0).frames).toBe(0);
    expect(summarize(10, 0, 1).seconds).toBe(0);
  });

  it('writes the sentence in both locales, and the empty one for no frames', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      const line = summaryText(300, 30, 2);
      expect(line).not.toBe('summary-line');
      expect(line).toContain('150');
      expect(summaryText(0, 30, 1)).not.toBe('summary-empty');
      expect(summaryText(0, 30, 1)).not.toBe(line);
    }
    setLocale('en');
    expect(summaryText(1, 30, 1)).toMatch(/^1 photograph becomes 1 frame,/);
  });
});
