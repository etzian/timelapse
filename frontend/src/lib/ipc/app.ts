import { Channel } from '@tauri-apps/api/core';
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

// ---- Phase 3 — brightness analysis & cleaning ---------------------------------------

export type ImageStatus = 'eligible' | 'flagged' | 'rejected' | 'quarantined';
export type Decision = 'accept' | 'reject' | 'clear';

export interface Thresholds {
  /** Flag when mean luma (0–255) is below this. */
  min_mean_luma: number;
  /** A pixel is dark at or below this luma (0–255)… */
  dark_level: number;
  /** …and an image is flagged when more than this fraction (0–1) is dark. */
  max_dark_fraction: number;
  flag_xmp_rejected: boolean;
}

export interface StatusCounts {
  eligible: number;
  flagged: number;
  rejected: number;
  quarantined: number;
  unanalyzed: number;
}

export interface AnalyzeProgress {
  done: number;
  total: number;
}

export interface AnalyzeReport {
  analyzed: number;
  failed: number;
  counts: StatusCounts;
}

export interface ThresholdsApplied {
  thresholds: Thresholds;
  counts: StatusCounts;
}

export type Reason =
  | { kind: 'mean-below'; mean: number; min: number }
  | { kind: 'too-many-dark-pixels'; fraction: number; max: number; level: number }
  | { kind: 'xmp-rejected' }
  | { kind: 'analysis-failed'; failure: string };

export interface ReviewItem {
  id: number;
  relative_path: string;
  normalized_datetime: string | null;
  status: ImageStatus | null;
  user_override: boolean;
  reasons: Reason[];
  mean_luma: number | null;
  median_luma: number | null;
  dark_fraction: number | null;
  /** 32 bins of the luma histogram. */
  histogram: number[];
  xmp_rating: number | null;
  failure: string | null;
  /** Where the original is now while quarantined (null = at its original path). */
  current_path: string | null;
}

export interface OpOutcome {
  image_id: number;
  operation_id: number | null;
  /** Stable kind, shown via `files-failure-<kind>`. */
  failure: string | null;
}

export interface FileOperation {
  id: number;
  image_id: number | null;
  operation_type: 'quarantine' | 'restore';
  source_path: string;
  destination_path: string;
  status: 'pending' | 'completed' | 'failed';
  failure: string | null;
  created_at: number;
  completed_at: number | null;
}

/** Decoding a large collection takes minutes; progress arrives on the channel meanwhile. */
const ANALYZE_TIMEOUT_MS = 30 * 60_000;
const MOVE_TIMEOUT_MS = 10 * 60_000;

export function analyzeImages(root: string, onProgress: (p: AnalyzeProgress) => void): Promise<AnalyzeReport> {
  const channel = new Channel<AnalyzeProgress>();
  channel.onmessage = onProgress;
  return invoke<AnalyzeReport>('analyze_images', { root, onProgress: channel }, ANALYZE_TIMEOUT_MS);
}

export const getCleanSettings = (root: string) => invoke<Thresholds>('get_clean_settings', { root });
export const setCleanSettings = (root: string, thresholds: Thresholds) =>
  invoke<ThresholdsApplied>('set_clean_settings', { root, thresholds });
export const classifyImages = (root: string) => invoke<StatusCounts>('classify_images', { root });
export const getReview = (root: string, statuses: ImageStatus[] = []) => invoke<ReviewItem[]>('get_review', { root, statuses });
export const setDecision = (root: string, ids: number[], decision: Decision) =>
  invoke<StatusCounts>('set_decision', { root, ids, decision });
export const quarantineImages = (root: string, ids: number[]) =>
  invoke<OpOutcome[]>('quarantine_images', { root, ids }, MOVE_TIMEOUT_MS);
export const restoreImages = (root: string, ids: number[]) => invoke<OpOutcome[]>('restore_images', { root, ids }, MOVE_TIMEOUT_MS);
export const listFileOperations = (root: string, limit = 100) => invoke<FileOperation[]>('list_file_operations', { root, limit });

/** A thumbnail as a `data:` URL (the CSP allows `data:` images, not `blob:`). */
export async function getThumbnail(root: string, id: number, size: 256 | 1024 = 256): Promise<string> {
  const buf = await invoke<ArrayBuffer>('get_thumbnail', { root, id, size }, 30_000);
  return `data:image/jpeg;base64,${toBase64(new Uint8Array(buf))}`;
}

export function toBase64(bytes: Uint8Array): string {
  let binary = '';
  const step = 0x8000;
  for (let i = 0; i < bytes.length; i += step) binary += String.fromCharCode(...bytes.subarray(i, i + step));
  return btoa(binary);
}

// ---- Phase 4 — temporal sampling ------------------------------------------------------

export type Policy = 'closest' | 'first-after' | 'last-before';

export type SamplingMethod =
  | { method: 'fixed-interval'; interval_seconds: number; policy: Policy; tolerance_seconds?: number | null }
  | { method: 'every-nth'; n: number; offset: number }
  | { method: 'uniform'; count: number };

/** A sampling configuration: the method (flattened) plus the input rules. */
export type SamplingConfig = SamplingMethod & {
  /** Normalised bounds, `from` inclusive, `to` exclusive (e.g. `2024-01-15`). */
  from?: string | null;
  to?: string | null;
  include_flagged: boolean;
};

export interface Excluded {
  rejected: number;
  quarantined: number;
  flagged: number;
  no_timestamp: number;
  out_of_period: number;
}

export interface SampledItem {
  sequence_index: number;
  image_id: number;
  relative_path: string;
  target_datetime: string | null;
  actual_datetime: string;
  epoch_ms: number;
}

export interface SamplingPreview {
  config: SamplingConfig;
  algorithm_version: number;
  input_count: number;
  input_fingerprint: string;
  excluded: Excluded;
  target_count: number;
  selected_count: number;
  unmatched_count: number;
  unmatched_targets: string[];
  reduction: number;
  effective_interval_seconds: number | null;
  /** Per UTC day, over the whole selection. */
  day_counts: DayCount[];
  /** The first 200 selected photographs only; `selected_count` is the total. */
  items: SampledItem[];
}

export interface DayCount {
  day: string;
  count: number;
}

export interface RunPage {
  run_id: number;
  total: number;
  offset: number;
  items: SampledItem[];
  day_counts: DayCount[];
}

export interface RunSummary {
  id: number;
  method: SamplingMethod['method'];
  config: SamplingConfig;
  algorithm_version: number;
  input_count: number;
  excluded: Excluded;
  target_count: number;
  selected_count: number;
  unmatched_count: number;
  created_at: number;
}

export interface VerifyReport {
  run_id: number;
  inputs_unchanged: boolean;
  algorithm_unchanged: boolean;
  selection_identical: boolean;
}

const SAMPLING_TIMEOUT_MS = 120_000;

export const previewSampling = (root: string, config: SamplingConfig) =>
  invoke<SamplingPreview>('preview_sampling', { root, config }, SAMPLING_TIMEOUT_MS);
export const createSamplingRun = (root: string, config: SamplingConfig) =>
  invoke<RunSummary>('create_sampling_run', { root, config }, SAMPLING_TIMEOUT_MS);
export const listSamplingRuns = (root: string) => invoke<RunSummary[]>('list_sampling_runs', { root });
export const getSamplingItems = (root: string, run: number, offset = 0, limit = 200) =>
  invoke<RunPage>('get_sampling_items', { root, run, offset, limit });
export const verifySamplingRun = (root: string, run: number) =>
  invoke<VerifyReport>('verify_sampling_run', { root, run }, SAMPLING_TIMEOUT_MS);
