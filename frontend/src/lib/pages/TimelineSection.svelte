<script lang="ts">
  import { untrack } from 'svelte';
  import { Badge, Breadcrumb, Button, Card, EmptyState, SectionLabel, Skeleton, Stat, StatGrid, Tabs } from '@etzianco/ui';

  import { bucketPeriod, finer } from '../contactSheet';
  import { formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { detectTimelineGaps, getThumbnail, getTimeline, identicalTimestamps, timelineDistribution, timelinePeriod } from '../ipc/app';
  import type { BackfillReport, DistributionBucket, GapReport, Granularity, IdenticalReport, TimelineEntry, TimelinePage } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { openContactSheet } from '../stores/contactSheet.svelte';
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

  /** Photographs of the chosen period shown at a time. */
  const PERIOD_PAGE = 48;
  /** Identical-time groups listed. */
  const IDENTICAL_LIMIT = 20;

  /** A period zoomed into: its bounds, its label, and the granularity it was a bar of. */
  interface Zoom {
    from: string;
    to: string;
    label: string;
    granularity: Granularity;
  }

  let zoom = $state<Zoom[]>([]);
  let selected = $state<Omit<Zoom, 'granularity'> | null>(null);
  let periodPage = $state<TimelinePage | null>(null);
  let periodError = $state('');
  let periodThumbs = $state<Record<number, string>>({});
  let identical = $state<IdenticalReport | null>(null);

  const range = $derived(zoom.at(-1) ?? null);
  const summary = $derived(backfill.summary);
  const maxCount = $derived(Math.max(0, ...(buckets ?? []).map((b) => b.count)));
  const granularityItems = $derived(
    (['hour', 'day', 'week', 'month'] as const).map((key) => ({ key, label: t(GRANULARITY_KEYS[key]) }))
  );
  const crumbs = $derived([
    { label: t('timeline-zoom-all'), onclick: zoom.length > 0 ? () => zoomTo(0) : undefined },
    ...zoom.map((z, i) => ({ label: z.label, onclick: i < zoom.length - 1 ? () => zoomTo(i + 1) : undefined }))
  ]);

  async function load(): Promise<void> {
    error = '';
    try {
      const [timeline, gaps, same] = await Promise.all([getTimeline(root), detectTimelineGaps(root), identicalTimestamps(root, IDENTICAL_LIMIT)]);
      entries = timeline;
      gapReport = gaps;
      identical = same;
      shown = PAGE;
    } catch (err) {
      error = errorText(err as IpcError);
    }
  }

  // Only the latest request may land: an older answer would label bars with the wrong
  // granularity, and a click on them would pick the wrong period.
  let distSeq = 0;
  async function loadDistribution(g: Granularity): Promise<void> {
    // Refresh keeps the previous bars visible until the new ones arrive (GUIDELINES §4).
    const seq = ++distSeq;
    const forRoot = root;
    distLoading = true;
    distError = '';
    try {
      const b = await timelineDistribution(forRoot, g, range?.from, range?.to);
      if (seq === distSeq && forRoot === root) buckets = b;
    } catch (err) {
      if (seq === distSeq && forRoot === root) distError = errorText(err as IpcError);
    } finally {
      if (seq === distSeq) distLoading = false;
    }
  }

  /** A bar picks its period; a bar that can be split zooms into it at the next granularity. */
  function pickBucket(b: DistributionBucket): void {
    const p = bucketPeriod(b.bucket, granularity);
    if (!p) return;
    const label = formatBucket(b.bucket, granularity);
    selected = { ...p, label };
    void loadPeriod(0);
    const f = finer(granularity);
    if (f) {
      zoom = [...zoom, { ...p, label, granularity }];
      granularity = f;
      void loadDistribution(f);
    }
  }

  /** Back to the level with `n` zoom steps (0 = the whole collection). */
  function zoomTo(n: number): void {
    const back = zoom[n];
    if (!back) return;
    zoom = zoom.slice(0, n);
    granularity = back.granularity;
    const last = zoom.at(-1);
    selected = last ? { from: last.from, to: last.to, label: last.label } : null;
    if (selected) void loadPeriod(0);
    else periodPage = null;
    void loadDistribution(granularity);
  }

  // One request at a time per period; a newer period (or a reload) supersedes the rest.
  let periodSeq = 0;
  let periodLoading = $state(false);
  async function loadPeriod(offset: number): Promise<void> {
    const p = selected;
    if (!p) return;
    if (offset > 0 && periodLoading) return;
    const seq = ++periodSeq;
    const forRoot = root;
    periodError = '';
    periodLoading = true;
    if (offset === 0) periodPage = null;
    try {
      const page = await timelinePeriod(forRoot, p.from, p.to, offset, PERIOD_PAGE);
      if (seq !== periodSeq || selected !== p || forRoot !== root) return;
      const before = periodPage?.entries ?? [];
      // Append only what follows what is shown, so an entry never appears twice.
      periodPage = offset === 0 ? page : offset === before.length ? { total: page.total, entries: [...before, ...page.entries] } : periodPage;
      void loadPeriodThumbs(page.entries, p);
    } catch (err) {
      if (seq === periodSeq && selected === p) periodError = errorText(err as IpcError);
    } finally {
      if (seq === periodSeq) periodLoading = false;
    }
  }

  async function loadPeriodThumbs(list: TimelineEntry[], p: typeof selected): Promise<void> {
    const forRoot = root;
    for (let i = 0; i < list.length; i += 6) {
      if (selected !== p || forRoot !== root) return;
      const batch = list.slice(i, i + 6).filter((e) => !(e.id in periodThumbs));
      const urls = await Promise.all(batch.map((e) => getThumbnail(forRoot, e.id, 256).catch(() => '')));
      if (forRoot !== root) return;
      const next = { ...periodThumbs };
      batch.forEach((e, k) => (next[e.id] = urls[k]));
      periodThumbs = next;
    }
  }

  $effect(() => {
    void root;
    void backfill;
    entries = null;
    gapReport = null;
    buckets = null;
    identical = null;
    zoom = [];
    selected = null;
    periodPage = null;
    periodThumbs = {};
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
      <Breadcrumb items={crumbs} />
      <p class="app-timeline__note">{finer(granularity) ? t('timeline-zoom-help') : t('timeline-pick-help')}</p>
      <ol class="app-timeline__bars" class:app-timeline__bars--refreshing={distLoading} aria-busy={distLoading}>
        {#each buckets as b (b.bucket)}
          <li>
            <button
              type="button"
              class="app-timeline__bar-button"
              class:app-timeline__bar-button--selected={selected?.label === formatBucket(b.bucket, granularity)}
              aria-pressed={selected?.label === formatBucket(b.bucket, granularity)}
              onclick={() => pickBucket(b)}
            >
              <span class="app-timeline__bar-label">{formatBucket(b.bucket, granularity)}</span>
              <span class="app-timeline__bar-track" aria-hidden="true">
                <span class="app-timeline__bar" style:width="{barPercent(b.count, maxCount)}%"></span>
              </span>
              <span class="app-timeline__bar-count">{formatNumber(b.count)}</span>
            </button>
          </li>
        {/each}
      </ol>
    {/if}

    {#if selected}
      <section class="app-timeline__period" aria-label={t('timeline-period-title', { period: selected.label })}>
        <h3 class="app-timeline__heading">{t('timeline-period-title', { period: selected.label })}</h3>
        {#if periodError}
          <div role="alert" class="app-timeline__error">
            <p>{periodError}</p>
            <Button onclick={() => loadPeriod(0)}>{t('timeline-retry')}</Button>
          </div>
        {:else if periodPage === null}
          <Skeleton lines={3} label={t('timeline-loading')} />
        {:else}
          {@const chosen = selected}
          <div class="app-timeline__period-head">
            <span class="app-timeline__note">{t('timeline-period-count', { count: periodPage.total, photos: formatNumber(periodPage.total) })}</span>
            <Button variant="secondary" onclick={() => openContactSheet({ kind: 'period', from: chosen.from, to: chosen.to })}>
              {t('timeline-period-sheet')}
            </Button>
          </div>
          <ul class="app-timeline__thumbs">
            {#each periodPage.entries as e (e.id)}
              <li title={e.relative_path}>
                {#if periodThumbs[e.id]}
                  <img src={periodThumbs[e.id]} alt={e.relative_path} />
                {:else}
                  <span class="app-timeline__thumb-placeholder" aria-hidden="true"></span>
                {/if}
                <span class="app-timeline__thumb-time">{e.epoch_ms !== null ? formatInstant(e.epoch_ms) : ''}</span>
              </li>
            {/each}
          </ul>
          {#if periodPage.entries.length < periodPage.total}
            <Button onclick={() => loadPeriod(periodPage?.entries.length ?? 0)} disabled={periodLoading}>{t('timeline-show-more')}</Button>
          {/if}
        {/if}
      </section>
    {/if}

    {#if identical}
      {#if identical.groups === 0}
        <p class="app-timeline__note">{t('timeline-identical-none')}</p>
      {:else}
        <h3 class="app-timeline__heading">{t('timeline-identical-title', { groups: identical.groups, photos: formatNumber(identical.photos) })}</h3>
        <p class="app-timeline__note">{t('timeline-identical-help')}</p>
        <ul class="app-timeline__gaps">
          {#each identical.largest as g (g.normalized_datetime)}
            <li>
              <strong>{formatInstant(g.epoch_ms)}</strong>
              · {t('timeline-identical-count', { count: g.count, photos: formatNumber(g.count) })}
              <p class="app-timeline__note app-timeline__sample">
                {g.sample.map((e) => e.relative_path).join(', ')}{g.count > g.sample.length ? ', …' : ''}
              </p>
            </li>
          {/each}
        </ul>
        {#if identical.groups > identical.largest.length}
          <p class="app-timeline__note">{t('timeline-identical-more', { count: identical.groups - identical.largest.length })}</p>
        {/if}
      {/if}
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
  .app-timeline__bar-button {
    display: grid;
    grid-template-columns: minmax(8rem, auto) minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--ez-space-sm);
    width: 100%;
    padding: var(--ez-space-2xs) var(--ez-space-xs);
    border: 0;
    border-radius: var(--ez-radius-sm);
    background: none;
    color: inherit;
    font: inherit;
    text-align: start;
    cursor: pointer;
  }
  .app-timeline__bar-button:hover,
  .app-timeline__bar-button--selected {
    background: var(--ez-color-border-soft);
  }
  .app-timeline__period {
    margin-top: var(--ez-space-md);
  }
  .app-timeline__period-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--ez-space-sm);
  }
  .app-timeline__thumbs {
    list-style: none;
    margin: var(--ez-space-sm) 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(7rem, 1fr));
    gap: var(--ez-space-sm);
  }
  .app-timeline__thumbs li {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
  }
  .app-timeline__thumbs img,
  .app-timeline__thumb-placeholder {
    display: block;
    width: 100%;
    aspect-ratio: 4 / 3;
    object-fit: cover;
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-border-soft);
  }
  .app-timeline__thumb-time {
    font-size: var(--ez-fs-xs);
    font-variant-numeric: tabular-nums;
    color: var(--ez-color-text-muted);
  }
  .app-timeline__sample {
    overflow-wrap: anywhere;
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
