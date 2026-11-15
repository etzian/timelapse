/**
 * Presentation helpers for the Phase 3 cleaning page. Data values from the core
 * (reason kinds, statuses, failure kinds) map to Fluent ids here; nothing in this
 * file is user-visible text.
 */
import { formatNumber, t } from './i18n';
import type { Decision, ImageStatus, Reason, ReviewItem, Thresholds } from './ipc/app';

export const STATUS_KEYS: Record<ImageStatus, string> = {
  eligible: 'clean-status-eligible',
  flagged: 'clean-status-flagged',
  rejected: 'clean-status-rejected',
  quarantined: 'clean-status-quarantined'
};

export const STATUS_VARIANT: Record<ImageStatus, 'success' | 'warning' | 'error' | 'info'> = {
  eligible: 'success',
  flagged: 'warning',
  rejected: 'error',
  quarantined: 'info'
};

/** Failure kinds a file operation can report (`crates/core/src/files`). */
export const FILE_FAILURES = [
  'unknown-image',
  'not-rejected',
  'not-quarantined',
  'unsafe-path',
  'source-unreadable',
  'changed-since-import',
  'destination-exists',
  'move-failed',
  'verify-failed',
  'source-in-use',
  'record-failed',
  'interrupted',
  'inconsistent',
  'missing',
  'modified',
  'moved-since',
  'remove-failed',
  'now-an-original',
  'unrecorded-copy',
  'replaced'
] as const;

export function fileFailureText(kind: string): string {
  return (FILE_FAILURES as readonly string[]).includes(kind) ? t(`files-failure-${kind}`) : t('files-failure-other');
}

/** Analysis failure kinds (`crates/core/src/clean/analysis.rs`). */
export function analysisFailureText(kind: string): string {
  return kind === 'unreadable' ? t('clean-failure-unreadable') : t('clean-failure-undecodable');
}

const percent = (v: number) => formatNumber(v, { style: 'percent', maximumFractionDigits: 0 });
const luma = (v: number) => formatNumber(v, { maximumFractionDigits: 0 });

/** One reason as a sentence, numbers through `Intl`. */
export function reasonText(r: Reason): string {
  switch (r.kind) {
    case 'mean-below':
      return t('clean-reason-mean-below', { mean: luma(r.mean), min: luma(r.min) });
    case 'too-many-dark-pixels':
      return t('clean-reason-too-many-dark-pixels', { fraction: percent(r.fraction), max: percent(r.max) });
    case 'xmp-rejected':
      return t('clean-reason-xmp-rejected');
    case 'analysis-failed':
      return t('clean-reason-analysis-failed', { failure: analysisFailureText(r.failure) });
  }
}

export { percent as formatPercent, luma as formatLuma };

/** Form values (strings, as typed) → thresholds, or null when any field is invalid. */
export function parseThresholds(form: { minMean: string; darkLevel: string; maxDarkPercent: string }, flagXmp: boolean): Thresholds | null {
  const n = (s: string) => (s.trim() === '' ? Number.NaN : Number(s.replace(',', '.')));
  const minMean = n(form.minMean);
  const darkLevel = n(form.darkLevel);
  const maxDark = n(form.maxDarkPercent);
  const inRange = (v: number, hi: number) => Number.isFinite(v) && v >= 0 && v <= hi;
  if (!inRange(minMean, 255) || !inRange(darkLevel, 255) || !Number.isInteger(darkLevel) || !inRange(maxDark, 100)) return null;
  return { min_mean_luma: minMean, dark_level: darkLevel, max_dark_fraction: maxDark / 100, flag_xmp_rejected: flagXmp };
}

export function thresholdsToForm(th: Thresholds): { minMean: string; darkLevel: string; maxDarkPercent: string } {
  return {
    minMean: String(th.min_mean_luma),
    darkLevel: String(th.dark_level),
    maxDarkPercent: String(Math.round(th.max_dark_fraction * 1000) / 10)
  };
}

/** Run `tasks` with at most `limit` in flight (thumbnail loading). */
export async function limited<T>(tasks: (() => Promise<T>)[], limit: number): Promise<PromiseSettledResult<T>[]> {
  const results: PromiseSettledResult<T>[] = new Array(tasks.length);
  let next = 0;
  async function worker(): Promise<void> {
    while (next < tasks.length) {
      const i = next++;
      try {
        results[i] = { status: 'fulfilled', value: await tasks[i]() };
      } catch (reason) {
        results[i] = { status: 'rejected', reason };
      }
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, tasks.length) }, worker));
  return results;
}

// ---- Fast triage: grid movement, focus after removal, undo snapshots -----------------

/** Keys the grid moves with. */
export type MoveKey = 'ArrowLeft' | 'ArrowRight' | 'ArrowUp' | 'ArrowDown' | 'Home' | 'End';

/** Columns of a wrapped grid, from the `offsetTop` of its tiles in order (tiles of the first row share a top). */
export function columnsFromTops(tops: number[]): number {
  if (tops.length === 0) return 1;
  const n = tops.findIndex((top) => top !== tops[0]);
  return n === -1 ? tops.length : n;
}

/**
 * Next index when `key` is pressed on tile `index` of `count` tiles laid out in `columns`.
 * Movement stops at the edges (no wrap); Up/Down stay put when no tile lies that way, except that
 * Down from a short last row goes to the very last tile. An empty grid has no focus (-1).
 */
export function moveIndex(index: number, key: MoveKey, count: number, columns: number): number {
  if (count <= 0) return -1;
  const cols = Math.max(1, Math.floor(columns));
  const at = Math.min(Math.max(index, 0), count - 1);
  switch (key) {
    case 'ArrowLeft':
      return Math.max(0, at - 1);
    case 'ArrowRight':
      return Math.min(count - 1, at + 1);
    case 'ArrowUp':
      return at - cols >= 0 ? at - cols : at;
    case 'ArrowDown':
      if (at + cols <= count - 1) return at + cols;
      return Math.floor(at / cols) < Math.floor((count - 1) / cols) ? count - 1 : at;
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
  }
}

/**
 * The id to focus once a list changed from `before` to `after` while `focused` had focus: the same
 * tile when it is still there, else the next survivor that followed it, else the previous one,
 * else null (the list is empty).
 */
export function focusAfterRemoval(before: number[], after: number[], focused: number | null): number | null {
  if (after.length === 0) return null;
  if (focused === null) return after[0];
  const kept = new Set(after);
  if (kept.has(focused)) return focused;
  const from = before.indexOf(focused);
  if (from === -1) return after[0];
  for (let i = from + 1; i < before.length; i++) if (kept.has(before[i])) return before[i];
  for (let i = from - 1; i >= 0; i--) if (kept.has(before[i])) return before[i];
  return after[0];
}

/** The id after `id` in `ids`, else the one before it, else null: where the inspector moves on to. */
export function neighbourId(ids: number[], id: number): number | null {
  const at = ids.indexOf(id);
  if (at === -1) return ids[0] ?? null;
  return ids[at + 1] ?? ids[at - 1] ?? null;
}

/** Whether `decision` would change anything for this photograph (unanalysed and quarantined ones cannot be decided). */
export function decisionApplies(item: Pick<ReviewItem, 'status' | 'user_override'>, decision: Decision): boolean {
  if (item.status === null || item.status === 'quarantined') return false;
  switch (decision) {
    case 'accept':
      return item.status !== 'eligible' || !item.user_override;
    case 'reject':
      return item.status !== 'rejected';
    case 'clear':
      return item.user_override;
  }
}

/** The decision that, set again, restores this photograph to how it is now. */
export function previousDecision(item: Pick<ReviewItem, 'status' | 'user_override'>): Decision {
  if (!item.user_override) return 'clear';
  return item.status === 'rejected' ? 'reject' : 'accept';
}

export interface DecisionPlan {
  /** Ids the decision changes. */
  ids: number[];
  /** What to send to `set_decision` to undo it, one group per previous decision. */
  undo: { decision: Decision; ids: number[] }[];
}

/** Which of `ids` the decision applies to, and the snapshot that undoes it. Unknown ids are ignored. */
export function planDecision(items: ReviewItem[], ids: number[], decision: Decision): DecisionPlan {
  const byId = new Map(items.map((i) => [i.id, i]));
  const groups = new Map<Decision, number[]>();
  const changed: number[] = [];
  for (const id of new Set(ids)) {
    const item = byId.get(id);
    if (!item || !decisionApplies(item, decision)) continue;
    changed.push(id);
    const prev = previousDecision(item);
    groups.set(prev, [...(groups.get(prev) ?? []), id]);
  }
  return { ids: changed, undo: [...groups].map(([d, g]) => ({ decision: d, ids: g })) };
}
