/** `selectionLabel` bound to the active locale. */
import type { RunSummary } from './ipc/app';
import { formatDate, formatNumber, t } from './i18n';
import { selectionLabel as build } from './sampling';

export function runLabel(run: Pick<RunSummary, 'config' | 'selected_count' | 'created_at'>): string {
  return build(run, {
    t,
    number: (n) => formatNumber(n),
    date: (ms, withYear) => formatDate(ms, withYear ? { day: 'numeric', month: 'short', year: 'numeric' } : { day: 'numeric', month: 'short' })
  });
}

/** The same label with the stored number as a secondary detail. */
export function runLabelWithId(run: Pick<RunSummary, 'id' | 'config' | 'selected_count' | 'created_at'>): string {
  return t('selection-with-id', { label: runLabel(run), id: run.id });
}
