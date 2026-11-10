/**
 * Every `.ftl` file under `locales/<code>/`, imported raw. `en` is the base
 * language and `fr-CA` the legally required one (Charter of the French
 * Language); adding a language means adding a folder and one entry here.
 */
const en = import.meta.glob('../../../../locales/en/*.ftl', { query: '?raw', import: 'default', eager: true });
const frCA = import.meta.glob('../../../../locales/fr-CA/*.ftl', { query: '?raw', import: 'default', eager: true });

export type Locale = 'fr-CA' | 'en';
export const SUPPORTED_LOCALES: readonly Locale[] = ['fr-CA', 'en'];

export const FTL_SOURCES: Record<Locale, string[]> = {
  en: Object.values(en) as string[],
  'fr-CA': Object.values(frCA) as string[]
};
