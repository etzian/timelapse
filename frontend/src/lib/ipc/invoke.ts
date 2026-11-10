/**
 * Typed `invoke`: every call gets a timeout and a typed error. A hung command
 * must never leave a permanent spinner (GUIDELINES §8.4, §6).
 */
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

/** The shape every command error crosses the boundary as; no English in it. */
export interface IpcError {
  kind: string;
  args?: Record<string, unknown>;
}

export class IpcTimeoutError extends Error {
  constructor(
    public readonly command: string,
    public readonly timeoutMs: number
  ) {
    super(`"${command}" did not respond within ${timeoutMs}ms`);
    this.name = 'IpcTimeoutError';
  }
}

const DEFAULT_TIMEOUT_MS = 10_000;

export async function invoke<T>(command: string, args?: Record<string, unknown>, timeoutMs = DEFAULT_TIMEOUT_MS): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new IpcTimeoutError(command, timeoutMs)), timeoutMs);
  });
  try {
    return await Promise.race([tauriInvoke<T>(command, args), timeout]);
  } catch (err) {
    if (err instanceof IpcTimeoutError) throw err;
    throw toIpcError(err);
  } finally {
    clearTimeout(timer);
  }
}

function toIpcError(err: unknown): IpcError {
  if (err && typeof err === 'object' && 'kind' in err && typeof err.kind === 'string') return err as IpcError;
  return { kind: 'generic' };
}
