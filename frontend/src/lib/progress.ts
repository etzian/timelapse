/**
 * Time remaining for a running job, from a smoothed rate. Pure: the caller passes the clock.
 * No estimate is given until the rate has settled (enough samples over enough time), so the
 * first seconds of a job never show a wild number.
 */
export interface RateState {
  startedAt: number;
  lastAt: number;
  lastDone: number;
  /** Smoothed items per millisecond; null until a first measurable step. */
  rate: number | null;
  samples: number;
}

/** Weight of the newest sample in the moving average. */
export const SMOOTHING = 0.25;
/** Steps seen before the rate counts as stable. */
export const MIN_SAMPLES = 4;
/** Milliseconds of observation before the rate counts as stable. */
export const MIN_ELAPSED_MS = 3000;
/** Estimates longer than this are not shown (almost certainly noise). */
export const MAX_ESTIMATE_SECONDS = 7 * 86_400;

export function startRate(done: number, now: number): RateState {
  return { startedAt: now, lastAt: now, lastDone: done, rate: null, samples: 0 };
}

/** Folds a new reading in. Progress going backwards (a new job) starts over. */
export function observe(prev: RateState | null, done: number, now: number): RateState {
  if (!prev || done < prev.lastDone || now < prev.lastAt) return startRate(done, now);
  const dt = now - prev.lastAt;
  const dd = done - prev.lastDone;
  if (dd <= 0 || dt <= 0) return { ...prev, lastAt: dd <= 0 ? prev.lastAt : now };
  const instant = dd / dt;
  const rate = prev.rate === null ? instant : prev.rate + SMOOTHING * (instant - prev.rate);
  return { ...prev, lastAt: now, lastDone: done, rate, samples: prev.samples + 1 };
}

/** Whole seconds left, or null while the rate is not stable (or the job is over). */
export function remainingSeconds(state: RateState | null, done: number, total: number): number | null {
  if (!state || state.rate === null || state.rate <= 0) return null;
  if (state.samples < MIN_SAMPLES || state.lastAt - state.startedAt < MIN_ELAPSED_MS) return null;
  const left = total - done;
  if (left <= 0) return null;
  const seconds = Math.ceil(left / state.rate / 1000);
  return seconds > MAX_ESTIMATE_SECONDS ? null : seconds;
}

/** The fraction done, 0 to 1, for a bar; an empty job is not full. */
export function fraction(done: number, total: number): number {
  if (!(total > 0)) return 0;
  return Math.min(1, Math.max(0, done / total));
}
