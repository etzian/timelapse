/**
 * The contact sheet being generated, kept outside the page so leaving and coming back
 * does not lose it (progress, Cancel, result), and the source another page asked for.
 * Held in memory only.
 */
import { cancelContactSheet, startContactSheet } from '../ipc/app';
import type { SheetLabels, SheetLayout, SheetProgress, SheetRecord, SheetSource } from '../ipc/app';
import type { IpcError } from '../ipc/invoke';
import { nav } from './nav.svelte';

interface SheetState {
  root: string;
  token: number;
  running: boolean;
  cancelling: boolean;
  progress: SheetProgress | null;
  result: SheetRecord | null;
  error: IpcError | null;
  /** A source chosen elsewhere (timeline, cleaning, sampling, export), taken by the page. */
  requested: SheetSource | null;
}

let state = $state<SheetState>({
  root: '',
  token: 0,
  running: false,
  cancelling: false,
  progress: null,
  result: null,
  error: null,
  requested: null
});

export const sheetJob = {
  get state(): SheetState {
    return state;
  },
  dismiss(): void {
    state.result = null;
    state.error = null;
  },
  /** The requested source, once: the page clears it when it takes it. */
  take(): SheetSource | null {
    const r = state.requested;
    state.requested = null;
    return r;
  }
};

/** Open the Contact sheets page with `source` chosen. */
export function openContactSheet(source: SheetSource): void {
  state.requested = source;
  nav.go('sheets');
}

let lastToken = 0;

/** `expected` is the photo count shown and printed; the core refuses if the source changed. */
export async function runContactSheet(
  root: string,
  source: SheetSource,
  layout: SheetLayout,
  labels: SheetLabels,
  expected: number
): Promise<void> {
  if (state.running) return;
  lastToken = Math.max(lastToken + 1, Date.now());
  const token = lastToken;
  state = { ...state, root, token, running: true, cancelling: false, progress: null, result: null, error: null };
  try {
    const rec = await startContactSheet(root, source, layout, labels, expected, token, (p) => {
      if (state.token === token) state.progress = p;
    });
    if (state.token === token) state.result = rec;
  } catch (err) {
    if (state.token === token) state.error = err as IpcError;
  } finally {
    if (state.token === token) {
      state.running = false;
      state.cancelling = false;
    }
  }
}

export async function stopContactSheet(): Promise<void> {
  if (!state.running || state.cancelling) return;
  state.cancelling = true;
  try {
    await cancelContactSheet(state.token);
  } catch {
    state.cancelling = false;
  }
}
