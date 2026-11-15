<script lang="ts">
  import { Button } from '@etzianco/ui';

  import { t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { openOutput, revealOutput } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { openContactSheet } from '../stores/contactSheet.svelte';

  /** Play, show in Finder (folder elsewhere) and, optionally, Contact sheet for a finished export. */
  let { root, exportId, size = 'sm', sheet = true }: { root: string; exportId: number; size?: 'sm' | 'md'; sheet?: boolean } = $props();

  const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.userAgent);
  let failure = $state('');
  let busy = $state(false);

  async function run(action: typeof openOutput): Promise<void> {
    if (busy) return;
    const forRoot = root;
    busy = true;
    failure = '';
    try {
      await action(forRoot, 'export', exportId);
    } catch (err) {
      if (forRoot === root) failure = errorText(err as IpcError);
    } finally {
      busy = false;
    }
  }
</script>

<div class="app-output-actions">
  <!-- aria-disabled while a request runs, so the clicked button keeps focus; run() refuses. -->
  <button type="button" class="ez-btn ez-btn--primary ez-btn--{size}" aria-disabled={busy} onclick={() => run(openOutput)}>{t('output-play')}</button>
  <button type="button" class="ez-btn ez-btn--secondary ez-btn--{size}" aria-disabled={busy} onclick={() => run(revealOutput)}>
    {isMac ? t('output-reveal-mac') : t('output-reveal')}
  </button>
  {#if sheet}
    <Button {size} variant="secondary" onclick={() => openContactSheet({ kind: 'export', export: exportId })}>{t('export-sheet')}</Button>
  {/if}
</div>
{#if failure}<p role="alert" class="app-output-actions__error">{failure}</p>{/if}

<style>
  .app-output-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-xs);
  }
  .app-output-actions__error {
    color: var(--ez-color-error);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-xs) 0 0;
  }
</style>
