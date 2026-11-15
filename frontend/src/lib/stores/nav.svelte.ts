/**
 * The page shown, kept outside App so any page can send the user to another one (for
 * example "Contact sheet of this selection"). Held in memory only.
 */
export type Page = 'home' | 'cleaning' | 'organize' | 'sampling' | 'export' | 'sheets' | 'settings';
export const PAGES: readonly Page[] = ['home', 'cleaning', 'organize', 'sampling', 'export', 'sheets', 'settings'];

let active = $state<Page>('home');

export const nav = {
  get active(): Page {
    return active;
  },
  go(page: Page): void {
    active = page;
  }
};
