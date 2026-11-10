import { describe, expect, it } from 'vitest';

import { resolveSystemLocale, setLocale, t } from './store.svelte';

describe('i18n', () => {
  it('resolves any French preference to fr-CA and everything else to en', () => {
    expect(resolveSystemLocale(['fr-FR', 'en'])).toBe('fr-CA');
    expect(resolveSystemLocale(['en-US'])).toBe('en');
    expect(resolveSystemLocale([])).toBe('en');
  });

  it('translates the same message in both locales, with arguments', () => {
    setLocale('en');
    const en = t('error-validation-too-long', { max: 40 });
    setLocale('fr-CA');
    const fr = t('error-validation-too-long', { max: 40 });
    expect(en).toContain('40');
    expect(fr).toContain('40');
    expect(fr).not.toBe(en);
  });
});
