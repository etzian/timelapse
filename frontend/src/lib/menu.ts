/**
 * The native menu as plain data, so the labels, the shortcuts and the page each one reaches
 * can be tested without a window. `menuNative.ts` turns this into the real menu.
 * Every action here also has a visible control; the shortcuts are a convenience.
 */
import { NAV_ORDER } from './stores/nav.svelte';
import type { Page } from './stores/nav.svelte';

/** The workflow steps ⌘1–⌘5 reach, in NAV_ORDER: Import, Clean, Select, Adjust, Export. */
export const WORKFLOW_PAGES: readonly Page[] = NAV_ORDER.slice(0, 5);

export type MenuAction = { type: 'go'; page: Page } | { type: 'open-folder' };

export type PredefinedKind = 'Undo' | 'Redo' | 'Cut' | 'Copy' | 'Paste' | 'SelectAll' | 'Minimize' | 'Maximize' | 'CloseWindow' | 'Hide' | 'HideOthers' | 'ShowAll' | 'Quit' | 'Fullscreen';

export type MenuEntry =
  | { kind: 'item'; id: string; text: string; accelerator: string; action: MenuAction }
  | { kind: 'predefined'; item: PredefinedKind }
  | { kind: 'separator' };

export interface MenuSpec {
  id: string;
  text: string;
  entries: MenuEntry[];
}

export const SHORTCUT_SETTINGS = 'CmdOrCtrl+,';
export const SHORTCUT_OPEN = 'CmdOrCtrl+O';
export const shortcutForStep = (index: number): string => `CmdOrCtrl+${index + 1}`;

const sep: MenuEntry = { kind: 'separator' };
const pre = (item: PredefinedKind): MenuEntry => ({ kind: 'predefined', item });

/** The whole menu bar. `label` is the Fluent `t()` (or a stand-in in tests). */
export function buildMenuModel(label: (id: string) => string): MenuSpec[] {
  const steps: MenuEntry[] = WORKFLOW_PAGES.map((page, i) => ({
    kind: 'item',
    id: `go-${page}`,
    text: label(`nav-${page}`),
    accelerator: shortcutForStep(i),
    action: { type: 'go', page }
  }));
  return [
    {
      id: 'app',
      text: label('app-name'),
      entries: [
        { kind: 'item', id: 'go-settings', text: label('menu-settings'), accelerator: SHORTCUT_SETTINGS, action: { type: 'go', page: 'settings' } },
        sep,
        pre('Hide'),
        pre('HideOthers'),
        pre('ShowAll'),
        sep,
        pre('Quit')
      ]
    },
    {
      id: 'file',
      text: label('menu-file'),
      entries: [{ kind: 'item', id: 'open-folder', text: label('menu-open-folder'), accelerator: SHORTCUT_OPEN, action: { type: 'open-folder' } }, sep, pre('CloseWindow')]
    },
    { id: 'edit', text: label('menu-edit'), entries: [pre('Undo'), pre('Redo'), sep, pre('Cut'), pre('Copy'), pre('Paste'), pre('SelectAll')] },
    { id: 'workflow', text: label('nav-group-workflow'), entries: steps },
    { id: 'window', text: label('menu-window'), entries: [pre('Minimize'), pre('Maximize'), pre('Fullscreen')] }
  ];
}

/** Every item that carries a shortcut, flattened. */
export function shortcuts(model: MenuSpec[]): { accelerator: string; action: MenuAction }[] {
  return model.flatMap((m) => m.entries).flatMap((e) => (e.kind === 'item' ? [{ accelerator: e.accelerator, action: e.action }] : []));
}
