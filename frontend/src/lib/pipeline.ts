/**
 * The pipeline as the navigation shows it: the state of each workflow step, derived from
 * counts the pages already load, and which running jobs just ended. Pure.
 */
export type StepId = 'home' | 'cleaning' | 'sampling' | 'adjust' | 'export';
export type StepState = 'not-started' | 'in-progress' | 'done';
export type JobId = 'analysis' | 'adjust' | 'export' | 'consolidation' | 'sheet';

export const STEP_IDS: readonly StepId[] = ['home', 'cleaning', 'sampling', 'adjust', 'export'];
export const JOB_IDS: readonly JobId[] = ['analysis', 'adjust', 'export', 'consolidation', 'sheet'];

/** The page whose nav item shows a running job. */
export const JOB_PAGE: Record<JobId, 'cleaning' | 'adjust' | 'export' | 'organize' | 'sheets'> = {
  analysis: 'cleaning',
  adjust: 'adjust',
  export: 'export',
  consolidation: 'organize',
  sheet: 'sheets'
};

export interface Figures {
  photos: number;
  /** Photographs not yet measured. */
  unanalyzed: number;
  /** Flagged and still waiting for a decision. */
  flagged: number;
  selections: number;
  /** Deflicker and stabilization runs made. */
  adjustments: number;
  /** Completed exports. */
  exports: number;
}

export type Running = Record<JobId, boolean>;
export const NO_JOBS: Running = { analysis: false, adjust: false, export: false, consolidation: false, sheet: false };

export function stepStates(f: Figures | null, running: Running = NO_JOBS): Record<StepId, StepState> {
  const none = !f || f.photos <= 0;
  const clean: StepState = none
    ? 'not-started'
    : f.unanalyzed >= f.photos
      ? 'not-started'
      : f.unanalyzed > 0 || f.flagged > 0
        ? 'in-progress'
        : 'done';
  return {
    home: none ? 'not-started' : 'done',
    cleaning: running.analysis && clean === 'not-started' ? 'in-progress' : clean,
    sampling: !f || f.selections <= 0 ? 'not-started' : 'done',
    adjust: running.adjust ? 'in-progress' : !f || f.adjustments <= 0 ? 'not-started' : 'done',
    export: running.export ? 'in-progress' : !f || f.exports <= 0 ? 'not-started' : 'done'
  };
}

/** Jobs that were running in `prev` and are not in `next`. */
export function endedJobs(prev: Running, next: Running): JobId[] {
  return JOB_IDS.filter((j) => prev[j] && !next[j]);
}

/** Jobs running in `next` that were not in `prev`, in JOB_IDS order. */
export function startedJobs(prev: Running, next: Running): JobId[] {
  return JOB_IDS.filter((j) => !prev[j] && next[j]);
}

export type NextTarget = 'cleaning' | 'sampling' | 'adjust' | 'export' | 'organize';
/** Why the next step cannot start yet; each maps to one `next-reason-*` message. */
export type NextReason = 'checking' | 'no-photos' | 'analysing' | 'no-selection' | 'adjusting';

export interface NextStep {
  target: NextTarget;
  /** `null` when the next step has what it needs. */
  reason: NextReason | null;
}

const FOLLOWING: Record<StepId, NextTarget> = {
  home: 'cleaning',
  cleaning: 'sampling',
  sampling: 'adjust',
  adjust: 'export',
  export: 'organize'
};

/** The step after `step` and whether it can start, from the figures the navigation already holds. */
export function nextStep(step: StepId, f: Figures | null, running: Running = NO_JOBS): NextStep {
  const target = FOLLOWING[step];
  if (!f) return { target, reason: 'checking' };
  if (f.photos <= 0) return { target, reason: 'no-photos' };
  if (step === 'cleaning' && running.analysis) return { target, reason: 'analysing' };
  if ((step === 'sampling' || step === 'adjust') && f.selections <= 0) return { target, reason: 'no-selection' };
  if (step === 'adjust' && running.adjust) return { target, reason: 'adjusting' };
  return { target, reason: null };
}
