<script lang="ts">
  import { Button } from '@etzianco/ui';

  import { t } from '../i18n';
  import { nextStep } from '../pipeline';
  import type { StepId } from '../pipeline';
  import { nav } from '../stores/nav.svelte';
  import { pipeline } from '../stores/pipeline.svelte';
  import { project } from '../stores/project.svelte';

  /** The workflow step this page is; the action names the one after it. */
  let { step }: { step: StepId } = $props();

  const next = $derived(nextStep(step, pipeline.figures, pipeline.running));
</script>

{#if project.root}
  <div class="app-next">
    <Button onclick={() => nav.go(next.target)} disabled={next.reason !== null}>
      {t('next-action', { step: t(`nav-${next.target}`) })}
    </Button>
    {#if next.reason}
      <p class="app-next__reason">{t(`next-reason-${next.reason}`)}</p>
    {/if}
  </div>
{/if}

<style>
  .app-next {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-sm) var(--ez-space-md);
    justify-content: flex-end;
  }
  .app-next__reason {
    margin: 0;
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
  }
</style>
