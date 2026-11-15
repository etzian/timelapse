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

/** The pieces of a saved selection's name; each maps to one Fluent message in the UI. */
export type SelectionMethodPart =
  | { kind: 'interval'; unit: IntervalUnit; value: number }
  | { kind: 'nth'; n: number }
  | { kind: 'uniform'; count: number };

/** How a run was sampled, in the largest unit that divides the interval evenly. */
export function selectionMethodPart(run: { config: SamplingConfig }): SelectionMethodPart {
  const c = run.config;
  if (c.method === 'every-nth') return { kind: 'nth', n: c.n };
  if (c.method === 'uniform') return { kind: 'uniform', count: c.count };
  const s = c.interval_seconds;
  for (const unit of ['days', 'hours', 'minutes'] as const) {
    if (s >= UNIT_SECONDS[unit] && s % UNIT_SECONDS[unit] === 0) return { kind: 'interval', unit, value: s / UNIT_SECONDS[unit] };
  }
  return { kind: 'interval', unit: 'seconds', value: s };
}

/** What the label needs from the UI layer: message lookup and `Intl` formatting. */
export interface LabelFormat {
  t: (id: string, args?: Record<string, unknown>) => string;
  number: (n: number) => string;
  /** `withYear` is true when the run was made in another year than `now`. */
  date: (ms: number, withYear: boolean) => string;
}

/**
 * "Every 5 min · 1,033 photos · 15 Nov": what a saved selection is, from its method, its
 * photo count and the day it was made. Pure; the id is not part of it (see `selectionRef`).
 */
export function selectionLabel(
  run: { config: SamplingConfig; selected_count: number; created_at: number },
  f: LabelFormat,
  now: number = Date.now()
): string {
  const part = selectionMethodPart(run);
  const method =
    part.kind === 'interval'
      ? f.t(`selection-every-${part.unit}`, { n: f.number(part.value), count: part.value })
      : part.kind === 'nth'
        ? f.t('selection-nth', { n: f.number(part.n) })
        : f.t('selection-uniform');
  const ms = run.created_at * 1000;
  const withYear = new Date(ms).getFullYear() !== new Date(now).getFullYear();
  return f.t('selection-label', {
    method,
    count: run.selected_count,
    photos: f.number(run.selected_count),
    date: f.date(ms, withYear)
  });
}
