/** Pure rules of the flipbook: timing, the loaded window, eviction and waiting. No DOM, no IPC. */

export type FrameState = 'loading' | 'ready' | 'failed';

export const WINDOW_AHEAD = 24;
export const WINDOW_BEHIND = 8;
export const BATCH = 8;

/** Milliseconds one frame stays on screen at `fps` frames per second and a playback `speed`. */
export function frameIntervalMs(fps: number, speed = 1): number {
  const rate = fps * speed;
  return rate > 0 && Number.isFinite(rate) ? 1000 / rate : 1000;
}

/** `index + delta`, wrapping around the sequence; -1 for an empty one. */
export function wrapIndex(index: number, delta: number, length: number): number {
  if (length <= 0) return -1;
  return (((index + delta) % length) + length) % length;
}

/** Keeps a scrubber or key value inside the sequence. */
export function clampIndex(index: number, length: number): number {
  if (length <= 0) return 0;
  return Math.min(length - 1, Math.max(0, Math.trunc(index) || 0));
}

/**
 * Indexes kept loaded around the playhead, in loading order: the playhead, then the frames ahead
 * (wrapping past the end, since playback loops), then the few behind it.
 */
export function windowIndexes(index: number, length: number, ahead = WINDOW_AHEAD, behind = WINDOW_BEHIND): number[] {
  if (length <= 0) return [];
  const out: number[] = [];
  const seen = new Set<number>();
  const push = (i: number) => {
    if (!seen.has(i)) {
      seen.add(i);
      out.push(i);
    }
  };
  for (let d = 0; d <= ahead && d < length; d++) push(wrapIndex(index, d, length));
  for (let d = 1; d <= behind && d < length; d++) push(wrapIndex(index, -d, length));
  return out;
}

export interface LoadPlan {
  /** The next batch to fetch: missing frames of the window, nearest first. */
  load: number[];
  /** Cached frames that fell out of the window. */
  evict: number[];
}

export function planLoad(
  index: number,
  length: number,
  cached: Iterable<number>,
  batch = BATCH,
  ahead = WINDOW_AHEAD,
  behind = WINDOW_BEHIND
): LoadPlan {
  const win = windowIndexes(index, length, ahead, behind);
  const inWindow = new Set(win);
  const have = new Set(cached);
  return {
    load: win.filter((i) => !have.has(i)).slice(0, batch),
    evict: [...have].filter((i) => !inWindow.has(i))
  };
}

export interface Advance {
  index: number;
  /** True when the next frame is still loading: stay where you are and try again. */
  waiting: boolean;
}

/**
 * One playback tick. A frame that is still loading holds playback (it never skips); a failed one
 * counts as shown (as a placeholder) so playback goes on. The sequence loops.
 */
export function advance(index: number, length: number, stateOf: (i: number) => FrameState | undefined): Advance {
  if (length <= 0) return { index: 0, waiting: false };
  const next = wrapIndex(index, 1, length);
  const s = stateOf(next);
  if (s === 'ready' || s === 'failed') return { index: next, waiting: false };
  return { index, waiting: true };
}
