/**
 * The page shown, kept outside App so any page can send the user to another one (for
 * example "Contact sheet of this selection"). Held in memory only.
 */
export type Page = 'home' | 'cleaning' | 'organize' | 'sampling' | 'adjust' | 'export' | 'sheets' | 'settings';

/** Sidebar and tab bar order: the workflow, the tools, then Settings. Home is the Import step. */
export const NAV_ORDER: readonly Page[] = ['home', 'cleaning', 'sampling', 'adjust', 'export', 'organize', 'sheets', 'settings'];

let active = $state<Page>('home');
let importRequested = $state(false);
/** Whether the next page swap moves focus to the new page's heading (not reactive). */
let focusHeading = false;

export const nav = {
  get active(): Page {
    return active;
  },
  /**
   * Show `page`. Focus moves to its heading once it is drawn, unless the user chose it from
   * the sidebar or the tab bar (`focus: false`), whose link keeps focus.
   */
  go(page: Page, { focus = true }: { focus?: boolean } = {}): void {
    if (page !== active) focusHeading = focus;
    active = page;
  },
  /** Taken once by the page swap: true when it should focus the new heading. */
  takeFocusRequest(): boolean {
    const f = focusHeading;
    focusHeading = false;
    return f;
  },
  /** True once the menu asked for the folder picker; Home takes it and opens the picker. */
  get importRequested(): boolean {
    return importRequested;
  },
  requestImport(): void {
    importRequested = true;
    if (active !== 'home') focusHeading = true;
    active = 'home';
  },
  clearImportRequest(): void {
    importRequested = false;
  }
};
