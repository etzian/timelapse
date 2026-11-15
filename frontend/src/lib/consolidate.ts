/**
 * Presentation helpers for the Phase 6 Organize page. Data values from the core
 * (operations, naming rules, statuses, refusal reasons) map to Fluent ids here; nothing
 * in this file is user-visible text.
 */
import { t } from './i18n';
import type { ConsolidationConfig, ConsolidationRecord, Naming, Operation, PlannedItem } from './ipc/app';

export const OPERATIONS: readonly Operation[] = ['copy', 'move'];
export const NAMINGS: readonly Naming[] = ['preserve', 'original', 'folder-prefix', 'sequence'];

/** Reasons of the core's `Error::Consolidation { reason }`. */
export const CONSOLIDATION_FAILURES = [
  'destination-invalid',
  'destination-overlaps-collection',
  'destination-unwritable',
  'insufficient-space',
  'plan-changed',
  'nothing-to-do',
  'busy',
  'unknown-consolidation',
  'already-undone'
] as const;

export function consolidationFailureText(reason: string | null): string {
  const known = reason !== null && (CONSOLIDATION_FAILURES as readonly string[]).includes(reason);
  return t('error-consolidation', { reason: known ? reason : 'other' });
}

/** A dry run is only valid for exactly the configuration it was made with. */
export function sameConfig(a: ConsolidationConfig | null | undefined, b: ConsolidationConfig | null | undefined): boolean {
  return (
    !!a &&
    !!b &&
    a.destination === b.destination &&
    a.operation === b.operation &&
    a.naming === b.naming &&
    (a.sampling_run ?? null) === (b.sampling_run ?? null)
  );
}

/** How a planned item is shown: its target joined under the destination, or why it is skipped. */
export function itemNote(item: PlannedItem): string {
  switch (item.action.kind) {
    case 'place':
      return item.renamed ? t('organize-item-renamed') : '';
    case 'duplicate':
      return t('organize-item-duplicate');
    case 'already-present':
      return t('organize-item-already-present');
    case 'unsafe':
      return t('organize-item-unsafe');
  }
}

/** `destination` + `target`, with the destination's own separator. */
export function joinTarget(destination: string, target: string): string {
  const sep = destination.includes('\\') && !destination.includes('/') ? '\\' : '/';
  const base = destination.endsWith(sep) ? destination.slice(0, -1) : destination;
  return `${base}${sep}${target.split('/').join(sep)}`;
}

export function statusVariant(s: ConsolidationRecord['status']): 'success' | 'info' | 'warning' | 'default' | 'error' {
  switch (s) {
    case 'completed':
      return 'success';
    case 'running':
      return 'info';
    case 'partial':
      return 'warning';
    case 'cancelled':
      return 'default';
    default:
      return 'error';
  }
}

/** An undo is offered while something placed is left to take back. */
export function canUndo(r: ConsolidationRecord): boolean {
  return r.status !== 'running' && r.placed > 0 && r.undo_status !== 'undone';
}
