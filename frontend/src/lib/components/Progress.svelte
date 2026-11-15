<script lang="ts">
  import type { Snippet } from 'svelte';

  import { formatNumber, t } from '../i18n';
  import { fraction, observe, remainingSeconds } from '../progress';
  import type { RateState } from '../progress';
  import { formatDuration } from '../timeline';

  /**
   * A determinate bar with "done of total" and the time left once the rate is stable. Announcements
   * of a job starting or ending come from the pipeline store, so nothing here is a live region.
   */
  let {
    done,
    total,
    label,
    thumb = null,
    children
  }: {
    done: number;
    total: number;
    /** What is running, for the bar's accessible name. */
    label: string;
    /** The photograph being worked on, shown beside the bar (decorative). */
    thumb?: string | null;
    children?: Snippet;
  } = $props();

  let rate: RateState | null = null;
  let left = $state<number | null>(null);

  $effect(() => {
    const d = done;
    const n = total;
    rate = observe(rate, d, performance.now());
    left = remainingSeconds(rate, d, n);
  });
</script>

<div class="app-progress">
  {#if thumb}<img class="app-progress__thumb" src={thumb} alt="" />{/if}
  <div class="app-progress__body">
    <progress max={Math.max(total, 1)} value={Math.min(done, Math.max(total, 1))} aria-label={label}></progress>
    <p class="app-progress__text">
      <span>{t('clean-progress', { done: formatNumber(done), total: formatNumber(total) })}</span>
      <span class="app-progress__pct">{formatNumber(fraction(done, total), { style: 'percent' })}</span>
      {#if left !== null}<span>{t('progress-remaining', { time: formatDuration(left) })}</span>{/if}
    </p>
    {@render children?.()}
  </div>
</div>

<style>
  .app-progress {
    display: flex;
    align-items: center;
    gap: var(--ez-space-sm);
    flex: 1 1 14rem;
    min-width: 0;
  }
  .app-progress__thumb {
    width: var(--app-progress-thumb);
    aspect-ratio: 3 / 2;
    object-fit: cover;
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-border-soft);
    flex: none;
  }
  .app-progress__body {
    flex: 1 1 auto;
    min-width: 0;
  }
  .app-progress progress {
    width: 100%;
  }
  .app-progress__text {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-xs);
    margin: var(--ez-space-2xs) 0 0;
    font-size: var(--ez-fs-sm);
    color: var(--ez-color-text-muted);
  }
</style>
