/**
 * Settings → Erase all data (ADR-026): what the user can choose, and the lines the
 * confirmation lists. Values come from the core's inventory; text from Fluent.
 */
import { formatBytes } from './export';
import { t } from './i18n';
import type { EraseInventory, EraseTargets } from './ipc/app';

export interface EraseChoice extends EraseTargets {
  settings: boolean;
}

export const DEFAULT_CHOICE: EraseChoice = { settings: true, project_data: true, exports: false, quarantine: false, copies: false };

/** Reasons of the core's `Error::Erase { reason }`. */
export const ERASE_FAILURES = ['busy', 'quarantine-inside', 'remove-failed'] as const;

export function eraseFailureText(reason: string | null): string {
  const known = reason !== null && (ERASE_FAILURES as readonly string[]).includes(reason);
  return t('error-erase', { reason: known ? reason : 'other' });
}

/** The last folder name of a path, for the copy that names the open folder. */
export function folderName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

/** What can be chosen: a target with nothing on disk cannot. */
export function available(inv: EraseInventory | null): EraseTargets {
  return {
    project_data: !!inv?.project_data.exists,
    exports: !!inv?.exports.exists,
    quarantine: !!inv?.quarantine.exists,
    copies: (inv?.copies.files ?? 0) > 0
  };
}

/** The chosen targets that exist (what is sent to the core). */
export function targetsOf(choice: EraseChoice, inv: EraseInventory | null): EraseTargets {
  const can = available(inv);
  return {
    project_data: choice.project_data && can.project_data,
    exports: choice.exports && can.exports,
    quarantine: choice.quarantine && can.quarantine,
    copies: choice.copies && can.copies
  };
}

/** One line per chosen item, for the confirmation dialog. */
export function erasedItems(choice: EraseChoice, inv: EraseInventory | null, root: string): string[] {
  const tg = targetsOf(choice, inv);
  const out: string[] = [];
  if (choice.settings) out.push(t('erase-item-settings'));
  if (inv && tg.project_data) out.push(t('erase-item-project', { folder: folderName(root), size: formatBytes(inv.project_data.bytes) }));
  if (inv && tg.exports) out.push(t('erase-item-exports', { count: inv.exports.files, size: formatBytes(inv.exports.bytes) }));
  if (inv && tg.copies) out.push(t('erase-item-copies', { count: inv.copies.files, size: formatBytes(inv.copies.bytes) }));
  if (inv && tg.quarantine) out.push(t('erase-item-quarantine', { count: inv.quarantine.files, size: formatBytes(inv.quarantine.bytes) }));
  return out;
}
