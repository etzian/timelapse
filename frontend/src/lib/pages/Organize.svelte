<script lang="ts">
  import { Badge, Banner, Button, Card, EmptyState, Modal, PageHeader, SectionLabel, Select, Skeleton, Stat, StatGrid, Toggle } from '@etzianco/ui';

  import { NAMINGS, OPERATIONS, canUndo, consolidationFailureText, itemNote, joinTarget, sameConfig, statusVariant } from '../consolidate';
  import { formatBytes } from '../export';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { listConsolidations, listSamplingRuns, pickFolder, previewConsolidation, verifyConsolidation, verifyOriginals } from '../ipc/app';
  import type {
    ConsolidationConfig,
    ConsolidationPreview,
    ConsolidationProgress,
    ConsolidationRecord,
    Naming,
    Operation,
    OriginalsCheck,
    RunCheck,
    RunSummary
  } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { consolidationJob, runConsolidation, runUndo, stopConsolidation } from '../stores/consolidate.svelte';
  import { project } from '../stores/project.svelte';

  let { ongohome }: { ongohome: () => void } = $props();

  const root = $derived(project.root);

  let runs = $state<RunSummary[] | null>(null);
  let loadError = $state('');

  let destination = $state('');
  let operation = $state<Operation>('copy');
  let naming = $state<Naming>('preserve');
  /** '' = every photograph except rejected and quarantined ones; otherwise a sampling run id. */
  let source = $state('');

  let preview = $state<ConsolidationPreview | null>(null);
  let previewing = $state(false);
  let previewError = $state('');

  let confirmOpen = $state(false);
  let moveUnderstood = $state(false);
  let undoTarget = $state<ConsolidationRecord | null>(null);

  let checks = $state<Record<number, RunCheck>>({});
  let checking = $state<number | null>(null);
  let checkError = $state('');

  let originals = $state<OriginalsCheck | null>(null);
  let originalsProgress = $state<ConsolidationProgress | null>(null);
  let originalsBusy = $state(false);
  let originalsError = $state('');

  // The running consolidation lives in a store, so it survives leaving this page.
  const job = $derived(consolidationJob.state);
  const mine = $derived(job.root === root);
  const busy = $derived(job.busy);
  const progress = $derived(mine ? job.progress : null);
  const result = $derived(mine ? job.result : null);
  const jobError = $derived(mine && job.error ? errorText(job.error) : '');
  const history = $derived(mine ? job.history : null);

  const config = $derived<ConsolidationConfig>({ destination, operation, naming, sampling_run: source ? Number(source) : null });
  /** The dry run shown is only usable for the settings it was made with. */
  const current = $derived(preview && sameConfig(preview.config, config) ? preview : null);
  const canRun = $derived(!!current && current.counts.planned > 0 && current.enough_space && !busy);

  const operationOptions = $derived(OPERATIONS.map((o) => ({ value: o, label: t(`organize-operation-${o}`) })));
  const namingOptions = $derived(NAMINGS.map((n) => ({ value: n, label: t(`organize-naming-${n}`) })));
  const sourceOptions = $derived([
    { value: '', label: t('organize-source-all') },
    ...(runs ?? []).map((r) => ({ value: String(r.id), label: t('export-run-option', { id: r.id, count: r.selected_count }) }))
  ]);

  async function load(): Promise<void> {
    const forRoot = root;
    loadError = '';
    try {
      const [r, h] = await Promise.all([listSamplingRuns(forRoot), listConsolidations(forRoot)]);
      if (forRoot !== root) return;
      runs = r;
      if (consolidationJob.state.root === forRoot || !consolidationJob.state.busy) {
        // Another project's finished result is not shown under this one.
        if (consolidationJob.state.root !== forRoot) consolidationJob.dismiss();
        consolidationJob.state.root = forRoot;
        consolidationJob.history = h;
      }
    } catch (err) {
      if (forRoot === root) loadError = errorText(err as IpcError);
    }
  }

  $effect(() => {
    if (!root) return;
    runs = null;
    preview = null;
    originals = null;
    checks = {};
    source = '';
    void load();
  });

  async function choose(): Promise<void> {
    const picked = await pickFolder();
    if (picked) destination = picked;
  }

  async function dryRun(): Promise<void> {
    if (!destination || previewing) return;
    const forRoot = root;
    const cfg = { ...config };
    previewing = true;
    previewError = '';
    try {
      const p = await previewConsolidation(forRoot, cfg);
      if (forRoot === root) preview = p;
    } catch (err) {
      if (forRoot === root) {
        preview = null;
        previewError = errorText(err as IpcError);
      }
    } finally {
      previewing = false;
    }
  }

  function openConfirm(): void {
    moveUnderstood = false;
    confirmOpen = true;
  }

  async function confirmRun(): Promise<void> {
    if (!current || !canRun) return;
    if (current.config.operation === 'move' && !moveUnderstood) return;
    confirmOpen = false;
    const p = current;
    await runConsolidation(root, p.config, p.fingerprint);
    // The folder now holds what was placed: the old dry run no longer describes it.
    preview = null;
  }

  async function confirmUndo(): Promise<void> {
    const target = undoTarget;
    if (!target) return;
    undoTarget = null;
    await runUndo(root, target.id);
    const { [target.id]: _, ...rest } = checks;
    checks = rest;
  }

  async function check(run: number): Promise<void> {
    if (checking !== null) return;
    const forRoot = root;
    checking = run;
    checkError = '';
    try {
      const c = await verifyConsolidation(forRoot, run);
      if (forRoot === root) checks = { ...checks, [run]: c };
    } catch (err) {
      if (forRoot === root) checkError = errorText(err as IpcError);
    } finally {
      checking = null;
    }
  }

  async function checkOriginals(): Promise<void> {
    if (originalsBusy) return;
    const forRoot = root;
    originalsBusy = true;
    originalsError = '';
    originalsProgress = null;
    try {
      const c = await verifyOriginals(forRoot, (p) => {
        if (forRoot === root) originalsProgress = p;
      });
      if (forRoot === root) originals = c;
    } catch (err) {
      if (forRoot === root) originalsError = errorText(err as IpcError);
    } finally {
      originalsBusy = false;
    }
  }

  const checkText = (c: RunCheck) =>
    c.ok === c.checked
      ? t('organize-check-ok', { count: c.checked })
      : t('organize-check-problems', { ok: formatNumber(c.ok), modified: c.modified, missing: c.missing, moved: c.moved_since });
</script>

<div class="app-page">
  <PageHeader title={t('organize-title')} />

  {#if !root}
    <Card>
      <EmptyState title={t('clean-no-project-title')} description={t('organize-no-project-description')}>
        {#snippet action()}
          <Button onclick={ongohome}>{t('clean-go-import')}</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else if loadError}
    <Card>
      <div role="alert" class="app-organize__error">
        <p>{loadError}</p>
        <Button onclick={load}>{t('clean-retry')}</Button>
      </div>
    </Card>
  {:else if runs === null}
    <Card><Skeleton lines={5} label={t('clean-loading')} /></Card>
  {:else}
    <p class="app-organize__note">{t('organize-intro')}</p>

    <Card>
      <SectionLabel text={t('organize-settings-title')} />
      <div class="app-organize__destination">
        <div class="app-organize__destination-text">
          <span class="app-organize__label">{t('organize-destination')}</span>
          {#if destination}
            <span class="app-organize__path">{destination}</span>
          {:else}
            <span class="app-organize__note">{t('organize-destination-none')}</span>
          {/if}
        </div>
        <Button variant="secondary" onclick={choose} disabled={busy}>{t('organize-destination-choose')}</Button>
      </div>
      <p class="app-organize__note">{t('organize-destination-help')}</p>

      <div class="app-organize__form">
        <Select label={t('organize-operation')} options={operationOptions} bind:value={operation} disabled={busy} />
        <Select label={t('organize-source')} options={sourceOptions} bind:value={source} disabled={busy} />
        <Select label={t('organize-naming')} options={namingOptions} bind:value={naming} disabled={busy} />
      </div>
      <p class="app-organize__note">{t(`organize-operation-${operation}-help`)}</p>
      <p class="app-organize__note">{t(`organize-naming-${naming}-help`)}</p>

      <div class="app-organize__actions">
        <Button onclick={dryRun} disabled={!destination || previewing || busy}>{previewing ? t('organize-previewing') : t('organize-preview')}</Button>
      </div>
      {#if previewError}<p role="alert" class="app-organize__error">{previewError}</p>{/if}
    </Card>

    {#if current}
      <Card>
        <SectionLabel text={t('organize-preview-title')} />
        <StatGrid>
          <Stat label={t('organize-stat-planned')} value={formatNumber(current.counts.planned)} />
          <Stat label={t('organize-stat-duplicates')} value={formatNumber(current.counts.duplicates)} />
          <Stat label={t('organize-stat-present')} value={formatNumber(current.counts.already_present)} />
          <Stat label={t('organize-stat-renamed')} value={formatNumber(current.renamed)} />
          <Stat label={t('organize-stat-bytes')} value={formatBytes(current.counts.bytes)} />
          <Stat label={t('organize-stat-free')} value={current.free_bytes === null ? t('organize-free-unknown') : formatBytes(current.free_bytes)} />
        </StatGrid>
        <p class="app-organize__note">
          {t('organize-excluded', {
            rejected: current.excluded.rejected,
            quarantined: current.excluded.quarantined,
            missing: current.excluded.missing
          })}
        </p>
        {#if !current.enough_space}
          <Banner tone="error" text={consolidationFailureText('insufficient-space')} />
        {/if}
        {#if current.counts.unsafe_names > 0}
          <Banner tone="warning" text={t('organize-unsafe-names', { count: current.counts.unsafe_names })} />
        {/if}
        {#if current.counts.planned === 0}
          <Banner tone="info" text={consolidationFailureText('nothing-to-do')} />
        {/if}

        {#if current.items.length > 0}
          <details class="app-organize__details" open>
            <summary>{t('organize-items-title', { shown: current.items.length, total: current.input_count })}</summary>
            <ul class="app-organize__items">
              {#each current.items as item (item.image_id)}
                <li>
                  <span class="app-organize__path">{item.relative_path}</span>
                  {#if item.target && item.action.kind !== 'duplicate'}
                    <span class="app-organize__path">→ {joinTarget(current.config.destination, item.target)}</span>
                  {/if}
                  {#if itemNote(item)}<span class="app-organize__note">{itemNote(item)}</span>{/if}
                </li>
              {/each}
            </ul>
          </details>
        {/if}
        {#if current.notable.length > 0}
          <details class="app-organize__details">
            <summary>{t('organize-notable-title', { count: current.notable.length })}</summary>
            <ul class="app-organize__items">
              {#each current.notable as item (item.image_id)}
                <li>
                  <span class="app-organize__path">{item.relative_path}</span>
                  <span class="app-organize__note">{itemNote(item)}</span>
                </li>
              {/each}
            </ul>
          </details>
        {/if}

        <div class="app-organize__actions">
          <Button variant={current.config.operation === 'move' ? 'danger' : 'primary'} onclick={openConfirm} disabled={!canRun}>
            {t(`organize-run-${current.config.operation}`, { count: current.counts.planned })}
          </Button>
        </div>
      </Card>
    {:else if preview}
      <p class="app-organize__note">{t('organize-preview-stale')}</p>
    {/if}

    {#if busy && mine}
      <Card>
        <div class="app-organize__progress">
          {#if progress}
            <progress max={Math.max(progress.total, 1)} value={progress.done} aria-label={t(`organize-busy-${job.kind ?? 'run'}`)}></progress>
            <span aria-live="polite">{t('clean-progress', { done: formatNumber(progress.done), total: formatNumber(progress.total) })}</span>
          {:else}
            <span aria-live="polite">{t(`organize-busy-${job.kind ?? 'run'}`)}</span>
          {/if}
          <Button variant="secondary" onclick={() => void stopConsolidation()} disabled={job.cancelling}>
            {job.cancelling ? t('export-cancelling') : t('export-cancel')}
          </Button>
        </div>
      </Card>
    {/if}

    {#if jobError}<Banner tone="error" text={jobError} ondismiss={() => consolidationJob.dismiss()} dismissLabel={t('clean-dismiss')} />{/if}

    {#if result}
      <Banner
        tone={job.resultKind === 'undo'
          ? result.undo_status === 'undone'
            ? 'success'
            : 'warning'
          : result.status === 'completed'
            ? 'success'
            : result.status === 'cancelled'
              ? 'neutral'
              : 'warning'}
        text={job.resultKind === 'undo'
          ? t('organize-undo-result', { undone: result.undone, failed: result.undo_failed })
          : t(`organize-result-${result.config.operation}`, { placed: result.placed, failed: result.failed, status: result.status })}
        ondismiss={() => consolidationJob.dismiss()}
        dismissLabel={t('clean-dismiss')}
      >
        {#if job.resultKind !== 'undo'}<p class="app-organize__path">{result.config.destination}</p>{/if}
      </Banner>
    {/if}

    <Card>
      <SectionLabel text={t('organize-history-title')} />
      {#if checkError}<p role="alert" class="app-organize__error">{checkError}</p>{/if}
      {#if !history || history.length === 0}
        <EmptyState title={t('organize-history-empty-title')} description={t('organize-history-empty-description')} compact />
      {:else}
        <ul class="app-organize__history">
          {#each history as h (h.id)}
            <li>
              <div class="app-organize__history-main">
                <span>
                  <Badge variant={statusVariant(h.status)}>{t(`organize-status-${h.status}`)}</Badge>
                  {#if h.undo_status}<Badge variant="default">{t(`organize-undo-status-${h.undo_status}`)}</Badge>{/if}
                  {t(`organize-history-line-${h.config.operation}`, { count: h.placed, naming: t(`organize-naming-${h.config.naming}`) })}
                </span>
                <span class="app-organize__path">{h.config.destination}</span>
                <span class="app-organize__note">
                  {formatDate(h.created_at * 1000, { dateStyle: 'medium', timeStyle: 'short' })}
                  {#if h.failed > 0}· {t('organize-history-failed', { count: h.failed })}{/if}
                  {#if h.undone > 0}· {t('organize-history-undone', { count: h.undone })}{/if}
                </span>
                {#if checks[h.id]}<span class="app-organize__note" aria-live="polite">{checkText(checks[h.id])}</span>{/if}
              </div>
              <div class="app-organize__history-actions">
                {#if h.placed > 0 && h.undo_status !== 'undone'}
                  <Button variant="secondary" size="sm" onclick={() => void check(h.id)} disabled={checking !== null || busy}>
                    {checking === h.id ? t('organize-checking') : t('organize-check')}
                  </Button>
                {/if}
                {#if canUndo(h)}
                  <Button variant="secondary" size="sm" onclick={() => (undoTarget = h)} disabled={busy}>{t('organize-undo')}</Button>
                {/if}
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </Card>

    <Card>
      <SectionLabel text={t('organize-originals-title')} />
      <p class="app-organize__note">{t('organize-originals-help')}</p>
      <div class="app-organize__actions">
        <Button variant="secondary" onclick={checkOriginals} disabled={originalsBusy}>{originalsBusy ? t('organize-checking') : t('organize-originals-check')}</Button>
      </div>
      {#if originalsBusy && originalsProgress}
        <div class="app-organize__progress">
          <progress max={Math.max(originalsProgress.total, 1)} value={originalsProgress.done} aria-label={t('organize-checking')}></progress>
          <span aria-live="polite">{t('clean-progress', { done: formatNumber(originalsProgress.done), total: formatNumber(originalsProgress.total) })}</span>
        </div>
      {/if}
      {#if originalsError}<p role="alert" class="app-organize__error">{originalsError}</p>{/if}
      {#if originals && !originalsBusy}
        <StatGrid>
          <Stat label={t('organize-originals-ok')} value={formatNumber(originals.ok)} />
          <Stat label={t('organize-originals-changed')} value={formatNumber(originals.changed)} />
          <Stat label={t('organize-originals-missing')} value={formatNumber(originals.missing)} />
          <Stat label={t('organize-originals-unhashed')} value={formatNumber(originals.unhashed)} />
        </StatGrid>
        {#if originals.changed === 0 && originals.missing === 0}
          <Banner tone="success" text={t('organize-originals-all-ok', { count: originals.ok })} />
        {:else}
          <Banner tone="warning" text={t('organize-originals-problems', { changed: originals.changed, missing: originals.missing })} />
          <ul class="app-organize__items">
            {#each originals.problems as p (p.image_id)}
              <li>
                <span class="app-organize__path">{p.path}</span>
                <span class="app-organize__note">{t(`organize-problem-${p.state}`)}</span>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </Card>
  {/if}
</div>

<Modal open={confirmOpen} title={t(`organize-confirm-title-${current?.config.operation ?? 'copy'}`)} onclose={() => (confirmOpen = false)}>
  {#if current}
    <p>{t(`organize-confirm-body-${current.config.operation}`, { count: current.counts.planned, path: current.config.destination })}</p>
    <p class="app-organize__note">{t('organize-confirm-safety')}</p>
    {#if current.config.operation === 'move'}
      <Toggle label={t('organize-confirm-move-understood')} bind:checked={moveUnderstood} />
    {/if}
  {/if}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => (confirmOpen = false)}>{t('clean-cancel')}</Button>
    <Button
      variant={current?.config.operation === 'move' ? 'danger' : 'primary'}
      onclick={confirmRun}
      disabled={!canRun || (current?.config.operation === 'move' && !moveUnderstood)}
    >
      {t(`organize-run-${current?.config.operation ?? 'copy'}`, { count: current?.counts.planned ?? 0 })}
    </Button>
  {/snippet}
</Modal>

<Modal open={undoTarget !== null} title={t('organize-undo-title')} onclose={() => (undoTarget = null)}>
  {#if undoTarget}
    <p>{t(`organize-undo-body-${undoTarget.config.operation}`, { count: undoTarget.placed - undoTarget.undone, path: undoTarget.config.destination })}</p>
    <p class="app-organize__note">{t('organize-undo-safety')}</p>
  {/if}
  {#snippet footer()}
    <Button variant="secondary" onclick={() => (undoTarget = null)}>{t('clean-cancel')}</Button>
    <Button variant="danger" onclick={confirmUndo} disabled={busy}>{t('organize-undo')}</Button>
  {/snippet}
</Modal>

<style>
  .app-organize__note {
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-xs) 0;
  }
  .app-organize__label {
    font-size: var(--ez-fs-sm);
    font-weight: 600;
  }
  .app-organize__error {
    color: var(--ez-color-error);
  }
  .app-organize__destination {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-organize__destination-text {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
  }
  .app-organize__form {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-organize__actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-md);
  }
  .app-organize__progress {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-sm);
  }
  .app-organize__progress progress {
    flex: 1 1 12rem;
  }
  .app-organize__path {
    font-size: var(--ez-fs-sm);
    word-break: break-all;
  }
  .app-organize__details {
    margin-top: var(--ez-space-sm);
  }
  .app-organize__items,
  .app-organize__history {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 24rem;
    overflow: auto;
  }
  .app-organize__items li {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    padding: var(--ez-space-xs) 0;
    border-bottom: 1px solid var(--ez-color-border);
  }
  .app-organize__history li {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--ez-space-sm);
    padding: var(--ez-space-sm) 0;
    border-bottom: 1px solid var(--ez-color-border);
  }
  .app-organize__history-main {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
    flex: 1 1 16rem;
  }
  .app-organize__history-actions {
    display: flex;
    gap: var(--ez-space-xs);
    align-items: flex-start;
  }
</style>
