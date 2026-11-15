import { beforeAll, describe, expect, it, vi } from 'vitest';

import { RECENT_KEY } from '../project';
import { ALL_STORAGE_KEYS, STORAGE_KEYS } from '../storageKeys';

/** A Storage stand-in: the test environment has no localStorage. */
function memoryStorage(): Storage {
  const data = new Map<string, string>();
  return {
    get length() {
      return data.size;
    },
    key: (i: number) => Array.from(data.keys())[i] ?? null,
    getItem: (k: string) => data.get(k) ?? null,
    setItem: (k: string, v: string) => void data.set(k, String(v)),
    removeItem: (k: string) => void data.delete(k),
    clear: () => data.clear()
  };
}

const storage = memoryStorage();
const ours = () => Array.from({ length: storage.length }, (_, i) => storage.key(i)!).filter((k) => k.startsWith('timelapse'));

let settings: typeof import('./settings.svelte');
let project: typeof import('./project.svelte');

beforeAll(async () => {
  vi.stubGlobal('localStorage', storage);
  const attrs = new Map<string, string>();
  vi.stubGlobal('document', {
    documentElement: { dataset: {}, getAttribute: (k: string) => attrs.get(k) ?? null, setAttribute: (k: string, v: string) => void attrs.set(k, v) }
  });
  vi.stubGlobal('navigator', { languages: ['en-CA'], language: 'en-CA', userAgent: 'test', platform: 'test' });
  settings = await import('./settings.svelte');
  project = await import('./project.svelte');
});

describe('storage key registry', () => {
  it('holds the recent-projects key defined once in project.ts, and no duplicate', () => {
    expect(STORAGE_KEYS.recent).toBe(RECENT_KEY);
    expect(new Set(ALL_STORAGE_KEYS).size).toBe(ALL_STORAGE_KEYS.length);
    for (const k of ALL_STORAGE_KEYS) expect(k.startsWith('timelapse')).toBe(true);
  });
});

describe('eraseLocalData', () => {
  it('leaves no timelapse key in localStorage, and does not touch other keys', () => {
    settings.loadSettings();
    settings.applyLanguage('fr-CA');
    settings.applyDensity('compact');
    settings.applyTheme('dark');
    project.rememberProject('/Volumes/K/trip');
    storage.setItem(STORAGE_KEYS.sidebar, '{"tools":false}');
    storage.setItem('another-app.key', 'kept');

    // Every key the app wrote is in the registry: otherwise the erase would miss it.
    expect(ours().sort()).toEqual([...ALL_STORAGE_KEYS].sort());

    settings.eraseLocalData();

    expect(ours()).toEqual([]);
    expect(storage.getItem('another-app.key')).toBe('kept');
    expect(project.project.recent).toEqual([]);
    // The defaults are applied again without being written back.
    expect(settings.settings.language).toBe('system');
    expect(settings.settings.density).toBe('comfortable');
    expect(settings.settings.theme).toBe('auto');
  });

  it('would miss a key that is not in the registry (the check above can fail)', () => {
    storage.setItem('timelapse.unregistered', 'x');
    settings.eraseLocalData();
    expect(ours()).toEqual(['timelapse.unregistered']);
    storage.removeItem('timelapse.unregistered');
  });
});
