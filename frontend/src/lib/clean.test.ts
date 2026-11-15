import { describe, expect, it } from 'vitest';

import {
  FILE_FAILURES,
  STATUS_KEYS,
  columnsFromTops,
  decisionApplies,
  fileFailureText,
  focusAfterRemoval,
  limited,
  moveIndex,
  neighbourId,
  parseThresholds,
  planDecision,
  previousDecision,
  reasonText,
  thresholdsToForm
} from './clean';
import { setLocale, t } from './i18n';
import { toBase64 } from './ipc/app';
import type { ImageStatus, Reason, ReviewItem } from './ipc/app';

const reasons: Reason[] = [
  { kind: 'mean-below', mean: 12.4, min: 40 },
  { kind: 'too-many-dark-pixels', fraction: 0.82, max: 0.6, level: 30 },
  { kind: 'xmp-rejected' },
  { kind: 'analysis-failed', failure: 'undecodable' }
];

describe('cleaning helpers', () => {
  it('has a message for every status, reason and failure in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const id of Object.values(STATUS_KEYS)) expect(t(id)).not.toBe(id);
      for (const r of reasons) expect(reasonText(r)).not.toMatch(/^clean-/);
      for (const f of FILE_FAILURES) expect(fileFailureText(f)).not.toMatch(/^files-/);
      expect(fileFailureText('something-new')).not.toMatch(/^files-/);
    }
  });

  it('formats reason numbers through Intl', () => {
    setLocale('en');
    expect(reasonText(reasons[1])).toContain('82%');
    setLocale('fr-CA');
    expect(reasonText(reasons[1])).toMatch(/82\s%/);
  });

  it('parses threshold forms and rejects invalid input', () => {
    expect(parseThresholds({ minMean: '40', darkLevel: '30', maxDarkPercent: '60' }, true)).toEqual({
      min_mean_luma: 40,
      dark_level: 30,
      max_dark_fraction: 0.6,
      flag_xmp_rejected: true
    });
    expect(parseThresholds({ minMean: '40,5', darkLevel: '30', maxDarkPercent: '60' }, false)?.min_mean_luma).toBe(40.5);
    expect(parseThresholds({ minMean: '', darkLevel: '30', maxDarkPercent: '60' }, true)).toBeNull();
    expect(parseThresholds({ minMean: '40', darkLevel: '30.5', maxDarkPercent: '60' }, true)).toBeNull();
    expect(parseThresholds({ minMean: '40', darkLevel: '30', maxDarkPercent: '101' }, true)).toBeNull();
    expect(thresholdsToForm({ min_mean_luma: 40, dark_level: 30, max_dark_fraction: 0.6, flag_xmp_rejected: true })).toEqual({
      minMean: '40',
      darkLevel: '30',
      maxDarkPercent: '60'
    });
  });

  it('limits concurrency and keeps result order', async () => {
    let inFlight = 0;
    let peak = 0;
    const tasks = Array.from({ length: 10 }, (_, i) => async () => {
      inFlight++;
      peak = Math.max(peak, inFlight);
      await new Promise((r) => setTimeout(r, 1));
      inFlight--;
      if (i === 3) throw new Error('x');
      return i;
    });
    const out = await limited(tasks, 3);
    expect(peak).toBeLessThanOrEqual(3);
    expect(out[0]).toEqual({ status: 'fulfilled', value: 0 });
    expect(out[3].status).toBe('rejected');
  });

  it('base64-encodes thumbnail bytes', () => {
    expect(toBase64(new Uint8Array([0xff, 0xd8, 0xff]))).toBe('/9j/');
  });

  it('has every decision toast text, singular and plural, in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const d of ['accept', 'reject', 'clear'] as const) {
        const id = `clean-done-${d}`;
        expect(t(id, { count: 1 })).not.toBe(id);
        expect(t(id, { count: 1 })).not.toBe(t(id, { count: 3 }));
      }
      expect(t('clean-selected', { count: 2 })).toContain('2');
    }
  });
});

const item = (id: number, status: ImageStatus | null, user_override = false): ReviewItem => ({
  id,
  relative_path: `p${id}.jpg`,
  normalized_datetime: null,
  status,
  user_override,
  reasons: [],
  mean_luma: null,
  median_luma: null,
  dark_fraction: null,
  histogram: [],
  xmp_rating: null,
  failure: null,
  current_path: null
});

describe('grid movement', () => {
  it('counts columns from tile tops', () => {
    expect(columnsFromTops([0, 0, 0, 100, 100])).toBe(3);
    expect(columnsFromTops([0, 0])).toBe(2);
    expect(columnsFromTops([])).toBe(1);
  });

  it('moves by one and by row, stopping at the edges', () => {
    expect(moveIndex(4, 'ArrowRight', 10, 3)).toBe(5);
    expect(moveIndex(4, 'ArrowLeft', 10, 3)).toBe(3);
    expect(moveIndex(4, 'ArrowDown', 10, 3)).toBe(7);
    expect(moveIndex(4, 'ArrowUp', 10, 3)).toBe(1);
    expect(moveIndex(0, 'ArrowLeft', 10, 3)).toBe(0);
    expect(moveIndex(9, 'ArrowRight', 10, 3)).toBe(9);
    expect(moveIndex(1, 'ArrowUp', 10, 3)).toBe(1);
    expect(moveIndex(9, 'ArrowDown', 10, 3)).toBe(9);
    expect(moveIndex(3, 'Home', 10, 3)).toBe(0);
    expect(moveIndex(3, 'End', 10, 3)).toBe(9);
  });

  it('goes to the last tile when Down leaves a short last row', () => {
    expect(moveIndex(5, 'ArrowDown', 8, 3)).toBe(7);
  });

  it('has no focus in an empty grid and clamps a stale index', () => {
    expect(moveIndex(0, 'ArrowRight', 0, 3)).toBe(-1);
    expect(moveIndex(50, 'ArrowLeft', 4, 2)).toBe(2);
    expect(moveIndex(1, 'ArrowDown', 5, 0)).toBe(2);
  });
});

describe('focus after removal', () => {
  it('keeps the focused tile when it stays', () => {
    expect(focusAfterRemoval([1, 2, 3], [1, 2, 3], 2)).toBe(2);
  });
  it('moves to the next survivor, else the previous, else nothing', () => {
    expect(focusAfterRemoval([1, 2, 3, 4], [1, 4], 2)).toBe(4);
    expect(focusAfterRemoval([1, 2, 3], [1, 2], 3)).toBe(2);
    expect(focusAfterRemoval([1, 2, 3], [], 2)).toBeNull();
  });
  it('falls back to the first tile for an unknown or missing focus', () => {
    expect(focusAfterRemoval([1, 2], [2], null)).toBe(2);
    expect(focusAfterRemoval([1, 2], [2], 99)).toBe(2);
  });
  it('finds the inspector neighbour', () => {
    expect(neighbourId([1, 2, 3], 2)).toBe(3);
    expect(neighbourId([1, 2, 3], 3)).toBe(2);
    expect(neighbourId([1], 1)).toBeNull();
    expect(neighbourId([], 1)).toBeNull();
  });
});

describe('decisions and their undo snapshot', () => {
  it('refuses unanalysed and quarantined photographs', () => {
    for (const d of ['accept', 'reject', 'clear'] as const) {
      expect(decisionApplies(item(1, null), d)).toBe(false);
      expect(decisionApplies(item(2, 'quarantined', true), d)).toBe(false);
    }
  });
  it('skips decisions that change nothing', () => {
    expect(decisionApplies(item(1, 'eligible', true), 'accept')).toBe(false);
    expect(decisionApplies(item(1, 'rejected'), 'reject')).toBe(false);
    expect(decisionApplies(item(1, 'flagged'), 'clear')).toBe(false);
    expect(decisionApplies(item(1, 'flagged'), 'accept')).toBe(true);
  });
  it('derives the previous decision', () => {
    expect(previousDecision(item(1, 'flagged'))).toBe('clear');
    expect(previousDecision(item(1, 'eligible', true))).toBe('accept');
    expect(previousDecision(item(1, 'rejected', true))).toBe('reject');
  });
  it('plans a bulk decision with one undo group per previous decision', () => {
    const items = [item(1, 'flagged'), item(2, 'eligible', true), item(3, 'rejected', true), item(4, 'quarantined'), item(5, null)];
    const plan = planDecision(items, [1, 2, 3, 4, 5, 6, 1], 'reject');
    expect(plan.ids).toEqual([1, 2]);
    expect(plan.undo).toEqual([
      { decision: 'clear', ids: [1] },
      { decision: 'accept', ids: [2] }
    ]);
  });
  it('plans nothing when nothing applies', () => {
    expect(planDecision([item(1, 'flagged')], [1], 'clear')).toEqual({ ids: [], undo: [] });
    expect(planDecision([], [1], 'accept')).toEqual({ ids: [], undo: [] });
  });
});
