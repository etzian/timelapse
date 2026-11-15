/**
 * User settings kept on the device (`localStorage`), applied before first paint
 * where it matters (the recent-projects list is kept by the project store). These are the only things the app stores; they are listed in
 * `legal/data-map.yaml` and erased by "Erase all data" in Settings.
 */
import { configureTheme, getPreference, resolveTheme, setPreference } from '@etzianco/ui';

import { resolveSystemLocale, setLocale, SUPPORTED_LOCALES, type Locale } from '../i18n';
import { ALL_STORAGE_KEYS, STORAGE_KEYS } from '../storageKeys';
import { clearRecent } from './project.svelte';

const KEY_LANGUAGE = STORAGE_KEYS.language;
const KEY_DENSITY = STORAGE_KEYS.density;

// @etzianco/ui does not re-export this type from its entry point yet.
export type ThemePreference = ReturnType<typeof getPreference>;
export type Language = 'system' | Locale;
export type Density = 'comfortable' | 'compact';

function read(key: string): string | null {
  try {
    return localStorage.getItem(key);
  } catch {
    return null;
  }
}
function write(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    /* storage disabled: the setting lasts for this session only */
  }
}

let language = $state<Language>('system');
let density = $state<Density>('comfortable');
let theme = $state<ThemePreference>('auto');

export const settings = {
  get language() {
    return language;
  },
  get density() {
    return density;
  },
  get theme() {
    return theme;
  }
};

// The `use*` functions apply a value; the `apply*` ones are the user's choice and also store it.
// Loading (at launch, after an erase) only applies, so an erase leaves nothing behind.
function useLanguage(next: Language): void {
  language = next;
  setLocale(next === 'system' ? resolveSystemLocale(navigator.languages) : next);
}

function useDensity(next: Density): void {
  density = next;
  document.documentElement.dataset.density = next;
}

export function applyLanguage(next: Language): void {
  useLanguage(next);
  write(KEY_LANGUAGE, next);
}

export function applyDensity(next: Density): void {
  useDensity(next);
  write(KEY_DENSITY, next);
}

export function applyTheme(next: ThemePreference): void {
  theme = next;
  setPreference(next);
}

/** Load and apply everything before the first mount. */
export function loadSettings(): void {
  configureTheme({ storageKey: STORAGE_KEYS.theme });
  theme = getPreference();
  resolveTheme(theme);
  const lang = read(KEY_LANGUAGE);
  useLanguage(lang === 'system' || SUPPORTED_LOCALES.includes(lang as Locale) ? (lang as Language) : 'system');
  useDensity(read(KEY_DENSITY) === 'compact' ? 'compact' : 'comfortable');
}

/** Erase everything this app keeps on the device, then re-apply the defaults. */
export function eraseLocalData(): void {
  for (const k of ALL_STORAGE_KEYS) {
    try {
      localStorage.removeItem(k);
    } catch {
      /* nothing to erase */
    }
  }
  clearRecent();
  loadSettings();
}
