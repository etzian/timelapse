import { t } from './store.svelte';
import type { IpcError } from '../ipc/invoke';

/** Text for an error that crossed IPC as `{ kind, args }`; unknown kinds get the generic message. */
export function errorText(err: IpcError): string {
  // Anything that is not a command error (a thrown JS error, say) is the generic message.
  const kind = err && typeof err === 'object' && typeof err.kind === 'string' ? err.kind : 'generic';
  return t(`error-${kind}`, err?.args ?? {});
}
