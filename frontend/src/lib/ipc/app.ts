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
export type Granularity = 'hour' | 'day' | 'week' | 'month';

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
  /** UTC key: `YYYY-MM-DDTHH`, `YYYY-MM-DD` or `YYYY-MM` (a prefix of the instants it counts); a week's key is its Monday. */
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
  /** The consolidation run a copy / move / undo step belongs to. */
  job_id: number | null;
  image_id: number | null;
  operation_type: 'quarantine' | 'restore' | 'copy' | 'move' | 'move-back' | 'remove-copy';
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

// ---- Phase 5 — video export ------------------------------------------------------------

export type Codec = 'av1' | 'h264' | 'hevc';
export type Quality = 'high' | 'standard' | 'small';

export interface FfmpegInfo {
  available: boolean;
  version: string | null;
  lgpl: boolean;
  av1: boolean;
  h264: boolean;
  hevc: boolean;
}

export interface ExportSettings {
  codec: Codec;
  fps: number;
  /** Playback multiplier, separate from fps (2 = twice as fast). */
  speed: number;
  width: number;
  height: number;
  quality: Quality;
}

export interface ExportProgress {
  done: number;
  total: number;
}

export interface Probed {
  codec_name: string;
  width: number;
  height: number;
  packets: number;
  duration_seconds: number;
}

export interface ExportRecord {
  id: number;
  sampling_run_id: number;
  settings: ExportSettings;
  ffmpeg_version: string | null;
  output_path: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  input_photos: number;
  output_frames: number;
  duration_seconds: number;
  file_size: number | null;
  probe: Probed | null;
  /** Stable kind, shown via `error-export` with this reason. */
  failure: string | null;
  stderr_tail: string | null;
  created_at: number;
  completed_at: number | null;
}

/** Encoding a long timelapse takes a while; progress arrives on the channel meanwhile. */
const EXPORT_TIMEOUT_MS = 6 * 60 * 60_000;

export const ffmpegInfo = () => invoke<FfmpegInfo>('ffmpeg_info', {}, 30_000);

/** `token` identifies this export for `cancelExport` (any non-zero safe integer). */
export function startExport(
  root: string,
  run: number,
  settings: ExportSettings,
  token: number,
  onProgress: (p: ExportProgress) => void
): Promise<ExportRecord> {
  const channel = new Channel<ExportProgress>();
  channel.onmessage = onProgress;
  return invoke<ExportRecord>('start_export', { root, run, settings, token, onProgress: channel }, EXPORT_TIMEOUT_MS);
}

export const cancelExport = (token: number) => invoke<void>('cancel_export', { token });
export const listExports = (root: string) => invoke<ExportRecord[]>('list_exports', { root });

// ---- Phase 6 — consolidation -------------------------------------------------------------

export type Operation = 'copy' | 'move';
export type Naming = 'preserve' | 'original' | 'folder-prefix' | 'sequence';

export interface ConsolidationConfig {
  /** An absolute folder outside the collection (it may not exist yet). */
  destination: string;
  operation: Operation;
  naming: Naming;
  /** A saved sampling selection; null = every photograph except rejected and quarantined ones. */
  sampling_run: number | null;
}

export type PlanAction = { kind: 'place' } | { kind: 'duplicate'; of: number } | { kind: 'already-present' } | { kind: 'unsafe' };

export interface PlannedItem {
  image_id: number;
  relative_path: string;
  /** Relative to the destination, `/`-separated; null for duplicates and unsafe names. */
  target: string | null;
  size: number;
  action: PlanAction;
  renamed: boolean;
}

export interface PlanCounts {
  planned: number;
  duplicates: number;
  already_present: number;
  unsafe_names: number;
  bytes: number;
}

export interface ConsolidationExcluded {
  rejected: number;
  quarantined: number;
  missing: number;
}

export interface ConsolidationPreview {
  config: ConsolidationConfig;
  fingerprint: string;
  input_count: number;
  excluded: ConsolidationExcluded;
  counts: PlanCounts;
  renamed: number;
  free_bytes: number | null;
  enough_space: boolean;
  /** The first 200 items of the plan. */
  items: PlannedItem[];
  /** Up to 200 items that are skipped or renamed. */
  notable: PlannedItem[];
}

export interface ConsolidationProgress {
  done: number;
  total: number;
  failed: number;
}

export type ConsolidationStatus = 'running' | 'completed' | 'partial' | 'cancelled' | 'interrupted';

export interface ConsolidationRecord {
  id: number;
  config: ConsolidationConfig;
  plan: PlanCounts;
  status: ConsolidationStatus;
  undo_status: 'undone' | 'partially-undone' | null;
  placed: number;
  failed: number;
  undone: number;
  undo_failed: number;
  created_at: number;
  completed_at: number | null;
  undone_at: number | null;
}

export interface Problem {
  image_id: number;
  path: string;
  state: 'modified' | 'missing' | 'moved-since' | 'changed';
}

export interface RunCheck {
  run_id: number;
  checked: number;
  ok: number;
  modified: number;
  missing: number;
  moved_since: number;
  problems: Problem[];
}

export interface OriginalsCheck {
  total: number;
  ok: number;
  changed: number;
  missing: number;
  unhashed: number;
  problems: Problem[];
}

/** Copying a large collection takes a while; progress arrives on the channel meanwhile. */
const CONSOLIDATE_TIMEOUT_MS = 6 * 60 * 60_000;
const PLAN_TIMEOUT_MS = 120_000;

function progressChannel(onProgress: (p: ConsolidationProgress) => void): Channel<ConsolidationProgress> {
  const channel = new Channel<ConsolidationProgress>();
  channel.onmessage = onProgress;
  return channel;
}

export const previewConsolidation = (root: string, config: ConsolidationConfig) =>
  invoke<ConsolidationPreview>('preview_consolidation', { root, config }, PLAN_TIMEOUT_MS);

/** `fingerprint` is the confirmed dry run's; `token` identifies the run for `cancelConsolidation`. */
export const startConsolidation = (
  root: string,
  config: ConsolidationConfig,
  fingerprint: string,
  token: number,
  onProgress: (p: ConsolidationProgress) => void
) =>
  invoke<ConsolidationRecord>(
    'start_consolidation',
    { root, config, fingerprint, token, onProgress: progressChannel(onProgress) },
    CONSOLIDATE_TIMEOUT_MS
  );

export const undoConsolidation = (root: string, run: number, token: number, onProgress: (p: ConsolidationProgress) => void) =>
  invoke<ConsolidationRecord>('undo_consolidation', { root, run, token, onProgress: progressChannel(onProgress) }, CONSOLIDATE_TIMEOUT_MS);

export const cancelConsolidation = (token: number) => invoke<void>('cancel_consolidation', { token });
export const listConsolidations = (root: string) => invoke<ConsolidationRecord[]>('list_consolidations', { root });
export const verifyConsolidation = (root: string, run: number) =>
  invoke<RunCheck>('verify_consolidation', { root, run }, CONSOLIDATE_TIMEOUT_MS);
export const verifyOriginals = (root: string, onProgress: (p: ConsolidationProgress) => void) =>
  invoke<OriginalsCheck>('verify_originals', { root, onProgress: progressChannel(onProgress) }, CONSOLIDATE_TIMEOUT_MS);

// ---- Phase 7 — contact sheets, export record, timeline ------------------------------------

/** What a sheet shows (PRD §5.11). */
export type SheetSource =
  | { kind: 'all' }
  | { kind: 'period'; from: string; to: string }
  | { kind: 'flagged' }
  | { kind: 'quarantined' }
  | { kind: 'sampling'; run: number }
  | { kind: 'export'; export: number };

export type PageSize = 'a4' | 'letter';
export type Orientation = 'portrait' | 'landscape';

export interface SheetLayout {
  page_size: PageSize;
  orientation: Orientation;
  /** 1–12 each. */
  columns: number;
  rows: number;
  show_filename: boolean;
  show_timestamp: boolean;
}

/** Texts printed on every page, already translated. `page` has `{page}` and `{pages}`. */
export interface SheetLabels {
  title: string;
  source: string;
  page: string;
  note: string;
  unreadable: string;
}

/** PDF points (1/72 inch), origin at the page's lower-left corner. */
export interface SheetRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface SheetGeometry {
  page_w: number;
  page_h: number;
  columns: number;
  rows: number;
  cell_w: number;
  cell_h: number;
  image_h: number;
  caption_size: number;
  caption_lines: number;
  title_y: number;
  source_y: number;
  footer_y: number;
}

export interface SheetPreviewCell {
  image_id: number;
  image_box: SheetRect;
  filename: string | null;
  timestamp: string | null;
}

export interface SheetPreview {
  photos: number;
  pages: number;
  geometry: SheetGeometry;
  first_page: SheetPreviewCell[];
  too_many: boolean;
}

export interface SheetProgress {
  done: number;
  total: number;
}

export interface SheetRecord {
  id: number;
  source: SheetSource;
  layout: SheetLayout;
  output_path: string;
  photos: number;
  unreadable: number;
  pages: number;
  file_size: number | null;
  status: 'running' | 'completed' | 'failed' | 'cancelled' | 'interrupted';
  /** Stable kind, shown via `error-contact-sheet` with this reason. */
  failure: string | null;
  created_at: number;
  completed_at: number | null;
}

/** A large sheet makes thousands of thumbnails; progress arrives on the channel meanwhile. */
const SHEET_TIMEOUT_MS = 6 * 60 * 60_000;

export const previewContactSheet = (root: string, source: SheetSource, layout: SheetLayout) =>
  invoke<SheetPreview>('preview_contact_sheet', { root, source, layout }, 120_000);

/** `expected` is the photo count printed in `labels` (refused as `source-changed` otherwise); `token` identifies this sheet for `cancelContactSheet`. */
export function startContactSheet(
  root: string,
  source: SheetSource,
  layout: SheetLayout,
  labels: SheetLabels,
  expected: number | null,
  token: number,
  onProgress: (p: SheetProgress) => void
): Promise<SheetRecord> {
  const channel = new Channel<SheetProgress>();
  channel.onmessage = onProgress;
  return invoke<SheetRecord>('start_contact_sheet', { root, source, layout, labels, expected, token, onProgress: channel }, SHEET_TIMEOUT_MS);
}

export const cancelContactSheet = (token: number) => invoke<void>('cancel_contact_sheet', { token });
export const listContactSheets = (root: string) => invoke<SheetRecord[]>('list_contact_sheets', { root });

/** The derived export record (ADR-015), pretty JSON. */
export const exportRecord = (root: string, exportId: number) => invoke<string>('export_record', { root, export: exportId });
/** Saves the record as `.timelapse/exports/<id>.record.json`; resolves to its path. */
export const writeExportRecord = (root: string, exportId: number) => invoke<string>('write_export_record', { root, export: exportId });

export interface TimelinePage {
  total: number;
  entries: TimelineEntry[];
}

export const timelinePeriod = (root: string, from: string, to: string, offset: number, limit: number) =>
  invoke<TimelinePage>('timeline_period', { root, from, to, offset, limit });

export interface IdenticalGroup {
  normalized_datetime: string;
  epoch_ms: number;
  count: number;
  /** The first few photographs of the group. */
  sample: TimelineEntry[];
}

export interface IdenticalReport {
  groups: number;
  photos: number;
  largest: IdenticalGroup[];
}

export const identicalTimestamps = (root: string, limit: number) => invoke<IdenticalReport>('identical_timestamps', { root, limit });
