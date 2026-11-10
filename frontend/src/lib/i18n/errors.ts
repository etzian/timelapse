import { t } from './store.svelte';
import type { IpcError } from '../ipc/invoke';

/** Text for an error that crossed IPC as `{ kind, args }`; unknown kinds get the generic message. */
export function errorText(err: IpcError): string {
  const id = `error-${err.kind}`;
  return t(id, err.args ?? {});
}
