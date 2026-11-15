/**
 * Presentation helpers for the Phase 5 export page. The duration maths mirrors
 * `crates/core/src/export/settings.rs` exactly, so the estimate matches the file.
 */
import { formatNumber, t } from './i18n';
import type { Codec, FfmpegInfo, Quality } from './ipc/app';
import { formatDuration } from './timeline';

export const CODECS: readonly Codec[] = ['av1', 'h264', 'hevc'];
export const QUALITIES: readonly Quality[] = ['high', 'standard', 'small'];
export const RESOLUTIONS = [
  { key: '2160', width: 3840, height: 2160 },
  { key: '1440', width: 2560, height: 1440 },
  { key: '1080', width: 1920, height: 1080 },
  { key: '720', width: 1280, height: 720 }
] as const;
export type ResolutionKey = (typeof RESOLUTIONS)[number]['key'];

/** Failure kinds from the core's `Error::Export { reason }` and export records. */
export const EXPORT_FAILURES = [
  'ffmpeg-missing',
  'encoder-unavailable',
  'invalid-settings',
  'empty-run',
  'busy',
  'cancelled',
  'ffmpeg-failed',
  'invalid-output',
  'output-exists',
  'image-unreadable',
  'write-failed',
  'interrupted',
  'unknown-export',
  'record-write-failed',
  'adjustment-mismatch',
  'adjustment-stale'
] as const;

export function exportFailureText(kind: string | null): string {
  return t('error-export', { reason: kind && (EXPORT_FAILURES as readonly string[]).includes(kind) ? kind : 'other' });
}

export function supports(info: FfmpegInfo | null, codec: Codec): boolean {
  return !!info && info.available && info[codec];
}

/** `ceil(n / speed)` output frames (photographs held or skipped by speed, never by FPS). */
export function outputFrames(selected: number, speed: number): number {
  if (selected <= 0 || !(speed > 0)) return 0;
  return Math.max(1, Math.ceil(selected / speed - 1e-9));
}

export function durationSeconds(selected: number, fps: number, speed: number): number {
  return fps > 0 ? outputFrames(selected, speed) / fps : 0;
}

export function formatBytes(bytes: number): string {
  const units = ['byte', 'kilobyte', 'megabyte', 'gigabyte'] as const;
  let v = bytes;
  let i = 0;
  while (v >= 1000 && i < units.length - 1) {
    v /= 1000;
    i++;
  }
  return formatNumber(v, { style: 'unit', unit: units[i], unitDisplay: 'short', maximumFractionDigits: i === 0 ? 0 : 1 });
}

// ---- Presets and the plain-language summary ------------------------------------------

export type ExportPresetKey = 'share' | 'archive' | 'master';

export interface ExportPreset {
  key: ExportPresetKey;
  codec: Codec;
  resolution: ResolutionKey;
  quality: Quality;
}

/** Presets only set parameters that already exist; frame rate, speed and adjustments are left alone. */
export const EXPORT_PRESETS: readonly ExportPreset[] = [
  { key: 'share', codec: 'h264', resolution: '1080', quality: 'standard' },
  { key: 'archive', codec: 'av1', resolution: '1080', quality: 'standard' },
  { key: 'master', codec: 'hevc', resolution: '2160', quality: 'high' }
];

/** The preset these values equal exactly, else `'custom'` (also for values this build does not know). */
export function matchExportPreset(values: { codec: string; resolution: string; quality: string }): ExportPresetKey | 'custom' {
  const hit = EXPORT_PRESETS.find((p) => p.codec === values.codec && p.resolution === values.resolution && p.quality === values.quality);
  return hit ? hit.key : 'custom';
}

/** Whether the encoder a preset needs is available on this Mac. */
export function presetAvailable(info: FfmpegInfo | null, preset: ExportPreset): boolean {
  return supports(info, preset.codec);
}

export interface ExportSummary {
  photos: number;
  frames: number;
  seconds: number;
}

/** Photographs → frames → seconds at the chosen rate. Invalid or empty input yields zero frames. */
export function summarize(photos: number, fps: number, speed: number): ExportSummary {
  const p = Number.isFinite(photos) && photos > 0 ? Math.floor(photos) : 0;
  const frames = outputFrames(p, speed);
  return { photos: p, frames, seconds: durationSeconds(p, fps, speed) };
}

/** The summary as one sentence, or the "nothing selected" sentence when there are no frames. */
export function summaryText(photos: number, fps: number, speed: number): string {
  const s = summarize(photos, fps, speed);
  if (s.frames === 0) return t('summary-empty');
  return t('summary-line', {
    photos: s.photos,
    frames: s.frames,
    duration: formatDuration(s.seconds),
    fps: formatNumber(fps)
  });
}
