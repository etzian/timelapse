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
