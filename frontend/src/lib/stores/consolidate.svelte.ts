/**
 * The running consolidation or undo, kept outside the page so leaving and coming back to
 * Organize does not lose it (progress, Cancel, result). Held in memory only.
 */
import { cancelConsolidation, listConsolidations, startConsolidation, undoConsolidation } from '../ipc/app';
import type { ConsolidationConfig, ConsolidationProgress, ConsolidationRecord } from '../ipc/app';
import type { IpcError } from '../ipc/invoke';

type JobKind = 'run' | 'undo';

interface JobState {
  root: string;
  token: number;
  kind: JobKind | null;
  busy: boolean;
  cancelling: boolean;
  progress: ConsolidationProgress | null;
  result: ConsolidationRecord | null;
  resultKind: JobKind | null;
  error: IpcError | null;
  history: ConsolidationRecord[] | null;
}

let state = $state<JobState>({
  root: '',
  token: 0,
  kind: null,
  busy: false,
  cancelling: false,
  progress: null,
  result: null,
  resultKind: null,
  error: null,
  history: null
});

export const consolidationJob = {
  get state(): JobState {
    return state;
  },
  set history(h: ConsolidationRecord[] | null) {
    state.history = h;
  },
  dismiss(): void {
    state.result = null;
    state.resultKind = null;
    state.error = null;
  }
};

let lastToken = 0;

async function track(root: string, kind: JobKind, work: (token: number, onProgress: (p: ConsolidationProgress) => void) => Promise<ConsolidationRecord>): Promise<void> {
  if (state.busy) return;
  lastToken = Math.max(lastToken + 1, Date.now());
  const token = lastToken;
  state = { ...state, root, token, kind, busy: true, cancelling: false, progress: null, result: null, resultKind: null, error: null };
  try {
    const rec = await work(token, (p) => {
      if (state.token === token) state.progress = p;
    });
    if (state.token === token) {
      state.result = rec;
      state.resultKind = kind;
    }
  } catch (err) {
    if (state.token === token) state.error = err as IpcError;
  } finally {
    if (state.root === root) {
      try {
        state.history = await listConsolidations(root);
      } catch {
        // The history stays as it was; the page offers a retry.
      }
    }
    if (state.token === token) {
      state.busy = false;
      state.cancelling = false;
      state.kind = null;
    }
  }
}

export function runConsolidation(root: string, config: ConsolidationConfig, fingerprint: string): Promise<void> {
  return track(root, 'run', (token, onProgress) => startConsolidation(root, config, fingerprint, token, onProgress));
}

export function runUndo(root: string, run: number): Promise<void> {
  return track(root, 'undo', (token, onProgress) => undoConsolidation(root, run, token, onProgress));
}

export async function stopConsolidation(): Promise<void> {
  if (!state.busy || state.cancelling) return;
  state.cancelling = true;
  try {
    await cancelConsolidation(state.token);
  } catch {
    state.cancelling = false;
  }
}
