<script lang="ts">
  import { Badge, Modal, SectionLabel, Skeleton } from '@etzianco/ui';

  import { analysisFailureText, decisionApplies, formatLuma, formatPercent, reasonText, STATUS_KEYS, STATUS_VARIANT } from '../clean';
  import { formatNumber, t } from '../i18n';
  import { getThumbnail } from '../ipc/app';
  import type { Decision, ReviewItem } from '../ipc/app';

  let {
    root,
    item,
    index,
    total,
    busy = false,
    onprevious,
    onnext,
    ondecide,
    onclose
  }: {
    root: string;
    /** The photograph shown, or null when the viewer is closed. */
    item: ReviewItem | null;
    /** Position in the current filter, 0-based. */
    index: number;
    total: number;
    busy?: boolean;
    onprevious: () => void;
    onnext: () => void;
    ondecide: (decision: Decision, fromKeyboard?: boolean) => void;
    onclose: () => void;
  } = $props();

  let url = $state('');
  const id = $derived(item?.id ?? null);
  const maxBin = (h: number[]) => Math.max(1, ...h);
  const statusText = (s: ReviewItem['status']) => (s ? t(STATUS_KEYS[s]) : t('clean-status-unanalyzed'));

  // The large preview follows the photograph; a response for a photograph left behind is dropped.
  $effect(() => {
    const current = id;
    url = '';
    if (current === null) return;
    void getThumbnail(root, current, 1024).then(
      (u) => {
        if (id === current) url = u;
      },
      () => {
        if (id === current) url = '';
      }
    );
  });

  // Buttons stay focusable when they cannot act (aria-disabled), so a decision or a step to
  // the first photograph never drops focus out of the dialog.
  const canPrevious = $derived(index > 0);
  const canNext = $derived(index < total - 1);
  const can = (d: Decision) => !busy && item !== null && decisionApplies(item, d);
  function choose(d: Decision): void {
    if (can(d)) ondecide(d);
  }

  function onkeydown(e: KeyboardEvent): void {
    if (!item || e.defaultPrevented || e.metaKey || e.ctrlKey || e.altKey) return;
    const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    const decide = (d: Decision) => {
      e.preventDefault();
      if (!busy && decisionApplies(item!, d)) ondecide(d, true);
    };
    if (key === 'ArrowLeft' && index > 0) {
      e.preventDefault();
      onprevious();
    } else if (key === 'ArrowRight' && index < total - 1) {
      e.preventDefault();
      onnext();
    } else if (key === 'a') decide('accept');
    else if (key === 'r') decide('reject');
    else if (key === 'u') decide('clear');
  }
</script>

<svelte:window onkeydown={item ? onkeydown : undefined} />

<Modal open={item !== null} title={item?.relative_path ?? ''} size="lg" class="app-inspector" {onclose}>
  {#if item}
    {#if url}
      <img class="app-inspector__image" src={url} alt={t('clean-inspect', { path: item.relative_path })} />
    {:else if item.failure}
      <p>{analysisFailureText(item.failure)}</p>
    {:else}
      <Skeleton variant="rect" size="16rem" label={t('clean-loading')} />
    {/if}
    <div class="app-inspector__nav">
      <button type="button" class="ez-btn ez-btn--secondary ez-btn--sm" aria-disabled={!canPrevious} onclick={() => canPrevious && onprevious()}>{t('clean-previous')}</button>
      <span class="app-inspector__position" aria-live="polite">{t('clean-inspector-position', { index: formatNumber(index + 1), total: formatNumber(total) })}</span>
      <button type="button" class="ez-btn ez-btn--secondary ez-btn--sm" aria-disabled={!canNext} onclick={() => canNext && onnext()}>{t('clean-next')}</button>
    </div>
    <p class="app-inspector__badges">
      <Badge variant={item.status ? STATUS_VARIANT[item.status] : 'default'}>{statusText(item.status)}</Badge>
      {#if item.user_override}<Badge>{t('clean-user-decision')}</Badge>{/if}
    </p>
    {#each item.reasons as r, i (i)}
      <p class="app-inspector__reason">{reasonText(r)}</p>
    {/each}
    <dl class="app-inspector__details">
      {#if item.mean_luma !== null}
        <div><dt>{t('clean-detail-mean')}</dt><dd>{formatLuma(item.mean_luma)}</dd></div>
        <div><dt>{t('clean-detail-median')}</dt><dd>{formatLuma(item.median_luma ?? 0)}</dd></div>
        <div><dt>{t('clean-detail-dark')}</dt><dd>{formatPercent(item.dark_fraction ?? 0)}</dd></div>
      {/if}
      {#if item.xmp_rating !== null}
        <div><dt>{t('clean-detail-xmp')}</dt><dd>{formatNumber(item.xmp_rating)}</dd></div>
      {/if}
      {#if item.current_path}
        <div><dt>{t('clean-detail-location')}</dt><dd class="app-inspector__path">{item.current_path}</dd></div>
      {/if}
    </dl>
    {#if item.histogram.length > 0}
      <SectionLabel text={t('clean-detail-histogram')} />
      <div class="app-inspector__histogram" aria-hidden="true">
        {#each item.histogram as bin, i (i)}
          <span style:height="{Math.round((bin / maxBin(item.histogram)) * 100)}%"></span>
        {/each}
      </div>
    {/if}
    <p class="app-inspector__note">{t('clean-inspector-keys')}</p>
  {/if}
  {#snippet footer()}
    {#if item}
      <button type="button" class="ez-btn ez-btn--secondary ez-btn--md" aria-disabled={!can('accept')} onclick={() => choose('accept')}>{t('clean-accept')}</button>
      <button type="button" class="ez-btn ez-btn--danger ez-btn--md" aria-disabled={!can('reject')} onclick={() => choose('reject')}>{t('clean-reject')}</button>
      <button type="button" class="ez-btn ez-btn--ghost ez-btn--md" aria-disabled={!can('clear')} onclick={() => choose('clear')}>{t('clean-clear')}</button>
    {/if}
  {/snippet}
</Modal>

<style>
  .app-inspector__image {
    display: block;
    max-width: 100%;
    max-height: 60vh;
    margin: 0 auto var(--ez-space-md);
  }
  .app-inspector__badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-2xs);
    margin: 0 0 var(--ez-space-xs);
  }
  .app-inspector__reason,
  .app-inspector__note {
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-2xs) 0;
  }
  /* The Modal is portaled to <body>, so its own box is styled through :global. 720 px (lg)
     is the design system's width; it never runs past the window, and the footer wraps. */
  :global(.ez-modal.app-inspector) {
    max-width: var(--app-modal-max-width);
  }
  :global(.ez-modal.app-inspector .ez-modal__footer) {
    flex-wrap: wrap;
  }
  .app-inspector__nav {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: var(--ez-space-sm);
    margin: 0 0 var(--ez-space-sm);
  }
  .app-inspector__position {
    color: var(--ez-color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .app-inspector__details {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: var(--ez-space-2xs) var(--ez-space-md);
    margin: var(--ez-space-sm) 0 var(--ez-space-md);
  }
  .app-inspector__details dt {
    color: var(--ez-color-text-muted);
  }
  .app-inspector__details dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }
  .app-inspector__path {
    overflow-wrap: anywhere;
  }
  .app-inspector__histogram {
    display: flex;
    align-items: flex-end;
    gap: 1px;
    height: 4rem;
  }
  .app-inspector__histogram span {
    flex: 1;
    min-height: 1px;
    background: var(--ez-color-accent);
  }
</style>
