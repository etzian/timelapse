import { describe, expect, it } from 'vitest';

import { WORKFLOW_PAGES, buildMenuModel, shortcuts } from './menu';
import { NAV_ORDER } from './stores/nav.svelte';

const model = buildMenuModel((id) => `<${id}>`);

describe('menu model', () => {
  it('maps ⌘1–⌘5 to the workflow steps in NAV_ORDER', () => {
    expect(WORKFLOW_PAGES).toEqual(NAV_ORDER.slice(0, 5));
    expect(WORKFLOW_PAGES).toEqual(['home', 'cleaning', 'sampling', 'adjust', 'export']);
    const byKey = Object.fromEntries(shortcuts(model).map((s) => [s.accelerator, s.action]));
    WORKFLOW_PAGES.forEach((page, i) => expect(byKey[`CmdOrCtrl+${i + 1}`]).toEqual({ type: 'go', page }));
  });

  it('puts Settings on ⌘, and Open folder on ⌘O', () => {
    const byKey = Object.fromEntries(shortcuts(model).map((s) => [s.accelerator, s.action]));
    expect(byKey['CmdOrCtrl+,']).toEqual({ type: 'go', page: 'settings' });
    expect(byKey['CmdOrCtrl+O']).toEqual({ type: 'open-folder' });
  });

  it('has no duplicate shortcut and none on a bare grid key', () => {
    const keys = shortcuts(model).map((s) => s.accelerator);
    expect(new Set(keys).size).toBe(keys.length);
    for (const k of keys) {
      expect(k.startsWith('CmdOrCtrl+')).toBe(true);
      expect(['A', 'R', 'U', 'X', 'Space']).not.toContain(k);
    }
  });

  it('takes every label from the label function', () => {
    const texts = model.flatMap((m) => [m.text, ...m.entries.flatMap((e) => (e.kind === 'item' ? [e.text] : []))]);
    expect(texts.length).toBeGreaterThan(8);
    for (const x of texts) expect(x).toMatch(/^<[\w-]+>$/);
  });
});
