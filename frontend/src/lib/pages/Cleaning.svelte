<script lang="ts">
  import { Badge, Banner, Button, Card, EmptyState, Input, Modal, PageHeader, SectionLabel, Skeleton, Stat, StatGrid, Tabs, Toggle } from '@etzianco/ui';

  import {
    STATUS_KEYS,
    STATUS_VARIANT,
    analysisFailureText,
    fileFailureText,
    formatLuma,
    formatPercent,
    limited,
    parseThresholds,
    reasonText,
    thresholdsToForm
  } from '../clean';
  import { formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import {
    analyzeImages,
    cancelAnalysis,
    getCleanSettings,
    getReview,
    getThumbnail,
    quarantineImages,
    restoreImages,
    setCleanSettings,
    setDecision
  } from '../ipc/app';
  import type { AnalyzeProgress, Decision, ImageStatus, OpOutcome, ReviewItem, StatusCounts } from '../ipc/app';
  import { openContactSheet } from '../stores/contactSheet.svelte';
  import type { IpcError } from '../ipc/invoke';
  import { project } from '../stores/project.svelte';

  let { ongohome }: { ongohome: () => void } = $props();

  type Filter = 'flagged' | 'rejected' | 'quarantined' | 'all';
  const PAGE = 60;
  const THUMB_CONCURRENCY = 4;

  const root = $derived(project.root);

  let items = $state<ReviewItem[] | null>(null);
  let filter = $state<Filter>('flagged');
  let shown = $state(PAGE);
  let loadError = $state('');
  let actionError = $state('');

  let form = $state({ minMean: '', darkLevel: '', maxDarkPercent: '' });
  let flagXmp = $state(true);
  let formInvalid = $state(false);
  let savingSettings = $state(false);

  let progress = $state<AnalyzeProgress | null>(null);
  let analyzing = $state(false);
  let stopping = $state(false);
  let analyzeToken = 0;
  /** Set when a run was stopped: how many photographs are still to analyse. */
  let stoppedRemaining = $state<number | null>(null);

  let thumbs = $state<Record<number, string>>({});
  let thumbFailed = $state<Record<number, true>>({});
  let inspect = $state<ReviewItem | null>(null);
  let inspectUrl = $state('');

  let confirmOpen = $state(false);
  let moving = $state(false);

  // Bulk decisions: the photographs ticked on the current tab, and the decision to confirm.
  let selection = $state<Record<number, true>>({});
  let bulk = $state<Decision | null>(null);
  let deciding = $state(false);
  let outcome = $state<{ kind: 'quarantine' | 'restore'; ok: number; failures: OpOutcome[] } | null>(null);

  const counts = $derived<StatusCounts>(
    (items ?? []).reduce<StatusCounts>(
      (c, i) => {
        if (i.status) c[i.status] += 1;
        else c.unanalyzed += 1;
        return c;
      },
      { eligible: 0, flagged: 0, rejected: 0, quarantined: 0, unanalyzed: 0 }
    )
  );
  const visible = $derived((items ?? []).filter((i) => filter === 'all' || i.status === filter));
  const page = $derived(visible.slice(0, shown));
  const rejectedIds = $derived((items ?? []).filter((i) => i.status === 'rejected').map((i) => i.id));
  /** Photographs of this tab a decision can apply to (a quarantined one must be restored first). */
  const decidable = $derived(visible.filter((i) => i.status !== null && i.status !== 'quarantined'));
  const selectedIds = $derived(decidable.filter((i) => selection[i.id]).map((i) => i.id));
  const allSelected = $derived(decidable.length > 0 && selectedIds.length === decidable.length);
  const quarantinePath = $derived(root ? `${root}${root.includes('\\') ? '\\' : '/'}.timelapse${root.includes('\\') ? '\\' : '/'}quarantine` : '');
  const filterItems = $derived(
    (['flagged', 'rejected', 'quarantined', 'all'] as const).map((key) => ({
      key,
      label: t(`clean-filter-${key}`),
      badge: formatNumber(key === 'all' ? (items ?? []).length : counts[key])
    }))
  );

  // Every response is dropped if the user switched project while it was in flight.
  async function loadReview(): Promise<void> {
    const forRoot = root;
    loadError = '';
    try {
      const result = await getReview(forRoot);
      if (forRoot === root) items = result;
    } catch (err) {
      if (forRoot === root) loadError = errorText(err as IpcError);
    }
  }

  async function loadSettings(): Promise<void> {
    const forRoot = root;
    try {
      const th = await getCleanSettings(forRoot);
      if (forRoot !== root) return;
      form = thresholdsToForm(th);
      flagXmp = th.flag_xmp_rejected;
    } catch (err) {
      if (forRoot === root) loadError = errorText(err as IpcError);
    }
  }

  $effect(() => {
    if (!root) return;
    items = null;
    thumbs = {};
    thumbFailed = {};
    requested = new Set();
    outcome = null;
    selection = {};
    bulk = null;
    void loadSettings();
    void loadReview();
  });

  // Thumbnails for the tiles on screen, a few at a time. `requested` is deliberately not
  // reactive: it only stops a tile from being fetched twice while its request is in flight.
  let requested = new Set<number>();
  $effect(() => {
    const missing = page.filter((i) => !requested.has(i.id) && !i.failure);
    if (missing.length === 0) return;
    for (const i of missing) requested.add(i.id);
    const forRoot = root;
    void limited(
      missing.map((i) => async () => {
        try {
          const url = await getThumbnail(forRoot, i.id, 256);
          if (forRoot === root) thumbs = { ...thumbs, [i.id]: url };
        } catch {
          if (forRoot === root) thumbFailed = { ...thumbFailed, [i.id]: true };
        }
      }),
      THUMB_CONCURRENCY
    );
  });

  async function analyze(): Promise<void> {
    if (analyzing) return;
    analyzing = true;
    stopping = false;
    stoppedRemaining = null;
    actionError = '';
    progress = null;
    analyzeToken = Math.max(analyzeToken + 1, Date.now());
    const token = analyzeToken;
    try {
      const report = await analyzeImages(root, token, (p) => (progress = p));
      if (report.cancelled) stoppedRemaining = report.remaining;
      await loadReview();
    } catch (err) {
      actionError = errorText(err as IpcError);
    } finally {
      analyzing = false;
      stopping = false;
    }
  }

  async function stopAnalysis(): Promise<void> {
    if (!analyzing || stopping) return;
    stopping = true;
    try {
      await cancelAnalysis(analyzeToken);
    } catch {
      stopping = false;
    }
  }

  async function applySettings(): Promise<void> {
    const th = parseThresholds(form, flagXmp);
    formInvalid = th === null;
    if (!th) return;
    savingSettings = true;
    actionError = '';
    try {
      const applied = await setCleanSettings(root, th);
      form = thresholdsToForm(applied.thresholds);
      await loadReview();
    } catch (err) {
      actionError = errorText(err as IpcError);
    } finally {
      savingSettings = false;
    }
  }

  async function decide(ids: number[], decision: Decision): Promise<void> {
    actionError = '';
    try {
      await setDecision(root, ids, decision);
      await loadReview();
    } catch (err) {
      actionError = errorText(err as IpcError);
    }
  }

  async function decideSelected(): Promise<void> {
    const decision = bulk;
    if (!decision || deciding || selectedIds.length === 0) return;
    deciding = true;
    try {
      await decide(selectedIds, decision);
      selection = {};
    } finally {
      deciding = false;
      bulk = null;
    }
  }

  function toggle(id: number, on: boolean): void {
    if (on) selection = { ...selection, [id]: true };
    else {
      const { [id]: _off, ...rest } = selection;
      selection = rest;
    }
  }

  function selectAll(): void {
    selection = Object.fromEntries(decidable.map((i) => [i.id, true as const]));
  }

  async function move(kind: 'quarantine' | 'restore', ids: number[]): Promise<void> {
    if (moving || ids.length === 0) return;
    moving = true;
    actionError = '';
    try {
      const out = kind === 'quarantine' ? await quarantineImages(root, ids) : await restoreImages(root, ids);
      outcome = { kind, ok: out.filter((o) => !o.failure).length, failures: out.filter((o) => o.failure) };
      await loadReview();
    } catch (err) {
      actionError = errorText(err as IpcError);
    } finally {
      moving = false;
      confirmOpen = false;
    }
  }

  async function openInspect(item: ReviewItem): Promise<void> {
    inspect = item;
    inspectUrl = '';
    try {
      inspectUrl = await getThumbnail(root, item.id, 1024);
    } catch {
      inspectUrl = '';
    }
  }

  function pathOf(id: number): string {
    return items?.find((i) => i.id === id)?.relative_path ?? String(id);
  }

  function changeFilter(key: string): void {
    filter = key as Filter;
    shown = PAGE;
    selection = {};
  }

  const statusOf = (s: ImageStatus | null) => (s ? t(STATUS_KEYS[s]) : t('clean-status-unanalyzed'));
  const maxBin = (h: number[]) => Math.max(1, ...h);
</script>

<div class="app-page">
  <PageHeader title={t('clean-title')} />

  {#if !root}
    <Card>
      <EmptyState title={t('clean-no-project-title')} description={t('clean-no-project-description')}>
        {#snippet action()}
          <Button onclick={ongohome}>{t('clean-go-import')}</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else}
    <p class="app-clean__intro">{t('clean-intro')}</p>

    <Card>
      <SectionLabel text={t('clean-thresholds-title')} />
      <div class="app-clean__form">
        <Input label={t('clean-threshold-min-mean')} type="number" bind:value={form.minMean} />
        <Input label={t('clean-threshold-dark-level')} type="number" bind:value={form.darkLevel} />
        <Input label={t('clean-threshold-max-dark')} type="number" bind:value={form.maxDarkPercent} />
      </div>
      <Toggle label={t('clean-threshold-xmp')} bind:checked={flagXmp} />
      {#if formInvalid}<p role="alert" class="app-clean__error">{t('clean-thresholds-invalid')}</p>{/if}
      <p class="app-clean__note">{t('clean-thresholds-note')}</p>
      <Button variant="secondary" onclick={applySettings} disabled={savingSettings || analyzing}>
        {savingSettings ? t('clean-applying') : t('clean-apply')}
      </Button>
    </Card>

    <Card>
      <SectionLabel text={t('clean-analysis-title')} />
      {#if items}
        <StatGrid>
          <Stat label={t('clean-status-eligible')} value={formatNumber(counts.eligible)} tone="positive" />
          <Stat label={t('clean-status-flagged')} value={formatNumber(counts.flagged)} tone={counts.flagged > 0 ? 'warning' : 'neutral'} />
          <Stat label={t('clean-status-rejected')} value={formatNumber(counts.rejected)} />
          <Stat label={t('clean-status-quarantined')} value={formatNumber(counts.quarantined)} />
          <Stat label={t('clean-status-unanalyzed')} value={formatNumber(counts.unanalyzed)} />
        </StatGrid>
      {:else if !loadError}
        <Skeleton lines={2} label={t('clean-loading')} />
      {/if}
      <div class="app-clean__actions">
        <Button onclick={analyze} disabled={analyzing || moving}>{analyzing ? t('clean-analyzing') : t('clean-analyze')}</Button>
        {#if analyzing}
          <Button variant="secondary" onclick={stopAnalysis} disabled={stopping}>{stopping ? t('clean-stopping') : t('clean-stop')}</Button>
        {/if}
        {#if items && counts.flagged > 0}
          <Button variant="secondary" onclick={() => openContactSheet({ kind: 'flagged' })}>{t('clean-sheet-flagged')}</Button>
        {/if}
        {#if items && counts.quarantined > 0}
          <Button variant="secondary" onclick={() => openContactSheet({ kind: 'quarantined' })}>{t('clean-sheet-quarantined')}</Button>
        {/if}
        {#if analyzing && progress}
          <progress max={Math.max(progress.total, 1)} value={progress.done} aria-label={t('clean-analyzing')}></progress>
          <span aria-live="polite">{t('clean-progress', { done: formatNumber(progress.done), total: formatNumber(progress.total) })}</span>
        {/if}
      </div>
    </Card>

    {#if actionError}
      <Banner tone="error" text={actionError} />
    {/if}

    {#if stoppedRemaining !== null && !analyzing}
      <Banner
        tone="neutral"
        text={t('clean-stopped', { count: stoppedRemaining })}
        ondismiss={() => (stoppedRemaining = null)}
        dismissLabel={t('clean-dismiss')}
      />
    {/if}

    {#if outcome}
      <Banner
        tone={outcome.failures.length > 0 ? 'warning' : 'success'}
        text={t(outcome.kind === 'quarantine' ? 'clean-quarantined-result' : 'clean-restored-result', {
          ok: outcome.ok,
          failed: outcome.failures.length
        })}
        ondismiss={() => (outcome = null)}
        dismissLabel={t('clean-dismiss')}
      >
        {#if outcome.failures.length > 0}
          <ul class="app-clean__failures">
            {#each outcome.failures as f (f.image_id)}
              <li>{pathOf(f.image_id)}: {fileFailureText(f.failure ?? '')}</li>
            {/each}
          </ul>
        {/if}
      </Banner>
    {/if}

    <Card>
      <Tabs items={filterItems} active={filter} label={t('clean-filter-label')} onchange={changeFilter} />

      {#if loadError}
        <div role="alert" class="app-clean__error">
          <p>{loadError}</p>
          <Button onclick={loadReview}>{t('clean-retry')}</Button>
        </div>
      {:else if items === null}
        <Skeleton lines={6} label={t('clean-loading')} />
      {:else if items.length === 0}
        <EmptyState title={t('clean-empty-title')} description={t('clean-empty-description')} compact />
      {:else if counts.unanalyzed === items.length}
        <EmptyState title={t('clean-unanalyzed-title')} description={t('clean-unanalyzed-description')} compact>
          {#snippet action()}
            <Button onclick={analyze} disabled={analyzing}>{t('clean-analyze')}</Button>
          {/snippet}
        </EmptyState>
      {:else if visible.length === 0}
        <EmptyState title={t(`clean-filter-empty-${filter}`)} description={t('clean-filter-empty-description')} compact>
          {#snippet action()}
            <Button variant="secondary" onclick={() => changeFilter('all')}>{t('clean-show-all')}</Button>
          {/snippet}
        </EmptyState>
      {:else}
        {#if filter === 'rejected' && rejectedIds.length > 0}
          <div class="app-clean__actions">
            <Button variant="danger" onclick={() => (confirmOpen = true)} disabled={moving}>
              {t('clean-quarantine-action', { count: rejectedIds.length })}
            </Button>
          </div>
        {/if}
        {#if decidable.length > 0}
          <div class="app-clean__bulk" role="group" aria-label={t('clean-bulk-label')}>
            {#if allSelected}
              <Button size="sm" variant="ghost" onclick={() => (selection = {})} disabled={deciding}>{t('clean-select-none')}</Button>
            {:else}
              <Button size="sm" variant="ghost" onclick={selectAll} disabled={deciding}>{t('clean-select-all', { count: decidable.length })}</Button>
            {/if}
            {#if selectedIds.length > 0}
              <span class="app-clean__note" aria-live="polite">{t('clean-selected', { count: selectedIds.length })}</span>
              <Button size="sm" variant="secondary" onclick={() => (bulk = 'accept')} disabled={deciding || moving}>{t('clean-bulk-accept')}</Button>
              <Button size="sm" variant="secondary" onclick={() => (bulk = 'reject')} disabled={deciding || moving}>{t('clean-bulk-reject')}</Button>
              <Button size="sm" variant="ghost" onclick={() => (bulk = 'clear')} disabled={deciding || moving}>{t('clean-bulk-clear')}</Button>
              {#if selectedIds.length > page.length}
                <span class="app-clean__note">{t('clean-selected-beyond', { count: selectedIds.length - page.filter((i) => selection[i.id]).length })}</span>
              {/if}
            {/if}
          </div>
        {/if}
        <ul class="app-clean__grid">
          {#each page as item (item.id)}
            <li class="app-clean__tile" class:app-clean__tile--selected={!!selection[item.id]}>
              {#if item.status && item.status !== 'quarantined'}
                <label class="app-clean__pick">
                  <input
                    type="checkbox"
                    checked={!!selection[item.id]}
                    onchange={(e) => toggle(item.id, e.currentTarget.checked)}
                    aria-label={t('clean-select-photo', { path: item.relative_path })}
                  />
                </label>
              {/if}
              <button type="button" class="app-clean__thumb" onclick={() => openInspect(item)} aria-label={t('clean-inspect', { path: item.relative_path })}>
                {#if thumbs[item.id]}
                  <img src={thumbs[item.id]} alt="" />
                {:else if item.failure || thumbFailed[item.id]}
                  <span class="app-clean__thumb-empty">{item.failure ? analysisFailureText(item.failure) : t('clean-no-preview')}</span>
                {:else}
                  <span class="app-clean__thumb-empty" aria-hidden="true"></span>
                {/if}
              </button>
              <div class="app-clean__meta">
                <span class="app-clean__path" title={item.relative_path}>{item.relative_path}</span>
                <span class="app-clean__badges">
                  {#if item.status}<Badge variant={STATUS_VARIANT[item.status]}>{statusOf(item.status)}</Badge>{/if}
                  {#if item.user_override}<Badge>{t('clean-user-decision')}</Badge>{/if}
                </span>
                {#if item.mean_luma !== null}
                  <span class="app-clean__metrics">
                    {t('clean-metrics', { mean: formatLuma(item.mean_luma), dark: formatPercent(item.dark_fraction ?? 0) })}
                  </span>
                {/if}
                {#each item.reasons as r, i (i)}
                  <span class="app-clean__reason">{reasonText(r)}</span>
                {/each}
              </div>
              <div class="app-clean__tile-actions">
                {#if item.status === 'quarantined'}
                  <Button size="sm" variant="secondary" onclick={() => move('restore', [item.id])} disabled={moving}>{t('clean-restore')}</Button>
                {:else if item.status}
                  {#if item.status !== 'eligible' || !item.user_override}
                    <Button size="sm" variant="secondary" onclick={() => decide([item.id], 'accept')} disabled={moving}>{t('clean-accept')}</Button>
                  {/if}
                  {#if item.status !== 'rejected'}
                    <Button size="sm" variant="ghost" onclick={() => decide([item.id], 'reject')} disabled={moving}>{t('clean-reject')}</Button>
                  {/if}
                  {#if item.user_override}
                    <Button size="sm" variant="ghost" onclick={() => decide([item.id], 'clear')} disabled={moving}>{t('clean-clear')}</Button>
                  {/if}
                {/if}
              </div>
            </li>
          {/each}
        </ul>
        {#if visible.length > shown}
          <p class="app-clean__note">{t('clean-shown', { shown: formatNumber(shown), total: formatNumber(visible.length) })}</p>
          <Button variant="secondary" onclick={() => (shown += PAGE)}>{t('clean-show-more')}</Button>
        {/if}
      {/if}
    </Card>
  {/if}
</div>

<Modal open={confirmOpen} title={t('clean-confirm-title')} onclose={() => (confirmOpen = false)}>
  <p>{t('clean-confirm-body', { count: rejectedIds.length, path: quarantinePath })}</p>
  <p class="app-clean__note">{t('clean-confirm-safety')}</p>
  {#snippet footer()}
    <Button variant="secondary" onclick={() => (confirmOpen = false)} disabled={moving}>{t('clean-cancel')}</Button>
    <Button variant="danger" onclick={() => move('quarantine', rejectedIds)} disabled={moving}>
      {moving ? t('clean-moving') : t('clean-confirm-action', { count: rejectedIds.length })}
    </Button>
  {/snippet}
</Modal>

<Modal open={bulk !== null} title={bulk ? t(`clean-bulk-${bulk}-title`, { count: selectedIds.length }) : ''} onclose={() => (bulk = null)}>
  {#if bulk}
    <p>{t(`clean-bulk-${bulk}-body`, { count: selectedIds.length })}</p>
    <p class="app-clean__note">{t('clean-bulk-undo')}</p>
  {/if}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => (bulk = null)} disabled={deciding}>{t('clean-cancel')}</Button>
    <Button variant={bulk === 'reject' ? 'danger' : 'primary'} onclick={decideSelected} disabled={deciding}>
      {bulk ? t(`clean-bulk-${bulk}-action`, { count: selectedIds.length }) : ''}
    </Button>
  {/snippet}
</Modal>

<Modal open={inspect !== null} title={inspect?.relative_path ?? ''} size="lg" onclose={() => (inspect = null)}>
  {#if inspect}
    {#if inspectUrl}
      <img class="app-clean__inspect" src={inspectUrl} alt={t('clean-inspect', { path: inspect.relative_path })} />
    {:else if inspect.failure}
      <p>{analysisFailureText(inspect.failure)}</p>
    {:else}
      <Skeleton variant="rect" size="16rem" label={t('clean-loading')} />
    {/if}
    <dl class="app-clean__details">
      <div><dt>{t('clean-detail-status')}</dt><dd>{statusOf(inspect.status)}</dd></div>
      {#if inspect.mean_luma !== null}
        <div><dt>{t('clean-detail-mean')}</dt><dd>{formatLuma(inspect.mean_luma)}</dd></div>
        <div><dt>{t('clean-detail-median')}</dt><dd>{formatLuma(inspect.median_luma ?? 0)}</dd></div>
        <div><dt>{t('clean-detail-dark')}</dt><dd>{formatPercent(inspect.dark_fraction ?? 0)}</dd></div>
      {/if}
      {#if inspect.xmp_rating !== null}
        <div><dt>{t('clean-detail-xmp')}</dt><dd>{formatNumber(inspect.xmp_rating)}</dd></div>
      {/if}
      {#if inspect.current_path}
        <div><dt>{t('clean-detail-location')}</dt><dd class="app-clean__path">{inspect.current_path}</dd></div>
      {/if}
    </dl>
    {#if inspect.histogram.length > 0}
      <SectionLabel text={t('clean-detail-histogram')} />
      <div class="app-clean__histogram" aria-hidden="true">
        {#each inspect.histogram as bin, i (i)}
          <span style:height="{Math.round((bin / maxBin(inspect.histogram)) * 100)}%"></span>
        {/each}
      </div>
    {/if}
  {/if}
</Modal>

<style>
  .app-clean__intro,
  .app-clean__note {
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-xs) 0;
  }
  .app-clean__error {
    color: var(--ez-color-error);
  }
  .app-clean__form {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: var(--ez-space-sm);
    margin-bottom: var(--ez-space-sm);
  }
  .app-clean__actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-clean__actions progress {
    flex: 1 1 12rem;
  }
  .app-clean__failures {
    margin: var(--ez-space-xs) 0 0;
    padding-left: var(--ez-space-md);
  }
  .app-clean__grid {
    list-style: none;
    margin: var(--ez-space-sm) 0 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr));
    gap: var(--ez-space-md);
  }
  .app-clean__bulk {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-xs);
    margin: var(--ez-space-sm) 0;
  }
  .app-clean__tile--selected {
    outline: 2px solid var(--ez-color-accent);
    outline-offset: 2px;
  }
  .app-clean__pick {
    position: absolute;
    top: var(--ez-space-xs);
    left: var(--ez-space-xs);
    z-index: var(--ez-z-raised);
    display: flex;
    padding: var(--ez-space-2xs);
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-surface);
    cursor: pointer;
  }
  .app-clean__pick input {
    width: 1.1rem;
    height: 1.1rem;
    margin: 0;
    cursor: pointer;
  }
  .app-clean__tile {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-xs);
    padding: var(--ez-space-xs);
    border: 1px solid var(--ez-color-border);
    border-radius: var(--ez-radius-md);
    content-visibility: auto;
    contain-intrinsic-size: auto 22rem;
  }
  .app-clean__thumb {
    display: flex;
    align-items: center;
    justify-content: center;
    aspect-ratio: 3 / 2;
    padding: 0;
    border: 0;
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-border-soft);
    overflow: hidden;
    cursor: zoom-in;
  }
  .app-clean__thumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .app-clean__thumb-empty {
    font-size: var(--ez-fs-sm);
    color: var(--ez-color-text-muted);
  }
  .app-clean__meta {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
  }
  .app-clean__path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .app-clean__badges,
  .app-clean__tile-actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-2xs);
  }
  .app-clean__metrics,
  .app-clean__reason {
    font-size: var(--ez-fs-sm);
    color: var(--ez-color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .app-clean__inspect {
    display: block;
    max-width: 100%;
    max-height: 60vh;
    margin: 0 auto var(--ez-space-md);
  }
  .app-clean__details {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: var(--ez-space-2xs) var(--ez-space-md);
    margin: 0 0 var(--ez-space-md);
  }
  .app-clean__details dt {
    color: var(--ez-color-text-muted);
  }
  .app-clean__details dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }
  .app-clean__histogram {
    display: flex;
    align-items: flex-end;
    gap: 1px;
    height: 4rem;
  }
  .app-clean__histogram span {
    flex: 1;
    min-height: 1px;
    background: var(--ez-color-accent);
  }
</style>
