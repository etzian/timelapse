/**
 * Presentation helpers for the Phase 4 sampling page: form → configuration, duration
 * estimate. Nothing here is user-visible text.
 */
import type { Policy, SamplingConfig, SamplingMethod } from './ipc/app';

export const FPS_OPTIONS = [12, 15, 24, 25, 30, 50, 60] as const;
export const MULTIPLIERS = [0.5, 1, 2, 4] as const;
export const POLICIES: readonly Policy[] = ['closest', 'first-after', 'last-before'];

export const UNIT_SECONDS = { seconds: 1, minutes: 60, hours: 3600, days: 86_400 } as const;
export type IntervalUnit = keyof typeof UNIT_SECONDS;

export type MethodName = SamplingMethod['method'];
export const METHODS: readonly MethodName[] = ['fixed-interval', 'every-nth', 'uniform'];

/** Raw form values, as typed. Dates are `YYYY-MM-DD` from `<input type="date">`. */
export interface SamplingForm {
  method: MethodName;
  interval: string;
  unit: IntervalUnit;
  policy: Policy;
  tolerance: string;
  n: string;
  offset: string;
  count: string;
  from: string;
  to: string;
  includeFlagged: boolean;
}

export function defaultForm(): SamplingForm {
  return {
    method: 'fixed-interval',
    interval: '1',
    unit: 'days',
    policy: 'closest',
    tolerance: '',
    n: '10',
    offset: '0',
    count: '300',
    from: '',
    to: '',
    includeFlagged: true
  };
}

const num = (s: string) => (s.trim() === '' ? Number.NaN : Number(s.trim().replace(',', '.')));
const isDate = (s: string) => /^\d{4}-\d{2}-\d{2}$/.test(s) && !Number.isNaN(Date.parse(`${s}T00:00:00Z`));

/** The day after `YYYY-MM-DD`, so an inclusive "to" date becomes the exclusive bound. */
export function nextDay(date: string): string {
  const d = new Date(`${date}T00:00:00Z`);
  d.setUTCDate(d.getUTCDate() + 1);
  return d.toISOString().slice(0, 10);
}

/** Validate the form; `null` when any field is out of range. */
export function buildConfig(f: SamplingForm): SamplingConfig | null {
  let method: SamplingMethod;
  if (f.method === 'fixed-interval') {
    const interval = num(f.interval) * UNIT_SECONDS[f.unit];
    if (!(interval > 0) || !Number.isFinite(interval)) return null;
    let tolerance: number | null = null;
    if (f.tolerance.trim() !== '') {
      tolerance = num(f.tolerance) * UNIT_SECONDS[f.unit];
      if (!(tolerance >= 0) || !Number.isFinite(tolerance)) return null;
    }
    method = { method: 'fixed-interval', interval_seconds: interval, policy: f.policy, tolerance_seconds: tolerance };
  } else if (f.method === 'every-nth') {
    const n = num(f.n);
    const offset = f.offset.trim() === '' ? 0 : num(f.offset);
    if (!Number.isInteger(n) || n < 1 || !Number.isInteger(offset) || offset < 0) return null;
    method = { method: 'every-nth', n, offset };
  } else {
    const count = num(f.count);
    if (!Number.isInteger(count) || count < 1) return null;
    method = { method: 'uniform', count };
  }
  if ((f.from && !isDate(f.from)) || (f.to && !isDate(f.to))) return null;
  if (f.from && f.to && f.to < f.from) return null;
  return {
    ...method,
    from: f.from || null,
    to: f.to ? nextDay(f.to) : null,
    include_flagged: f.includeFlagged
  };
}

/** Video length: one photograph per frame at `fps`, played `multiplier` times faster (PRD §5.7). */
export function estimatedDurationSeconds(selected: number, fps: number, multiplier: number): number {
  return fps > 0 && multiplier > 0 ? selected / fps / multiplier : 0;
}
