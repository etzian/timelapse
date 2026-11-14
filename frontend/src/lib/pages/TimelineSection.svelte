<script lang="ts">
  import { untrack } from 'svelte';
  import { Badge, Button, Card, EmptyState, SectionLabel, Skeleton, Stat, StatGrid, Tabs } from '@etzianco/ui';

  import { formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { detectTimelineGaps, getTimeline, timelineDistribution } from '../ipc/app';
  import type { BackfillReport, DistributionBucket, GapReport, Granularity, TimelineEntry } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import {
    CONFIDENCE_KEYS,
    CONFIDENCE_VARIANT,
    GRANULARITY_KEYS,
    SOURCE_KEYS,
    SOURCE_ORDER,
    barPercent,
    formatBucket,
    formatDuration,
    formatInstant
  } from '../timeline';

  let { root, backfill }: { root: string; backfill: BackfillReport } = $props();

  /** Rows rendered at a time (GUIDELINES §12: paginate long lists rather than render them all). */
  const PAGE = 200;

  let entries = $state<TimelineEntry[] | null>(null);
  let gapReport = $state<GapReport | null>(null);
  let buckets = $state<DistributionBucket[] | null>(null);
  let granularity = $state<Granularity>('day');
  let shown = $state(PAGE);
  let error = $state('');
  let distError = $state('');
  let distLoading = $state(false);

  const summary = $derived(backfill.summary);
  const maxCount = $derived(Math.max(0, ...(buckets ?? []).map((b) => b.count)));
  const granularityItems = $derived(
    (['hour', 'day', 'month'] as const).map((key) => ({ key, label: t(GRANULARITY_KEYS[key]) }))
  );

  async function load(): Promise<void> {
    error = '';
    try {
      const [timeline, gaps] = await Promise.all([getTimeline(root), detectTimelineGaps(root)]);
      entries = timeline;
      gapReport = gaps;
      shown = PAGE;
    } catch (err) {
      error = errorText(err as IpcError);
    }
  }

  async function loadDistribution(g: Granularity): Promise<void> {
    // Refresh keeps the previous bars visible until the new ones arrive (GUIDELINES §4).
    distLoading = true;
    distError = '';
    try {
      buckets = await timelineDistribution(root, g);
    } catch (err) {
      distError = errorText(err as IpcError);
    } finally {
      distLoading = false;
    }
  }

  $effect(() => {
    void root;
    void backfill;
    entries = null;
    gapReport = null;
    buckets = null;
    // Re-run on a new root / backfill only; a granularity change reloads the bars alone.
    untrack(() => {
      void load();
      void loadDistribution(granularity);
    });
  });

  function changeGranularity(key: string): void {
    granularity = key as Granularity;
    void loadDistribution(granularity);
  }
</script>

<Card class="app-timeline">
  <SectionLabel text={t('timeline-title')} />

  <StatGrid>
    <Stat label={t('timeline-summary-resolved')} value={formatNumber(summary.resolved)} />
    <Stat
      label={t('timeline-summary-unresolved')}
      value={formatNumber(summary.unresolved)}
      tone={summary.unresolved > 0 ? 'warning' : 'neutral'}
    />
    <Stat
      label={t('timeline-summary-low-confidence')}
      value={formatNumber(summary.low_confidence)}
      tone={summary.low_confidence > 0 ? 'warning' : 'neutral'}
    />
    <Stat label={t('timeline-summary-tz-assumed')} value={formatNumber(summary.tz_assumed)} />
  </StatGrid>

  <h3 class="app-timeline__heading">{t('timeline-by-source')}</h3>
  <dl class="app-timeline__sources">
    {#each SOURCE_ORDER as source (source)}
      <div><dt>{t(SOURCE_KEYS[source])}</dt><dd>{formatNumber(summary.by_source[source])}</dd></div>
    {/each}
  </dl>
  <p class="app-timeline__note">{t('timeline-dates-utc')}</p>

  {#if error}
    <div role="alert" class="app-timeline__error">
      <p>{error}</p>
      <Button onclick={load}>{t('timeline-retry')}</Button>
    </div>
  {:else if summary.resolved === 0}
    <EmptyState title={t('timeline-empty-title')} description={t('timeline-empty-description')} compact />
  {:else}
    <h3 class="app-timeline__heading">{t('timeline-distribution')}</h3>
    <Tabs items={granularityItems} active={granularity} label={t('timeline-distribution')} onchange={changeGranularity} size="sm" />
    {#if distError}
      <div role="alert" class="app-timeline__error">
        <p>{distError}</p>
        <Button onclick={() => loadDistribution(granularity)}>{t('timeline-retry')}</Button>
      </div>
    {:else if buckets === null}
      <Skeleton lines={4} label={t('timeline-loading')} />
    {:else}
      <ol class="app-timeline__bars" class:app-timeline__bars--refreshing={distLoading} aria-busy={distLoading}>
        {#each buckets as b (b.bucket)}
          <li>
            <span class="app-timeline__bar-label">{formatBucket(b.bucket, granularity)}</span>
            <span class="app-timeline__bar-track" aria-hidden="true">
              <span class="app-timeline__bar" style:width="{barPercent(b.count, maxCount)}%"></span>
            </span>
            <span class="app-timeline__bar-count">{formatNumber(b.count)}</span>
          </li>
        {/each}
      </ol>
    {/if}

    {#if gapReport === null}
      <Skeleton lines={2} label={t('timeline-loading')} />
    {:else}
      <h3 class="app-timeline__heading">
        {gapReport.gaps.length > 0
          ? t('timeline-gaps-title', { count: gapReport.gaps.length })
          : t('timeline-gaps-none', { threshold: formatDuration(gapReport.threshold_seconds) })}
      </h3>
      {#if gapReport.gaps.length > 0}
        <ul class="app-timeline__gaps">
          {#each gapReport.gaps as g (g.before.id)}
            <li>
              {t('timeline-gap', {
                from: formatInstant(g.before.epoch_ms ?? 0),
                to: formatInstant(g.after.epoch_ms ?? 0),
                duration: formatDuration(g.seconds)
              })}
              {#if g.source_changed}
                <p class="app-timeline__note">{t('timeline-gap-source-changed')}</p>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  {/if}

  {#if entries === null && !error}
    <Skeleton lines={6} label={t('timeline-loading')} />
  {:else if entries && entries.length > 0}
    <h3 class="app-timeline__heading">{t('timeline-list-title')}</h3>
    <ol class="app-timeline__list">
      {#each entries.slice(0, shown) as e (e.id)}
        <li class="app-timeline__row">
          <span class="app-timeline__path" title={e.relative_path}>{e.relative_path}</span>
          <span class="app-timeline__when">
            {e.epoch_ms !== null ? formatInstant(e.epoch_ms) : t('timeline-unresolved')}
          </span>
          <span class="app-timeline__badges">
            {#if e.source}<Badge>{t(SOURCE_KEYS[e.source])}</Badge>{/if}
            {#if e.confidence}
              <Badge variant={CONFIDENCE_VARIANT[e.confidence]}>{t(CONFIDENCE_KEYS[e.confidence])}</Badge>
            {:else}
              <Badge variant="error">{t('timeline-unresolved')}</Badge>
            {/if}
            {#if e.tz_assumed}<Badge variant="warning">{t('timeline-tz-assumed')}</Badge>{/if}
          </span>
        </li>
      {/each}
    </ol>
    {#if entries.length > shown}
      <p class="app-timeline__note">
        {t('timeline-list-truncated', { shown: formatNumber(shown), total: formatNumber(entries.length) })}
      </p>
      <Button onclick={() => (shown += PAGE)}>{t('timeline-show-more')}</Button>
    {/if}
  {/if}
</Card>

<style>
  .app-timeline__heading {
    font-size: var(--ez-fs-md);
    margin: var(--ez-space-md) 0 var(--ez-space-xs);
  }
  .app-timeline__sources {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: var(--ez-space-2xs) var(--ez-space-md);
    margin: 0;
  }
  .app-timeline__sources dt {
    color: var(--ez-color-text-muted);
  }
  .app-timeline__sources dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }
  .app-timeline__note {
    font-size: var(--ez-fs-sm);
    color: var(--ez-color-text-muted);
    margin: var(--ez-space-xs) 0;
  }
  .app-timeline__error {
    color: var(--ez-color-error);
  }
  .app-timeline__bars,
  .app-timeline__gaps,
  .app-timeline__list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .app-timeline__bars li {
    display: grid;
    grid-template-columns: minmax(8rem, auto) minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--ez-space-sm);
    padding: var(--ez-space-2xs) 0;
  }
  .app-timeline__bars--refreshing {
    opacity: 0.6;
  }
  .app-timeline__bar-label,
  .app-timeline__bar-count {
    font-size: var(--ez-fs-sm);
    font-variant-numeric: tabular-nums;
  }
  .app-timeline__bar-track {
    display: block;
    height: var(--ez-space-xs);
    border-radius: var(--ez-radius-pill);
    background: var(--ez-color-border-soft);
  }
  .app-timeline__bar {
    display: block;
    height: 100%;
    border-radius: var(--ez-radius-pill);
    background: var(--ez-color-accent);
  }
  .app-timeline__gaps li {
    padding: var(--ez-space-2xs) 0;
  }
  .app-timeline__row {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(0, 1.5fr) auto;
    align-items: center;
    gap: var(--ez-space-sm);
    padding: var(--ez-space-2xs) 0;
    border-bottom: 1px solid var(--ez-color-border);
    content-visibility: auto;
    contain-intrinsic-size: auto 2.25rem;
  }
  .app-timeline__path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .app-timeline__when {
    font-variant-numeric: tabular-nums;
    font-size: var(--ez-fs-sm);
  }
  .app-timeline__badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-2xs);
    justify-content: flex-end;
  }
</style>
