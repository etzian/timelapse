import { describe, expect, it } from 'vitest';

import { CONSOLIDATION_FAILURES, NAMINGS, OPERATIONS, canUndo, consolidationFailureText, itemNote, joinTarget, sameConfig, statusVariant } from './consolidate';
import { FILE_FAILURES, fileFailureText } from './clean';
import { setLocale, t } from './i18n';
import type { ConsolidationConfig, ConsolidationRecord, PlannedItem } from './ipc/app';

const cfg: ConsolidationConfig = { destination: '/Volumes/Work/tl', operation: 'copy', naming: 'preserve', sampling_run: null };

function record(over: Partial<ConsolidationRecord> = {}): ConsolidationRecord {
  return {
    id: 1,
    config: cfg,
    plan: { planned: 3, duplicates: 0, already_present: 0, unsafe_names: 0, bytes: 30 },
    status: 'completed',
    undo_status: null,
    placed: 3,
    failed: 0,
    undone: 0,
    undo_failed: 0,
    created_at: 0,
    completed_at: 0,
    undone_at: null,
    ...over
  };
}

describe('consolidation helpers', () => {
  it('has text for every operation, naming rule, refusal and file failure in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      for (const o of OPERATIONS) {
        expect(t(`organize-operation-${o}`)).not.toBe(`organize-operation-${o}`);
        expect(t(`organize-run-${o}`, { count: 2 })).not.toBe(`organize-run-${o}`);
      }
      for (const n of NAMINGS) expect(t(`organize-naming-${n}-help`)).not.toBe(`organize-naming-${n}-help`);
      const texts = new Set(CONSOLIDATION_FAILURES.map((f) => consolidationFailureText(f)));
      expect(texts.size).toBe(CONSOLIDATION_FAILURES.length);
      expect(consolidationFailureText('something-new')).toBe(consolidationFailureText(null));
      for (const f of FILE_FAILURES) expect(fileFailureText(f)).not.toBe(t('files-failure-other'));
      for (const s of ['running', 'completed', 'partial', 'cancelled', 'interrupted']) {
        expect(t(`organize-status-${s}`)).not.toBe(`organize-status-${s}`);
      }
    }
  });

  it('only reuses a dry run for exactly the same settings', () => {
    expect(sameConfig(cfg, { ...cfg })).toBe(true);
    expect(sameConfig(cfg, { ...cfg, operation: 'move' })).toBe(false);
    expect(sameConfig(cfg, { ...cfg, sampling_run: 4 })).toBe(false);
    expect(sameConfig(cfg, { ...cfg, destination: '/Volumes/Work/other' })).toBe(false);
    expect(sameConfig(null, cfg)).toBe(false);
  });

  it('joins targets with the destination separator', () => {
    expect(joinTarget('/Volumes/Work/tl', '157gopro/G001.JPG')).toBe('/Volumes/Work/tl/157gopro/G001.JPG');
    expect(joinTarget('/Volumes/Work/tl/', 'a.jpg')).toBe('/Volumes/Work/tl/a.jpg');
    expect(joinTarget('D:\\work', '157gopro/G001.JPG')).toBe('D:\\work\\157gopro\\G001.JPG');
  });

  it('explains every skipped or renamed item', () => {
    setLocale('en');
    const item = (action: PlannedItem['action'], renamed = false): PlannedItem => ({ image_id: 1, relative_path: 'a.jpg', target: 'a.jpg', size: 1, action, renamed });
    expect(itemNote(item({ kind: 'place' }))).toBe('');
    for (const n of [item({ kind: 'place' }, true), item({ kind: 'duplicate', of: 2 }), item({ kind: 'already-present' }), item({ kind: 'unsafe' })]) {
      expect(itemNote(n)).not.toBe('');
    }
  });

  it('offers undo only while something placed is left to take back', () => {
    expect(canUndo(record())).toBe(true);
    expect(canUndo(record({ undo_status: 'partially-undone' }))).toBe(true);
    expect(canUndo(record({ undo_status: 'undone' }))).toBe(false);
    expect(canUndo(record({ placed: 0, status: 'cancelled' }))).toBe(false);
    expect(canUndo(record({ status: 'running' }))).toBe(false);
    expect(statusVariant('partial')).toBe('warning');
  });
});
