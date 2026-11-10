/**
 * User settings kept on the device (`localStorage`), applied before first paint
 * where it matters. These are the only things the app stores; they are listed in
 * `legal/data-map.yaml` and erased by "Erase all data" in Settings.
 */
import { configureTheme, getPreference, setPreference } from '@etzianco/ui';

import { resolveSystemLocale, setLocale, SUPPORTED_LOCALES, type Locale } from '../i18n';

const KEY_LANGUAGE = 'timelapse.language';
const KEY_DENSITY = 'timelapse.density';

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

export function applyLanguage(next: Language): void {
  language = next;
  write(KEY_LANGUAGE, next);
  setLocale(next === 'system' ? resolveSystemLocale(navigator.languages) : next);
}

export function applyDensity(next: Density): void {
  density = next;
  write(KEY_DENSITY, next);
  document.documentElement.dataset.density = next;
}

export function applyTheme(next: ThemePreference): void {
  theme = next;
  setPreference(next);
}

/** Load and apply everything before the first mount. */
export function loadSettings(): void {
  configureTheme({ storageKey: 'timelapse.theme' });
  theme = getPreference();
  setPreference(theme);
  const lang = read(KEY_LANGUAGE);
  applyLanguage(lang === 'system' || SUPPORTED_LOCALES.includes(lang as Locale) ? (lang as Language) : 'system');
  applyDensity(read(KEY_DENSITY) === 'compact' ? 'compact' : 'comfortable');
}

/** Erase everything this app keeps on the device, then re-apply the defaults. */
export function eraseLocalData(): void {
  for (const k of [KEY_LANGUAGE, KEY_DENSITY, 'timelapse.theme']) {
    try {
      localStorage.removeItem(k);
    } catch {
      /* nothing to erase */
    }
  }
  loadSettings();
}
