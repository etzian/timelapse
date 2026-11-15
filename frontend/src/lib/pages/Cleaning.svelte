<script lang="ts">
  import Inspector from '../components/Inspector.svelte';
  import NextStep from '../components/NextStep.svelte';
  import { Badge, Banner, Button, Card, EmptyState, Input, Modal, PageHeader, SectionLabel, Skeleton, Stat, StatGrid, Tabs, Toggle, toast } from '@etzianco/ui';
  import { tick } from 'svelte';

  import {
    STATUS_KEYS,
    STATUS_VARIANT,
    analysisFailureText,
    columnsFromTops,
    decisionApplies,
    fileFailureText,
    focusAfterRemoval,
    formatLuma,
    formatPercent,
    limited,
    moveIndex,
    neighbourId,
    parseThresholds,
    planDecision,
    reasonText,
    thresholdsToForm
  } from '../clean';
  import type { MoveKey } from '../clean';
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
  import { pipeline } from '../stores/pipeline.svelte';
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
  /** Id of the photograph open in the viewer. */
  let inspectId = $state<number | null>(null);
  /** Roving tabindex: the tile that holds the grid's single tab stop. */
  let activeId = $state<number | null>(null);
  let gridEl = $state<HTMLElement>();

  let confirmOpen = $state(false);
  let moving = $state(false);

  // Bulk decisions: the photographs ticked on the current tab, and the decision to confirm.
  let selection = $state<Record<number, true>>({});
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
  const visibleIds = $derived(visible.map((i) => i.id));
  const inspectIndex = $derived(inspectId === null ? -1 : visibleIds.indexOf(inspectId));
  const inspectItem = $derived(inspectId === null ? null : (items?.find((i) => i.id === inspectId) ?? null));
  const tabStopId = $derived(page.some((i) => i.id === activeId) ? activeId : (page[0]?.id ?? null));
  const rejectedIds = $derived((items ?? []).filter((i) => i.status === 'rejected').map((i) => i.id));
  /** Photographs of this tab a decision can apply to (a quarantined one must be restored first). */
  const decidable = $derived(visible.filter((i) => i.status !== null && i.status !== 'quarantined'));
  const selectedIds = $derived(decidable.filter((i) => selection[i.id]).map((i) => i.id));
  const allSelected = $derived(decidable.length > 0 && selectedIds.length === decidable.length);
  // ADR-026: the quarantine is a visible folder at the top of the source folder.
  const quarantinePath = $derived(root ? `${root}${root.includes('\\') ? '\\' : '/'}Timelapse quarantine` : '');
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
    inspectId = null;
    activeId = null;
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
    pipeline.setAnalyzing(true);
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
      pipeline.setAnalyzing(false);
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

  /**
   * Apply a decision at once and offer Undo. The undo snapshot is taken before the change and
   * restores each photograph's previous decision through `set_decision`. Returns the visible ids
   * after the change so a caller can place focus.
   */
  async function decide(ids: number[], decision: Decision): Promise<void> {
    if (deciding || !items) return;
    const plan = planDecision(items, ids, decision);
    if (plan.ids.length === 0) return;
    deciding = true;
    actionError = '';
    const before = visibleIds;
    const focused = activeId;
    try {
      await setDecision(root, plan.ids, decision);
      await loadReview();
      toast.undo(t(`clean-done-${decision}`, { count: plan.ids.length }), {
        label: t('clean-undo'),
        duration: 8000,
        onundo: () => void undo(plan.undo)
      });
      selection = Object.fromEntries(Object.keys(selection).filter((k) => !plan.ids.includes(Number(k))).map((k) => [k, true as const]));
      const next = focusAfterRemoval(before, visibleIds, focused);
      if (next !== null) activeId = next;
    } catch (err) {
      actionError = errorText(err as IpcError);
    } finally {
      deciding = false;
    }
  }

  async function undo(groups: { decision: Decision; ids: number[] }[]): Promise<void> {
    const forRoot = root;
    try {
      for (const g of groups) await setDecision(forRoot, g.ids, g.decision);
      if (forRoot === root) {
        await loadReview();
        toast.success(t('clean-undone'));
      }
    } catch (err) {
      if (forRoot === root) actionError = errorText(err as IpcError);
    }
  }

  /** Decide from the viewer, then move on to the next photograph (or close at the end of the list). */
  async function decideInspected(decision: Decision): Promise<void> {
    if (inspectId === null) return;
    const current = inspectId;
    const next = neighbourId(visibleIds, current);
    activeId = current;
    await decide([current], decision);
    if (inspectId === current) {
      inspectId = next !== null && visibleIds.includes(next) ? next : null;
      if (inspectId !== null) activeId = inspectId;
      else void focusTile();
    }
  }

  function step(delta: number): void {
    const to = visibleIds[inspectIndex + delta];
    if (to === undefined) return;
    inspectId = to;
    activeId = to;
    shown = Math.max(shown, inspectIndex + 1);
  }

  function closeInspector(): void {
    const last = inspectId;
    inspectId = null;
    if (last !== null) activeId = last;
    void focusTile();
  }

  /** Move DOM focus to the tile holding the tab stop (after the viewer or a re-render). */
  async function focusTile(): Promise<void> {
    await tick();
    requestAnimationFrame(() => gridEl?.querySelector<HTMLElement>('[role="option"][tabindex="0"]')?.focus());
  }

  function onGridKeydown(e: KeyboardEvent, item: ReviewItem): void {
    if (e.metaKey || e.ctrlKey || e.altKey || e.target !== e.currentTarget) return;
    const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;
    if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End'].includes(key)) {
      e.preventDefault();
      const tiles = Array.from(gridEl?.querySelectorAll<HTMLElement>('[role="option"]') ?? []);
      const at = page.findIndex((i) => i.id === item.id);
      const to = page[moveIndex(at, key as MoveKey, page.length, columnsFromTops(tiles.map((el) => el.offsetTop)))];
      if (to) {
        activeId = to.id;
        void focusTile();
      }
    } else if (key === 'a') {
      e.preventDefault();
      void decide([item.id], 'accept').then(focusTile);
    } else if (key === 'r') {
      e.preventDefault();
      void decide([item.id], 'reject').then(focusTile);
    } else if (key === 'u') {
      e.preventDefault();
      void decide([item.id], 'clear').then(focusTile);
    } else if (key === ' ' || key === 'Enter') {
      e.preventDefault();
      openInspect(item);
    } else if (key === 'x' && item.status && item.status !== 'quarantined') {
      e.preventDefault();
      toggle(item.id, !selection[item.id]);
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

  function openInspect(item: ReviewItem): void {
    activeId = item.id;
    inspectId = item.id;
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
              <Button size="sm" variant="secondary" onclick={() => decide(selectedIds, 'accept')} disabled={deciding || moving}>{t('clean-bulk-accept')}</Button>
              <Button size="sm" variant="secondary" onclick={() => decide(selectedIds, 'reject')} disabled={deciding || moving}>{t('clean-bulk-reject')}</Button>
              <Button size="sm" variant="ghost" onclick={() => decide(selectedIds, 'clear')} disabled={deciding || moving}>{t('clean-bulk-clear')}</Button>
              {#if selectedIds.length > page.length}
                <span class="app-clean__note">{t('clean-selected-beyond', { count: selectedIds.length - page.filter((i) => selection[i.id]).length })}</span>
              {/if}
            {/if}
          </div>
        {/if}
        <details class="app-clean__keys">
          <summary>{t('clean-keys-title')}</summary>
          <ul>
            <li><kbd>←</kbd> <kbd>↑</kbd> <kbd>→</kbd> <kbd>↓</kbd> {t('clean-key-move')}</li>
            <li><kbd>{t('clean-cap-keep')}</kbd> {t('clean-key-keep')}</li>
            <li><kbd>{t('clean-cap-reject')}</kbd> {t('clean-key-reject')}</li>
            <li><kbd>{t('clean-cap-clear')}</kbd> {t('clean-key-clear')}</li>
            <li><kbd>{t('clean-cap-open')}</kbd> {t('clean-key-open')}</li>
            <li><kbd>{t('clean-cap-select')}</kbd> {t('clean-key-select')}</li>
          </ul>
          <p class="app-clean__note">{t('clean-keys-hint')}</p>
        </details>
        <ul class="app-clean__grid" role="listbox" aria-multiselectable="true" aria-label={t('clean-grid-label')} bind:this={gridEl}>
          {#each page as item (item.id)}
            <li
              class="app-clean__tile"
              class:app-clean__tile--selected={!!selection[item.id]}
              role="option"
              aria-selected={!!selection[item.id]}
              aria-label={t('clean-photo-label', { path: item.relative_path, status: statusOf(item.status) })}
              tabindex={item.id === tabStopId ? 0 : -1}
              onkeydown={(e) => onGridKeydown(e, item)}
              onfocus={() => (activeId = item.id)}
            >
              {#if item.status && item.status !== 'quarantined'}
                <label class="app-clean__pick">
                  <input
                    type="checkbox"
                    tabindex="-1"
                    checked={!!selection[item.id]}
                    onchange={(e) => toggle(item.id, e.currentTarget.checked)}
                    aria-label={t('clean-select-photo', { path: item.relative_path })}
                  />
                </label>
              {/if}
              <button type="button" tabindex="-1" class="app-clean__thumb" onclick={() => openInspect(item)} aria-label={t('clean-inspect', { path: item.relative_path })}>
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
                  <button type="button" tabindex="-1" class="ez-btn ez-btn--secondary ez-btn--sm" onclick={() => move('restore', [item.id])} disabled={moving}>{t('clean-restore')}</button>
                {:else if item.status}
                  {#if decisionApplies(item, 'accept')}
                    <button type="button" tabindex="-1" class="ez-btn ez-btn--secondary ez-btn--sm" onclick={() => decide([item.id], 'accept')} disabled={moving || deciding}>{t('clean-accept')}</button>
                  {/if}
                  {#if decisionApplies(item, 'reject')}
                    <button type="button" tabindex="-1" class="ez-btn ez-btn--ghost ez-btn--sm" onclick={() => decide([item.id], 'reject')} disabled={moving || deciding}>{t('clean-reject')}</button>
                  {/if}
                  {#if decisionApplies(item, 'clear')}
                    <button type="button" tabindex="-1" class="ez-btn ez-btn--ghost ez-btn--sm" onclick={() => decide([item.id], 'clear')} disabled={moving || deciding}>{t('clean-clear')}</button>
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
  <NextStep step="cleaning" />
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

<Inspector
  {root}
  item={inspectItem}
  index={inspectIndex}
  total={visibleIds.length}
  busy={deciding}
  onprevious={() => step(-1)}
  onnext={() => step(1)}
  ondecide={decideInspected}
  onclose={closeInspector}
/>

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
  .app-clean__keys {
    margin: var(--ez-space-sm) 0;
    font-size: var(--ez-fs-sm);
    color: var(--ez-color-text-muted);
  }
  .app-clean__keys ul {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-xs) var(--ez-space-md);
    list-style: none;
    margin: var(--ez-space-xs) 0;
    padding: 0;
  }
  .app-clean__keys kbd {
    display: inline-block;
    min-width: 1.5rem;
    padding: 0 var(--ez-space-2xs);
    border: 1px solid var(--ez-color-border);
    border-radius: var(--ez-radius-sm);
    font: inherit;
    font-weight: 600;
    text-align: center;
    color: var(--ez-color-text);
  }
  .app-clean__tile:focus-visible {
    outline: var(--ez-focus-ring);
    outline-offset: var(--ez-focus-ring-offset);
  }
</style>
