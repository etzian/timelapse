import { open } from '@tauri-apps/plugin-dialog';

import { invoke } from './invoke';

export interface AppInfo {
  version: string;
}

export interface ImportReport {
  scanned_count: number;
  hash_computed: number;
  duplicates_found: number;
  duplicate_groups: string[][];
  new_rows: number;
  unchanged: number;
  skipped_unreadable: number;
}

export interface ImageRow {
  id: number;
  original_path: string;
  relative_path: string;
  filename: string;
  extension: string;
  file_size: number;
  modified_at: number | null;
  import_status: string;
}

export interface IntegrityResult {
  missing_files: string[];
  duplicates: string[][];
  total_images: number;
}

/** Long-running command (scans + hashes many files); well over the default 10s. */
const IMPORT_TIMEOUT_MS = 180_000;

export const appInfo = () => invoke<AppInfo>('app_info');

/** Open a folder picker. Returns the chosen path, or null if the user cancelled. */
export function pickFolder(): Promise<string | null> {
  return open({ directory: true });
}

/** Import `root` (scan → SHA-256 → dedup → upsert into SQLite). Long timeout for large trees. */
export const importRoot = (root: string) => invoke<ImportReport>('import_folder', { root }, IMPORT_TIMEOUT_MS);

export const listImages = (root: string) => invoke<ImageRow[]>('list_images', { root });
export const integrityCheck = (root: string) => invoke<IntegrityResult>('integrity_check', { root });

// ---- Phase 2 — EXIF & timeline ----------------------------------------------------

/** Stable data values from `images.timestamp_source` (mapped to Fluent keys in the UI). */
export type TimestampSource = 'exif_datetime_original' | 'exif_datetime_digitized' | 'filename' | 'filesystem';
export type Confidence = 'high' | 'medium' | 'low';
export type Granularity = 'hour' | 'day' | 'month';

export interface SourceCounts {
  exif_datetime_original: number;
  exif_datetime_digitized: number;
  filename: number;
  filesystem: number;
}

export interface TimelineSummary {
  total: number;
  resolved: number;
  unresolved: number;
  by_source: SourceCounts;
  low_confidence: number;
  /** Resolved from a clock with no stated time zone; UTC was assumed. */
  tz_assumed: number;
}

export interface BackfillReport {
  newly_resolved: number;
  unresolved_this_run: number;
  summary: TimelineSummary;
}

export interface TimelineEntry {
  id: number;
  relative_path: string;
  /** `YYYY-MM-DDTHH:MM:SS.mmmZ`, or null when unresolved. */
  normalized_datetime: string | null;
  epoch_ms: number | null;
  source: TimestampSource | null;
  confidence: Confidence | null;
  offset_minutes: number | null;
  tz_assumed: boolean;
}

export interface DistributionBucket {
  /** UTC key: `YYYY-MM-DDTHH`, `YYYY-MM-DD` or `YYYY-MM` (a prefix of the instants it counts). */
  bucket: string;
  count: number;
}

export interface TimelineGap {
  before: TimelineEntry;
  after: TimelineEntry;
  seconds: number;
  /** The two sides come from different sources or zone assumptions: maybe metadata, not a real pause. */
  source_changed: boolean;
}

export interface GapReport {
  median_interval_seconds: number | null;
  threshold_seconds: number;
  gaps: TimelineGap[];
}

export interface GapOptions {
  factor?: number;
  minSeconds?: number;
}

/** Reads EXIF from every pending image; same long timeout as the import. */
export const backfillMetadata = (root: string) => invoke<BackfillReport>('backfill_metadata', { root }, IMPORT_TIMEOUT_MS);

export const getTimeline = (root: string) => invoke<TimelineEntry[]>('get_timeline', { root });

export const timelineDistribution = (root: string, granularity: Granularity, from?: string, to?: string) =>
  invoke<DistributionBucket[]>('timeline_distribution', { root, granularity, from: from ?? null, to: to ?? null });

export const detectTimelineGaps = (root: string, options: GapOptions = {}) =>
  invoke<GapReport>('detect_timeline_gaps', {
    root,
    factor: options.factor ?? null,
    minSeconds: options.minSeconds ?? null
  });
