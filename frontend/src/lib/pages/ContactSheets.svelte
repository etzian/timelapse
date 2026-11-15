<script lang="ts">
  import { untrack } from 'svelte';
  import { Badge, Banner, Button, Card, EmptyState, Input, PageHeader, SectionLabel, Select, Skeleton, Stat, StatGrid, Toggle } from '@etzianco/ui';

  import {
    DEFAULT_LAYOUT,
    GRID_OPTIONS,
    SOURCE_KINDS,
    boundMs,
    clampLayout,
    periodSource,
    sheetFailureText,
    sheetLabels,
    sheetStatusVariant,
    sourceText
  } from '../contactSheet';
  import type { SourceKind } from '../contactSheet';
  import { formatBytes } from '../export';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { getThumbnail, listContactSheets, listExports, listSamplingRuns, previewContactSheet } from '../ipc/app';
  import type { ExportRecord, Orientation, PageSize, RunSummary, SheetLayout, SheetPreview, SheetRecord, SheetSource } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { project } from '../stores/project.svelte';
  import { runContactSheet, sheetJob, stopContactSheet } from '../stores/contactSheet.svelte';

  let { ongohome }: { ongohome: () => void } = $props();

  const root = $derived(project.root);

  let runs = $state<RunSummary[] | null>(null);
  let exports = $state<ExportRecord[] | null>(null);
  let loadError = $state('');

  let kind = $state<SourceKind>('all');
  let from = $state('');
  let to = $state('');
  let runId = $state('');
  let exportId = $state('');
  let pageSize = $state<PageSize>(DEFAULT_LAYOUT.page_size);
  let orientation = $state<Orientation>(DEFAULT_LAYOUT.orientation);
  let columns = $state(String(DEFAULT_LAYOUT.columns));
  let rows = $state(String(DEFAULT_LAYOUT.rows));
  let showFilename = $state(DEFAULT_LAYOUT.show_filename);
  let showTimestamp = $state(DEFAULT_LAYOUT.show_timestamp);

  let preview = $state<SheetPreview | null>(null);
  let previewError = $state('');
  let previewing = $state(false);
  let thumbs = $state<Record<number, string>>({});

  // The running sheet lives in a store, so it survives leaving this page.
  const job = $derived(sheetJob.state);
  const mine = $derived(job.root === root);
  /** A sheet is being made, for this project or another one (one at a time). */
  const busy = $derived(job.running);
  const running = $derived(mine && job.running);
  const otherRunning = $derived(!mine && job.running);
  const cancelling = $derived(job.cancelling);
  const progress = $derived(mine ? job.progress : null);
  const result = $derived(mine ? job.result : null);
  const error = $derived(mine && job.error ? errorText(job.error) : '');
  /** This project's sheets, kept here: the job store may be busy with another project. */
  let history = $state<SheetRecord[] | null>(null);
  /** Bumped to ask for a fresh preview (after the source changed under a generation). */
  let previewNonce = $state(0);

  const layout = $derived<SheetLayout>(
    clampLayout({
      page_size: pageSize,
      orientation,
      columns: Number(columns),
      rows: Number(rows),
      show_filename: showFilename,
      show_timestamp: showTimestamp
    })
  );

  const completedExports = $derived((exports ?? []).filter((e) => e.status === 'completed'));

  /** The chosen source, or null while it is incomplete (no period, no selection…). */
  const source = $derived.by<SheetSource | null>(() => {
    switch (kind) {
      case 'period':
        return periodSource(from, to);
      case 'sampling':
        return runId ? { kind: 'sampling', run: Number(runId) } : null;
      case 'export':
        return exportId ? { kind: 'export', export: Number(exportId) } : null;
      default:
        return { kind };
    }
  });

  const canGenerate = $derived(!!source && !!preview && preview.photos > 0 && !preview.too_many && !busy && !previewing);

  const kindOptions = $derived(SOURCE_KINDS.map((k) => ({ value: k, label: t(`sheets-source-${k}`) })));
  const runOptions = $derived((runs ?? []).map((r) => ({ value: String(r.id), label: t('export-run-option', { id: r.id, count: r.selected_count }) })));
  const exportOptions = $derived(
    completedExports.map((e) => ({ value: String(e.id), label: t('sheets-export-option', { id: e.id, run: e.sampling_run_id, count: e.input_photos }) }))
  );
  const pageSizeOptions = $derived((['a4', 'letter'] as const).map((p) => ({ value: p, label: t(`sheets-page-${p}`) })));
  const orientationOptions = $derived((['portrait', 'landscape'] as const).map((o) => ({ value: o, label: t(`sheets-orientation-${o}`) })));
  const gridOptions = $derived(GRID_OPTIONS.map((n) => ({ value: String(n), label: formatNumber(n) })));

  const UTC = { timeZone: 'UTC' } as const;
  function formatBound(b: string): string {
    const ms = boundMs(b);
    return Number.isNaN(ms) ? b : formatDate(ms, { ...UTC, dateStyle: 'medium', timeStyle: 'short' });
  }

  function apply(s: SheetSource): void {
    kind = s.kind;
    if (s.kind === 'period') {
      from = s.from;
      to = s.to;
    } else if (s.kind === 'sampling') {
      runId = String(s.run);
    } else if (s.kind === 'export') {
      exportId = String(s.export);
    }
  }

  async function load(): Promise<void> {
    const forRoot = root;
    loadError = '';
    try {
      const [r, e, h] = await Promise.all([listSamplingRuns(forRoot), listExports(forRoot), listContactSheets(forRoot)]);
      if (forRoot !== root) return;
      runs = r;
      exports = e;
      history = h;
      if (sheetJob.state.root !== forRoot && !sheetJob.state.running) {
        // Another project's finished result is not shown under this one.
        sheetJob.dismiss();
        sheetJob.state.root = forRoot;
      }
      if (!runId && r.length > 0) runId = String(r[0].id);
      const done = e.filter((x) => x.status === 'completed');
      if (!exportId && done.length > 0) exportId = String(done[0].id);
    } catch (err) {
      if (forRoot === root) loadError = errorText(err as IpcError);
    }
  }

  $effect(() => {
    if (!root) return;
    runs = null;
    exports = null;
    history = null;
    thumbs = {};
    runId = '';
    exportId = '';
    const requested = sheetJob.take();
    if (requested) apply(requested);
    void load();
  });

  // When a sheet ends (here or for another project), this project's history is re-read.
  $effect(() => {
    void job.result;
    void job.error;
    const forRoot = root;
    if (!forRoot || job.running || runs === null) return;
    listContactSheets(forRoot)
      .then((h) => {
        if (forRoot === root) history = h;
      })
      .catch(() => {});
    // The source changed under the generation: show the preview it has now.
    const reason = (job.error?.args as { reason?: string } | undefined)?.reason;
    if (untrack(() => mine) && job.error?.kind === 'contact-sheet' && reason === 'source-changed') {
      untrack(() => (previewNonce += 1));
    }
  });

  // The preview follows the source and layout; a newer request wins over an older one.
  let previewSeq = 0;
  $effect(() => {
    const s = source;
    const l = layout;
    const forRoot = root;
    void previewNonce;
    if (!forRoot || !s || runs === null) {
      preview = null;
      previewError = '';
      return;
    }
    const seq = ++previewSeq;
    previewing = true;
    previewError = '';
    const timer = setTimeout(async () => {
      try {
        const p = await previewContactSheet(forRoot, s, l);
        if (seq !== previewSeq) return;
        preview = p;
        void loadThumbs(forRoot, p, seq);
      } catch (err) {
        if (seq === previewSeq) {
          preview = null;
          previewError = errorText(err as IpcError);
        }
      } finally {
        if (seq === previewSeq) previewing = false;
      }
    }, 250);
    return () => clearTimeout(timer);
  });

  async function loadThumbs(forRoot: string, p: SheetPreview, seq: number): Promise<void> {
    // A few at a time, so a large first page does not flood the decoder.
    const ids = p.first_page.map((c) => c.image_id).filter((id) => !(id in thumbs));
    for (let i = 0; i < ids.length; i += 6) {
      if (seq !== previewSeq || forRoot !== root) return;
      const batch = ids.slice(i, i + 6);
      const urls = await Promise.all(batch.map((id) => getThumbnail(forRoot, id, 256).catch(() => '')));
      if (seq !== previewSeq || forRoot !== root) return;
      const next = { ...thumbs };
      batch.forEach((id, k) => (next[id] = urls[k]));
      thumbs = next;
    }
  }

  function generate(): void {
    if (!source || !preview || !canGenerate) return;
    void runContactSheet(root, source, layout, sheetLabels(source, preview.photos, formatBound), preview.photos);
  }

  const pct = (v: number, of: number) => `${(v / of) * 100}%`;
</script>

<div class="app-page">
  <PageHeader title={t('sheets-title')} />

  {#if !root}
    <Card>
      <EmptyState title={t('clean-no-project-title')} description={t('sheets-no-project-description')}>
        {#snippet action()}
          <Button onclick={ongohome}>{t('clean-go-import')}</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else if loadError}
    <Card>
      <div role="alert" class="app-sheets__error">
        <p>{loadError}</p>
        <Button onclick={load}>{t('clean-retry')}</Button>
      </div>
    </Card>
  {:else if runs === null || exports === null}
    <Card><Skeleton lines={5} label={t('clean-loading')} /></Card>
  {:else}
    <p class="app-sheets__note">{t('sheets-intro')}</p>

    <Card>
      <SectionLabel text={t('sheets-source-title')} />
      <div class="app-sheets__form">
        <Select label={t('sheets-source')} options={kindOptions} bind:value={kind} disabled={running} />
        {#if kind === 'period'}
          <Input type="datetime-local" label={t('sheets-period-from')} bind:value={from} disabled={running} />
          <Input type="datetime-local" label={t('sheets-period-to')} bind:value={to} disabled={running} />
        {:else if kind === 'sampling'}
          {#if runOptions.length > 0}
            <Select label={t('sheets-sampling-run')} options={runOptions} bind:value={runId} disabled={running} />
          {:else}
            <p class="app-sheets__note">{t('sheets-no-runs')}</p>
          {/if}
        {:else if kind === 'export'}
          {#if exportOptions.length > 0}
            <Select label={t('sheets-export')} options={exportOptions} bind:value={exportId} disabled={running} />
          {:else}
            <p class="app-sheets__note">{t('sheets-no-exports')}</p>
          {/if}
        {/if}
      </div>
      {#if kind === 'period'}
        <p class="app-sheets__note">{source ? t('sheets-period-help') : t('sheets-period-invalid')}</p>
      {/if}

      <SectionLabel text={t('sheets-layout-title')} />
      <div class="app-sheets__form">
        <Select label={t('sheets-page-size')} options={pageSizeOptions} bind:value={pageSize} disabled={running} />
        <Select label={t('sheets-orientation')} options={orientationOptions} bind:value={orientation} disabled={running} />
        <Select label={t('sheets-columns')} options={gridOptions} bind:value={columns} disabled={running} />
        <Select label={t('sheets-rows')} options={gridOptions} bind:value={rows} disabled={running} />
      </div>
      <div class="app-sheets__toggles">
        <Toggle label={t('sheets-show-filename')} bind:checked={showFilename} disabled={running} />
        <Toggle label={t('sheets-show-timestamp')} bind:checked={showTimestamp} disabled={running} />
      </div>
      <p class="app-sheets__note">{t('sheets-times-utc')}</p>
    </Card>

    <Card>
      <SectionLabel text={t('sheets-preview-title')} />
      {#if previewError}
        <Banner tone="error" text={previewError} />
      {:else if !source}
        <EmptyState title={t('sheets-preview-none-title')} description={t('sheets-preview-none-description')} compact />
      {:else if preview === null}
        <Skeleton lines={4} label={t('clean-loading')} />
      {:else}
        <StatGrid>
          <Stat label={t('sheets-stat-photos')} value={formatNumber(preview.photos)} />
          <Stat label={t('sheets-stat-pages')} value={formatNumber(preview.pages)} />
          <Stat label={t('sheets-stat-per-page')} value={formatNumber(preview.geometry.columns * preview.geometry.rows)} />
        </StatGrid>
        {#if preview.too_many}
          <Banner tone="warning" text={t('error-contact-sheet', { reason: 'too-many' })} />
        {:else if preview.photos === 0}
          <EmptyState title={t('sheets-empty-title')} description={t('sheets-empty-description')} compact />
        {:else}
          {@const g = preview.geometry}
          <div
            class="app-sheets__page"
            class:app-sheets__page--refreshing={previewing}
            style:aspect-ratio="{g.page_w} / {g.page_h}"
            role="img"
            aria-label={t('sheets-preview-label', { count: preview.first_page.length })}
          >
            {#each preview.first_page as cell (cell.image_id)}
              <div
                class="app-sheets__cell"
                style:left={pct(cell.image_box.x, g.page_w)}
                style:top={pct(g.page_h - cell.image_box.y - cell.image_box.h, g.page_h)}
                style:width={pct(cell.image_box.w, g.page_w)}
                style:height={pct(cell.image_box.h, g.page_h)}
                title={[cell.filename, cell.timestamp].filter(Boolean).join(' · ')}
              >
                {#if thumbs[cell.image_id]}
                  <img src={thumbs[cell.image_id]} alt="" />
                {/if}
              </div>
            {/each}
          </div>
          <p class="app-sheets__note">{t('sheets-preview-help')}</p>
        {/if}
      {/if}

      <div class="app-sheets__actions">
        <Button onclick={generate} disabled={!canGenerate}>{running ? t('sheets-generating') : t('sheets-generate')}</Button>
        {#if running}
          <Button variant="secondary" onclick={() => void stopContactSheet()} disabled={cancelling}>
            {cancelling ? t('export-cancelling') : t('export-cancel')}
          </Button>
        {/if}
      </div>
      {#if otherRunning}
        <p class="app-sheets__note" aria-live="polite">{t('sheets-other-running')}</p>
      {/if}
      {#if running && progress}
        <div class="app-sheets__progress">
          <progress max={Math.max(progress.total, 1)} value={progress.done} aria-label={t('sheets-generating')}></progress>
          <span aria-live="polite">{t('clean-progress', { done: formatNumber(progress.done), total: formatNumber(progress.total) })}</span>
        </div>
      {/if}
    </Card>

    {#if error}<Banner tone="error" text={error} />{/if}

    {#if result}
      {#if result.status === 'completed'}
        <Banner
          tone="success"
          title={t('sheets-done-title')}
          text={t('sheets-done', { pages: result.pages, count: result.photos, photos: formatNumber(result.photos), size: formatBytes(result.file_size ?? 0) })}
          ondismiss={() => sheetJob.dismiss()}
          dismissLabel={t('clean-dismiss')}
        >
          {#if result.unreadable > 0}<p>{t('sheets-unreadable', { count: result.unreadable })}</p>{/if}
          <p class="app-sheets__path">{result.output_path}</p>
        </Banner>
      {:else if result.status === 'cancelled'}
        <Banner tone="neutral" text={t('sheets-cancelled')} ondismiss={() => sheetJob.dismiss()} dismissLabel={t('clean-dismiss')} />
      {:else}
        <Banner tone="error" text={sheetFailureText(result.failure)} ondismiss={() => sheetJob.dismiss()} dismissLabel={t('clean-dismiss')} />
      {/if}
    {/if}

    <Card>
      <SectionLabel text={t('sheets-history-title')} />
      {#if !history || history.length === 0}
        <EmptyState title={t('sheets-history-empty-title')} description={t('sheets-history-empty-description')} compact />
      {:else}
        <ul class="app-sheets__history">
          {#each history as h (h.id)}
            <li>
              <span>
                <Badge variant={sheetStatusVariant(h.status)}>{t(`sheets-status-${h.status}`)}</Badge>
                {sourceText(h.source, formatBound)}
              </span>
              <span class="app-sheets__note">
                {formatDate(h.created_at * 1000, { dateStyle: 'medium', timeStyle: 'short' })}
                · {t('sheets-history-line', { pages: h.pages, count: h.photos, photos: formatNumber(h.photos) })}
                {#if h.file_size}· {formatBytes(h.file_size)}{/if}
                {#if h.status === 'failed' || h.status === 'interrupted'}· {sheetFailureText(h.failure ?? h.status)}{/if}
              </span>
              {#if h.status === 'completed'}<span class="app-sheets__path">{h.output_path}</span>{/if}
            </li>
          {/each}
        </ul>
      {/if}
    </Card>
  {/if}
</div>

<style>
  .app-sheets__note {
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-xs) 0;
  }
  .app-sheets__error {
    color: var(--ez-color-error);
  }
  .app-sheets__form {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-sheets__toggles {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-md);
    margin: var(--ez-space-sm) 0;
  }
  .app-sheets__page {
    position: relative;
    width: min(100%, 28rem);
    margin: var(--ez-space-md) auto;
    background: var(--ez-color-surface);
    border: 1px solid var(--ez-color-border);
    box-shadow: var(--ez-shadow-sm);
  }
  .app-sheets__page--refreshing {
    opacity: 0.6;
  }
  .app-sheets__cell {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--ez-color-border-soft);
    overflow: hidden;
  }
  .app-sheets__cell img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }
  .app-sheets__actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-md);
  }
  .app-sheets__progress {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-sm);
  }
  .app-sheets__progress progress {
    flex: 1 1 12rem;
  }
  .app-sheets__path {
    font-size: var(--ez-fs-sm);
    word-break: break-all;
    user-select: all;
  }
  .app-sheets__history {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .app-sheets__history li {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    padding: var(--ez-space-sm) 0;
    border-bottom: 1px solid var(--ez-color-border);
  }
</style>
