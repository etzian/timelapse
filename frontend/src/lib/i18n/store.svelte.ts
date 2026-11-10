/**
 * The Fluent bundle and `t()`, reactive to `setLocale`. A missing message
 * returns its id and logs an error rather than blanking the page; the
 * `ez i18n-check` gate keeps that from reaching users.
 */
import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle';

import { FTL_SOURCES, SUPPORTED_LOCALES, type Locale } from './messages';

function buildBundle(locale: Locale): FluentBundle {
  const bundle = new FluentBundle(locale, { useIsolating: false });
  for (const source of FTL_SOURCES[locale]) {
    const errors = bundle.addResource(new FluentResource(source));
    if (errors.length > 0) console.error(`i18n: errors adding a resource for "${locale}"`, errors);
  }
  return bundle;
}

let currentLocale = $state<Locale>('fr-CA');
let currentBundle = $state<FluentBundle>(buildBundle('fr-CA'));

export function getLocale(): Locale {
  return currentLocale;
}

/** Switch language at runtime: no reload. Keeps `<html lang>` in step. */
export function setLocale(next: Locale): void {
  if (next === currentLocale) return;
  currentLocale = next;
  currentBundle = buildBundle(next);
  if (typeof document !== 'undefined') document.documentElement.lang = next;
}

export function t(id: string, args?: Record<string, unknown>): string {
  const message = currentBundle.getMessage(id);
  if (!message?.value) {
    console.error(`i18n: missing message "${id}" for locale "${currentLocale}"`);
    return id;
  }
  const errors: Error[] = [];
  const text = currentBundle.formatPattern(message.value, args as Record<string, FluentVariable> | undefined, errors);
  if (errors.length > 0) console.error(`i18n: format errors for "${id}"`, errors);
  return text;
}

/** fr-CA when the OS prefers any French (there is no fr-FR text), otherwise en. */
export function resolveSystemLocale(preferred: readonly string[]): Locale {
  return preferred[0]?.toLowerCase().startsWith('fr') ? 'fr-CA' : 'en';
}

export { SUPPORTED_LOCALES };
export type { Locale };
