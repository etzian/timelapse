/** Numbers and dates go through `Intl` in the active locale, never typed into a string. */
import { getLocale } from './store.svelte';

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(getLocale(), options).format(value);
}

export function formatDate(value: Date | number, options: Intl.DateTimeFormatOptions = { dateStyle: 'long' }): string {
  return new Intl.DateTimeFormat(getLocale(), options).format(value);
}
