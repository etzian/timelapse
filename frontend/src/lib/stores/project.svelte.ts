/**
 * The project the user is working on: the source folder, and what Home learned about it
 * (import report, integrity, backfill), so it survives leaving Home. Also the recent
 * list (`localStorage`: path + last-opened time, listed in `legal/data-map.yaml`).
 * On launch the last project is reopened from the database, without a rescan.
 */
import { integrityCheck, projectSummary } from '../ipc/app';
import type { BackfillReport, ImportReport, IntegrityResult } from '../ipc/app';
import { addRecent, parseRecent, RECENT_KEY, removeRecent, serializeRecent, type RecentProject } from '../project';

function read(): string | null {
  try {
    return localStorage.getItem(RECENT_KEY);
  } catch {
    return null;
  }
}
function persist(list: RecentProject[]): void {
  try {
    localStorage.setItem(RECENT_KEY, serializeRecent(list));
  } catch {
    /* storage disabled: the list lasts for this session only */
  }
}

let root = $state('');
let report = $state<ImportReport | null>(null);
let integrity = $state<IntegrityResult | null>(null);
let backfill = $state<BackfillReport | null>(null);
let recent = $state<RecentProject[]>(parseRecent(read()));
/** `opening` while the saved project is read; `unreachable` when its folder is missing or unmounted. */
let status = $state<'none' | 'opening' | 'ready' | 'unreachable'>('none');
/** The folder that could not be reached (stays in the list until removed). */
let unreachable = $state('');

export const project = {
  get root(): string {
    return root;
  },
  set root(value: string) {
    root = value;
  },
  get report() {
    return report;
  },
  set report(v: ImportReport | null) {
    report = v;
  },
  get integrity() {
    return integrity;
  },
  set integrity(v: IntegrityResult | null) {
    integrity = v;
  },
  get backfill() {
    return backfill;
  },
  set backfill(v: BackfillReport | null) {
    backfill = v;
  },
  get recent() {
    return recent;
  },
  get status() {
    return status;
  },
  get unreachable() {
    return unreachable;
  }
};

/** Forget what was learned about the previous folder (a new folder is being imported). */
export function resetProjectState(): void {
  report = null;
  integrity = null;
  backfill = null;
}

/** Record `path` as the last project. */
export function rememberProject(path: string, now = Date.now()): void {
  recent = addRecent(recent, path, now);
  persist(recent);
}

export function forgetProject(path: string): void {
  recent = removeRecent(recent, path);
  persist(recent);
  if (unreachable === path) {
    unreachable = '';
    if (status === 'unreachable') status = 'none';
  }
}

/** "Erase app settings": the recent list goes too. */
export function clearRecent(): void {
  recent = [];
  unreachable = '';
  if (status === 'unreachable') status = 'none';
  try {
    localStorage.removeItem(RECENT_KEY);
  } catch {
    /* nothing to erase */
  }
}

/**
 * Open a saved project without scanning: read the summary of the database. A folder that
 * cannot be reached keeps its entry and becomes the `unreachable` state (Retry / Remove).
 */
export async function openProject(path: string): Promise<void> {
  status = 'opening';
  unreachable = '';
  try {
    const summary = await projectSummary(path);
    resetProjectState();
    root = path;
    if (summary.imported) {
      backfill = { newly_resolved: 0, unresolved_this_run: 0, summary: summary.timeline };
      // Integrity is a read of the inventory against the disk; a failure never blocks opening.
      void integrityCheck(path)
        .then((r) => {
          if (root === path) integrity = r;
        })
        .catch(() => {});
    }
    status = 'ready';
    rememberProject(path);
  } catch {
    unreachable = path;
    status = 'unreachable';
  }
}

/** On launch: reopen the most recent project, if any. */
export async function restoreLastProject(): Promise<void> {
  const last = recent[0];
  if (last && !root) await openProject(last.path);
}
