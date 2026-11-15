/**
 * The deflicker or stabilization analysis being run, kept outside the page so leaving
 * and coming back does not lose it (progress, Stop, result). One at a time, as in the
 * core. Held in memory only.
 */
import { cancelAdjustment, startDeflicker, startStabilization } from '../ipc/app';
import type { AdjustProgress, DeflickerConfig, FlickerRun, StabilizationRun, StabilizeConfig } from '../ipc/app';
import type { IpcError } from '../ipc/invoke';

export type AdjustKind = 'deflicker' | 'stabilize';

interface AdjustState {
  root: string;
  kind: AdjustKind | null;
  token: number;
  running: boolean;
  cancelling: boolean;
  progress: AdjustProgress | null;
  /** The run just made, or `null` after a cancel. */
  result: { kind: AdjustKind; run: FlickerRun | StabilizationRun | null } | null;
  error: IpcError | null;
}

let state = $state<AdjustState>({
  root: '',
  kind: null,
  token: 0,
  running: false,
  cancelling: false,
  progress: null,
  result: null,
  error: null
});

export const adjustJob = {
  get state(): AdjustState {
    return state;
  },
  dismiss(): void {
    state.result = null;
    state.error = null;
  }
};

let lastToken = 0;

export async function runAdjustment(
  root: string,
  kind: AdjustKind,
  run: number,
  config: DeflickerConfig | StabilizeConfig
): Promise<void> {
  if (state.running) return;
  lastToken = Math.max(lastToken + 1, Date.now());
  const token = lastToken;
  state = { ...state, root, kind, token, running: true, cancelling: false, progress: null, result: null, error: null };
  const onProgress = (p: AdjustProgress) => {
    if (state.token === token) state.progress = p;
  };
  try {
    const made =
      kind === 'deflicker'
        ? await startDeflicker(root, run, config as DeflickerConfig, token, onProgress)
        : await startStabilization(root, run, config as StabilizeConfig, token, onProgress);
    if (state.token === token) state.result = { kind, run: made };
  } catch (err) {
    if (state.token === token) state.error = err as IpcError;
  } finally {
    if (state.token === token) {
      state.running = false;
      state.cancelling = false;
    }
  }
}

export async function stopAdjustment(): Promise<void> {
  if (!state.running || state.cancelling) return;
  state.cancelling = true;
  try {
    await cancelAdjustment(state.token);
  } catch {
    state.cancelling = false;
  }
}
