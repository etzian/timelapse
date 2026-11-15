/**
 * The page shown, kept outside App so any page can send the user to another one (for
 * example "Contact sheet of this selection"). Held in memory only.
 */
export type Page = 'home' | 'cleaning' | 'organize' | 'sampling' | 'adjust' | 'export' | 'sheets' | 'settings';

/** Sidebar and tab bar order: the workflow, the tools, then Settings. Home is the Import step. */
export const NAV_ORDER: readonly Page[] = ['home', 'cleaning', 'sampling', 'adjust', 'export', 'organize', 'sheets', 'settings'];

let active = $state<Page>('home');
let importRequested = $state(false);

export const nav = {
  get active(): Page {
    return active;
  },
  go(page: Page): void {
    active = page;
  },
  /** True once the menu asked for the folder picker; Home takes it and opens the picker. */
  get importRequested(): boolean {
    return importRequested;
  },
  requestImport(): void {
    importRequested = true;
    active = 'home';
  },
  clearImportRequest(): void {
    importRequested = false;
  }
};
