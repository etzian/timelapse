<script lang="ts">
  import { t } from '../i18n';
  import { nextStep } from '../pipeline';
  import type { StepId } from '../pipeline';
  import { nav } from '../stores/nav.svelte';
  import { pipeline } from '../stores/pipeline.svelte';
  import { project } from '../stores/project.svelte';

  /** The workflow step this page is; the action names the one after it. */
  let { step }: { step: StepId } = $props();

  const next = $derived(nextStep(step, pipeline.figures, pipeline.running));
  const reasonId = $props.id();

  function go(): void {
    if (next.reason === null) nav.go(next.target);
  }
</script>

{#if project.root}
  <div class="app-next">
    <!-- aria-disabled, not disabled: the button stays reachable and says why it waits. -->
    <button
      type="button"
      class="ez-btn ez-btn--primary ez-btn--md"
      aria-disabled={next.reason !== null}
      aria-describedby={next.reason ? reasonId : undefined}
      onclick={go}
    >
      {t('next-action', { step: t(`nav-${next.target}`) })}
    </button>
    {#if next.reason}
      <p class="app-next__reason" id={reasonId}>{t(`next-reason-${next.reason}`)}</p>
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
