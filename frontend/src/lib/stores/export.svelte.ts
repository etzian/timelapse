/**
 * The running export, kept outside the page so leaving and coming back to Export does
 * not lose it (progress, Cancel, result). Held in memory only.
 */
import { cancelExport, listExports, startExport } from '../ipc/app';
import type { ExportProgress, ExportRecord, ExportSettings } from '../ipc/app';
import type { IpcError } from '../ipc/invoke';

interface ExportState {
  root: string;
  token: number;
  exporting: boolean;
  cancelling: boolean;
  progress: ExportProgress | null;
  result: ExportRecord | null;
  error: IpcError | null;
  history: ExportRecord[] | null;
}

let state = $state<ExportState>({
  root: '',
  token: 0,
  exporting: false,
  cancelling: false,
  progress: null,
  result: null,
  error: null,
  history: null
});

export const exportJob = {
  get state(): ExportState {
    return state;
  },
  set history(h: ExportRecord[] | null) {
    state.history = h;
  },
  dismiss(): void {
    state.result = null;
    state.error = null;
  }
};

let lastToken = 0;

export async function runExport(root: string, run: number, settings: ExportSettings): Promise<void> {
  if (state.exporting) return;
  lastToken = Math.max(lastToken + 1, Date.now());
  const token = lastToken;
  state = { ...state, root, token, exporting: true, cancelling: false, progress: null, result: null, error: null };
  try {
    const rec = await startExport(root, run, settings, token, (p) => {
      if (state.token === token) state.progress = p;
    });
    if (state.token === token) state.result = rec;
    if (state.root === root) state.history = await listExports(root);
  } catch (err) {
    if (state.token === token) state.error = err as IpcError;
  } finally {
    if (state.token === token) {
      state.exporting = false;
      state.cancelling = false;
    }
  }
}

export async function stopExport(): Promise<void> {
  if (!state.exporting || state.cancelling) return;
  state.cancelling = true;
  try {
    await cancelExport(state.token);
  } catch {
    state.cancelling = false;
  }
}
