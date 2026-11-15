import { describe, expect, it } from 'vitest';

import { DEFAULT_CHOICE, ERASE_FAILURES, eraseFailureText, erasedItems, folderName, targetsOf } from './erase';
import { setLocale, t } from './i18n';
import type { EraseInventory } from './ipc/app';

const folder = (exists: boolean, files = 0, bytes = 0) => ({ path: '/p', exists, files, bytes });
const inventory: EraseInventory = {
  project_data: folder(true, 4, 2_000_000),
  exports: folder(true, 2, 50_000_000),
  quarantine: folder(false),
  copies: { folders: ['/work'], files: 0, bytes: 0, moved: 3 }
};

describe('erase helpers', () => {
  it('has text for every refusal and choice in both locales', () => {
    for (const locale of ['en', 'fr-CA'] as const) {
      setLocale(locale);
      const texts = new Set(ERASE_FAILURES.map((f) => eraseFailureText(f)));
      expect(texts.size).toBe(ERASE_FAILURES.length);
      expect(eraseFailureText('new-thing')).toBe(eraseFailureText(null));
      for (const key of ['erase-section', 'erase-choice-project', 'erase-choice-exports', 'erase-choice-copies', 'erase-choice-quarantine', 'erase-no-project', 'erase-nothing-chosen']) {
        expect(t(key)).not.toBe(key);
      }
      expect(t('erase-choice-quarantine-hint', { count: 1, size: '1 kB' })).not.toBe(t('erase-choice-quarantine-hint', { count: 5, size: '1 kB' }));
    }
  });

  it('sends only chosen targets that exist on disk', () => {
    const all = { settings: false, project_data: true, exports: true, quarantine: true, copies: true };
    expect(targetsOf(all, inventory)).toEqual({ project_data: true, exports: true, quarantine: false, copies: false });
    expect(targetsOf(all, null)).toEqual({ project_data: false, exports: false, quarantine: false, copies: false });
  });

  it('lists what the dialog will delete', () => {
    setLocale('en');
    expect(erasedItems(DEFAULT_CHOICE, inventory, '/Volumes/K/trip')).toEqual([t('erase-item-settings'), expect.stringContaining('trip')]);
    expect(erasedItems({ ...DEFAULT_CHOICE, settings: false, project_data: false }, inventory, '/x')).toEqual([]);
    expect(folderName('C:\\Photos\\2024\\')).toBe('2024');
  });
});
