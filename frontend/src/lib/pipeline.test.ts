import { describe, expect, it } from 'vitest';

import { endedJobs, NO_JOBS, startedJobs, stepStates, type Figures } from './pipeline';

const base: Figures = { photos: 100, unanalyzed: 100, flagged: 0, selections: 0, adjustments: 0, exports: 0 };

describe('stepStates', () => {
  it('starts with nothing done before a project has photographs', () => {
    expect(stepStates(null)).toEqual({ home: 'not-started', cleaning: 'not-started', sampling: 'not-started', adjust: 'not-started', export: 'not-started' });
    expect(stepStates({ ...base, photos: 0 }).home).toBe('not-started');
  });

  it('marks import done and clean not started once photographs are in', () => {
    const s = stepStates(base);
    expect(s.home).toBe('done');
    expect(s.cleaning).toBe('not-started');
  });

  it('cleaning is in progress while some are unmeasured or flagged, done when clear', () => {
    expect(stepStates({ ...base, unanalyzed: 40 }).cleaning).toBe('in-progress');
    expect(stepStates({ ...base, unanalyzed: 0, flagged: 3 }).cleaning).toBe('in-progress');
    expect(stepStates({ ...base, unanalyzed: 0, flagged: 0 }).cleaning).toBe('done');
  });

  it('a running analysis counts as in progress', () => {
    expect(stepStates(base, { ...NO_JOBS, analysis: true }).cleaning).toBe('in-progress');
  });

  it('select, adjust and export follow their counts, and running wins', () => {
    const f = { ...base, selections: 2, adjustments: 1, exports: 1 };
    const s = stepStates(f);
    expect([s.sampling, s.adjust, s.export]).toEqual(['done', 'done', 'done']);
    const r = stepStates(f, { ...NO_JOBS, adjust: true, export: true });
    expect([r.adjust, r.export]).toEqual(['in-progress', 'in-progress']);
    expect(stepStates(base).sampling).toBe('not-started');
  });
});

describe('endedJobs', () => {
  it('reports only jobs that stopped', () => {
    const prev = { ...NO_JOBS, export: true, sheet: true };
    const next = { ...NO_JOBS, sheet: true, adjust: true };
    expect(endedJobs(prev, next)).toEqual(['export']);
    expect(endedJobs(NO_JOBS, NO_JOBS)).toEqual([]);
  });
});

describe('startedJobs', () => {
  it('reports only jobs that began, never one still running or one that stopped', () => {
    const prev = { ...NO_JOBS, export: true, sheet: true };
    const next = { ...NO_JOBS, sheet: true, adjust: true };
    expect(startedJobs(prev, next)).toEqual(['adjust']);
    expect(startedJobs(next, next)).toEqual([]);
  });
});

import { nextStep } from './pipeline';

describe('nextStep', () => {
  const f = { photos: 10, unanalyzed: 0, flagged: 0, selections: 1, adjustments: 0, exports: 0 };
  it('names the following step', () => {
    expect(nextStep('home', f)).toEqual({ target: 'cleaning', reason: null });
    expect(nextStep('export', f).target).toBe('organize');
  });
  it('refuses with a reason when the step lacks what it needs', () => {
    expect(nextStep('home', null).reason).toBe('checking');
    expect(nextStep('home', { ...f, photos: 0 }).reason).toBe('no-photos');
    expect(nextStep('cleaning', f, { ...NO_JOBS, analysis: true }).reason).toBe('analysing');
    expect(nextStep('sampling', { ...f, selections: 0 }).reason).toBe('no-selection');
    expect(nextStep('adjust', { ...f, selections: 0 }).reason).toBe('no-selection');
    expect(nextStep('adjust', f, { ...NO_JOBS, adjust: true }).reason).toBe('adjusting');
  });
});
