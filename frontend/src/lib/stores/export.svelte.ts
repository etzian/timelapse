/**
 * The running export, kept outside the page so leaving and coming back to Export does
 * not lose it (progress, Cancel, result). Held in memory only.
 */
import { cancelExport, getSamplingItems, getThumbnail, listExports, startExport } from '../ipc/app';
import type { ExportProgress, ExportRecord, ExportSettings } from '../ipc/app';
import type { IpcError } from '../ipc/invoke';

interface ExportState {
  root: string;
  token: number;
  exporting: boolean;
  cancelling: boolean;
  progress: ExportProgress | null;
  /** Thumbnail of the frame being encoded (data URL), refreshed a few times a second. */
  thumb: string | null;
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
  thumb: null,
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
  state = { ...state, root, token, exporting: true, cancelling: false, progress: null, thumb: null, result: null, error: null };
  try {
    const rec = await startExport(root, run, settings, token, (p) => {
      if (state.token === token) {
        state.progress = p;
        void followFrame(root, run, token, p);
      }
    });
    if (state.token === token) state.result = rec;
    if (state.root === root) state.history = await listExports(root);
  } catch (err) {
    if (state.token === token) state.error = err as IpcError;
  } finally {
    if (state.token === token) {
      state.thumb = null;
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

/** At most one thumbnail request at a time, and not more than one in THUMB_EVERY_MS. */
const THUMB_EVERY_MS = 800;
let thumbBusy = false;
let thumbAt = 0;
let thumbTotal: { key: string; total: number } | null = null;

async function selectionSize(root: string, run: number): Promise<number> {
  const key = `${root}|${run}`;
  if (thumbTotal?.key !== key) thumbTotal = { key, total: (await getSamplingItems(root, run, 0, 1)).total };
  return thumbTotal.total;
}

/** Shows the photograph the encoder is at: the selection item at the same fraction of the way. */
async function followFrame(root: string, run: number, token: number, p: ExportProgress): Promise<void> {
  const now = Date.now();
  if (thumbBusy || now - thumbAt < THUMB_EVERY_MS || p.total <= 0 || p.done <= 0) return;
  thumbBusy = true;
  thumbAt = now;
  try {
    const size = await selectionSize(root, run);
    if (size <= 0) return;
    const index = Math.min(size - 1, Math.floor((Math.min(p.done, p.total) / p.total) * size));
    const page = await getSamplingItems(root, run, index, 1);
    const item = page.items[0];
    if (!item) return;
    const url = await getThumbnail(root, item.image_id, 256);
    if (state.token === token && state.exporting) state.thumb = url;
  } catch {
    // The picture is a nicety; the bar carries the information.
  } finally {
    thumbBusy = false;
  }
}

/** The poster of a finished video: the thumbnail of its selection's middle frame, or null. */
export async function posterFor(root: string, run: number): Promise<string | null> {
  try {
    const size = (await getSamplingItems(root, run, 0, 1)).total;
    if (size <= 0) return null;
    const page = await getSamplingItems(root, run, Math.floor(size / 2), 1);
    const item = page.items[0];
    return item ? await getThumbnail(root, item.image_id, 256) : null;
  } catch {
    return null;
  }
}
