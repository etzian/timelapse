<script lang="ts">
  import ExportActions from '../components/ExportActions.svelte';
  import Flipbook from '../components/Flipbook.svelte';
  import NextStep from '../components/NextStep.svelte';
  import Progress from '../components/Progress.svelte';
  import { runLabel, runLabelWithId } from '../selectionLabel';
  import { Badge, Banner, Button, Card, EmptyState, PageHeader, SectionLabel, Select, Skeleton, Stat, StatGrid } from '@etzianco/ui';

  import { CODECS, EXPORT_PRESETS, QUALITIES, RESOLUTIONS, exportFailureText, formatBytes, matchExportPreset, presetAvailable, summarize, summaryText, supports } from '../export';
  import type { ExportPreset, ResolutionKey } from '../export';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { exportRecord, ffmpegInfo, getSamplingItems, listAdjustments, listExports, listSamplingRuns, writeExportRecord } from '../ipc/app';
  import type { Adjustments, Codec, ExportRecord, FfmpegInfo, Quality, RunSummary, SampledItem } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { FPS_OPTIONS, MULTIPLIERS } from '../sampling';
  import { openContactSheet } from '../stores/contactSheet.svelte';
  import { exportJob, posterFor, runExport, stopExport } from '../stores/export.svelte';
  import { project } from '../stores/project.svelte';
  import { formatDuration } from '../timeline';

  let { ongohome, ongosampling }: { ongohome: () => void; ongosampling: () => void } = $props();

  const root = $derived(project.root);

  let info = $state<FfmpegInfo | null>(null);
  let runs = $state<RunSummary[] | null>(null);
  let loadError = $state('');

  let runId = $state('');
  let adjustments = $state<Adjustments | null>(null);
  /** Phase 8 runs of the chosen selection to apply ('' = none). */
  let flickerRun = $state('');
  let stabRun = $state('');
  let codec = $state<Codec>('av1');
  let fps = $state('30');
  let speed = $state('1');
  let resolution = $state<ResolutionKey>('1080');
  let quality = $state<Quality>('standard');

  // The running export lives in a store, so it survives leaving this page.
  const job = $derived(exportJob.state);
  const mine = $derived(job.root === root);
  const exporting = $derived(job.exporting);
  const cancelling = $derived(job.cancelling);
  const progress = $derived(mine ? job.progress : null);
  const result = $derived(mine ? job.result : null);
  const error = $derived(mine && job.error ? errorText(job.error) : '');
  const history = $derived(mine ? job.history : null);

  const historyRun = (id: number): string => {
    const r = (runs ?? []).find((x) => x.id === id);
    return r ? runLabel(r) : `#${id}`;
  };
  const run = $derived(runs?.find((r) => String(r.id) === runId) ?? null);
  const flickerOptions = $derived([
    { value: '', label: t('export-adjust-none') },
    ...(adjustments?.flicker ?? [])
      .filter((r) => String(r.sampling_run_id) === runId)
      .map((r) => ({ value: String(r.id), label: t('export-flicker-option', { id: r.id, strength: formatNumber(r.config.strength, { style: 'percent' }) }) }))
  ]);
  const stabOptions = $derived([
    { value: '', label: t('export-adjust-none') },
    ...(adjustments?.stabilization ?? [])
      .filter((r) => String(r.sampling_run_id) === runId)
      .map((r) => ({ value: String(r.id), label: t('export-stab-option', { id: r.id, crop: formatNumber(r.crop, { style: 'percent', maximumFractionDigits: 1 }) }) }))
  ]);

  // Another selection: its own runs, none chosen.
  $effect(() => {
    void runId;
    flickerRun = '';
    stabRun = '';
  });
  const summary = $derived(run ? summarize(run.selected_count, Number(fps), Number(speed)) : null);
  const preset = $derived(matchExportPreset({ codec, resolution, quality }));
  const unavailablePresets = $derived(EXPORT_PRESETS.filter((p) => !presetAvailable(info, p)));

  function choosePreset(p: ExportPreset): void {
    if (exporting || !presetAvailable(info, p)) return;
    codec = p.codec;
    resolution = p.resolution;
    quality = p.quality;
  }
  const canExport = $derived(!!run && supports(info, codec) && !exporting);

  const runOptions = $derived(
    (runs ?? []).map((r) => ({ value: String(r.id), label: runLabelWithId(r) }))
  );
  const codecOptions = $derived(
    CODECS.map((c) => ({
      value: c,
      label: supports(info, c) ? t(`export-codec-${c}`) : t('export-codec-unavailable', { codec: t(`export-codec-${c}`) }),
      disabled: !supports(info, c)
    }))
  );
  const fpsOptions = $derived(FPS_OPTIONS.map((v) => ({ value: String(v), label: t('sampling-fps-option', { fps: v }) })));
  const speedOptions = $derived(
    MULTIPLIERS.map((v) => ({ value: String(v), label: t('sampling-multiplier-option', { multiplier: formatNumber(v) }) }))
  );
  const resolutionOptions = $derived(RESOLUTIONS.map((r) => ({ value: r.key, label: t('export-resolution-option', { width: String(r.width), height: String(r.height) }) })));
  const qualityOptions = $derived(QUALITIES.map((q) => ({ value: q, label: t(`export-quality-${q}`) })));

  async function load(): Promise<void> {
    const forRoot = root;
    loadError = '';
    try {
      const [i, r, h, a] = await Promise.all([ffmpegInfo(), listSamplingRuns(forRoot), listExports(forRoot), listAdjustments(forRoot)]);
      if (forRoot !== root) return;
      adjustments = a;
      info = i;
      runs = r;
      if (exportJob.state.root === forRoot || !exportJob.state.exporting) {
        // Another project's finished result is not shown under this one.
        if (exportJob.state.root !== forRoot) exportJob.dismiss();
        exportJob.state.root = forRoot;
        exportJob.history = h;
      }
      if (!runId && r.length > 0) runId = String(r[0].id);
      if (!supports(i, codec)) codec = CODECS.find((c) => supports(i, c)) ?? 'av1';
    } catch (err) {
      if (forRoot === root) loadError = errorText(err as IpcError);
    }
  }

  $effect(() => {
    if (!root) return;
    info = null;
    runs = null;
    runId = '';
    void load();
  });

  // The preview: the chosen selection as shot, loaded 200 photographs at a time (at most PREVIEW_MAX).
  const PREVIEW_MAX = 1000;
  const PREVIEW_PAGE = 200;
  let previewItems = $state<SampledItem[] | null>(null);
  let previewError = $state('');
  let previewTry = $state(0);
  let previewSeq = 0;
  $effect(() => {
    const forRoot = root;
    const id = run?.id;
    void previewTry;
    if (!forRoot || id === undefined) return;
    const seq = ++previewSeq;
    previewItems = null;
    previewError = '';
    (async () => {
      try {
        const first = await getSamplingItems(forRoot, id, 0, PREVIEW_PAGE);
        if (seq !== previewSeq) return;
        let items = first.items;
        previewItems = items;
        while (items.length < Math.min(first.total, PREVIEW_MAX)) {
          const page = await getSamplingItems(forRoot, id, items.length, PREVIEW_PAGE);
          if (seq !== previewSeq) return;
          if (page.items.length === 0) break;
          items = [...items, ...page.items];
          previewItems = items;
        }
      } catch (err) {
        if (seq === previewSeq) previewError = errorText(err as IpcError);
      }
    })();
  });

  function start(): void {
    if (!run || !canExport) return;
    const r = RESOLUTIONS.find((x) => x.key === resolution) ?? RESOLUTIONS[2];
    void runExport(root, run.id, {
      codec,
      fps: Number(fps),
      speed: Number(speed),
      width: r.width,
      height: r.height,
      quality,
      flicker_run: flickerRun ? Number(flickerRun) : null,
      stabilization_run: stabRun ? Number(stabRun) : null
    });
  }

  function cancel(): void {
    void stopExport();
  }

  // The derived export record (ADR-015): fetched when opened, saved on request.
  let records = $state<Record<number, { json?: string; saved?: string; error?: string; busy?: boolean }>>({});

  async function toggleRecord(id: number): Promise<void> {
    if (records[id]?.json !== undefined) {
      const { [id]: _closed, ...rest } = records;
      records = rest;
      return;
    }
    const forRoot = root;
    records = { ...records, [id]: { busy: true } };
    try {
      const json = await exportRecord(forRoot, id);
      if (forRoot === root) records = { ...records, [id]: { json } };
    } catch (err) {
      if (forRoot === root) records = { ...records, [id]: { error: errorText(err as IpcError) } };
    }
  }

  async function saveRecord(id: number): Promise<void> {
    const forRoot = root;
    records = { ...records, [id]: { ...records[id], busy: true, error: undefined } };
    try {
      const saved = await writeExportRecord(forRoot, id);
      if (forRoot === root) records = { ...records, [id]: { ...records[id], saved, busy: false } };
    } catch (err) {
      if (forRoot === root) records = { ...records, [id]: { ...records[id], error: errorText(err as IpcError), busy: false } };
    }
  }

  // The finished video's poster: its selection's middle frame.
  let poster = $state<string | null>(null);
  $effect(() => {
    const r = result;
    const forRoot = root;
    poster = null;
    if (!r || r.status !== 'completed') return;
    void posterFor(forRoot, r.sampling_run_id).then((url) => {
      if (result === r && forRoot === root) poster = url;
    });
  });

  const statusVariant = (s: ExportRecord['status']) =>
    s === 'completed' ? 'success' : s === 'running' ? 'info' : s === 'cancelled' ? 'default' : 'error';
</script>

<div class="app-page">
  <PageHeader title={t('export-title')} />

  {#if !root}
    <Card>
      <EmptyState title={t('clean-no-project-title')} description={t('export-no-project-description')}>
        {#snippet action()}
          <Button onclick={ongohome}>{t('clean-go-import')}</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else if loadError}
    <Card>
      <div role="alert" class="app-export__error">
        <p>{loadError}</p>
        <Button onclick={load}>{t('clean-retry')}</Button>
      </div>
    </Card>
  {:else if info === null || runs === null}
    <Card><Skeleton lines={5} label={t('clean-loading')} /></Card>
  {:else}
    <p class="app-export__note">{t('export-intro')}</p>

    {#if !info.available}
      <Banner tone="warning" title={t('export-no-ffmpeg-title')} text={t('export-no-ffmpeg-description')} />
    {/if}

    {#if runs.length === 0}
      <Card>
        <EmptyState title={t('export-no-runs-title')} description={t('export-no-runs-description')}>
          {#snippet action()}
            <Button onclick={ongosampling}>{t('export-go-sampling')}</Button>
          {/snippet}
        </EmptyState>
      </Card>
    {:else}
      <Card>
        <SectionLabel text={t('export-settings-title')} />
        <div class="app-export__form">
          <Select label={t('export-run')} options={runOptions} bind:value={runId} disabled={exporting} />
        </div>

        <SectionLabel text={t('export-preset-label')} />
        <div class="app-export__presets" role="group" aria-label={t('export-preset-label')}>
          {#each EXPORT_PRESETS as p (p.key)}
            <button
              type="button"
              class="ez-btn ez-btn--md ez-btn--{preset === p.key ? 'primary' : 'secondary'}"
              aria-pressed={preset === p.key}
              aria-describedby="app-export-preset-help"
              onclick={() => choosePreset(p)}
              disabled={exporting || !presetAvailable(info, p)}
            >
              {t(`export-preset-${p.key}`)}
            </button>
          {/each}
        </div>
        <p id="app-export-preset-help" class="app-export__note" aria-live="polite">
          {preset === 'custom' ? t('export-preset-custom') : t(`export-preset-${preset}-help`)}
        </p>
        {#each unavailablePresets as p (p.key)}
          <p class="app-export__note">{t('export-preset-unavailable', { preset: t(`export-preset-${p.key}`), codec: t(`export-codec-${p.codec}`) })}</p>
        {/each}

        {#if run}
          <p class="app-export__summary" aria-live="polite">{summaryText(run.selected_count, Number(fps), Number(speed))}</p>
        {/if}

        <details class="app-export__advanced">
          <summary>{t('export-advanced')}</summary>
          <div class="app-export__form">
            <Select label={t('export-codec')} options={codecOptions} bind:value={codec} disabled={exporting} />
            <Select label={t('sampling-fps')} options={fpsOptions} bind:value={fps} disabled={exporting} />
            <Select label={t('sampling-multiplier')} options={speedOptions} bind:value={speed} disabled={exporting} />
            <Select label={t('export-resolution')} options={resolutionOptions} bind:value={resolution} disabled={exporting} />
            <Select label={t('export-quality')} options={qualityOptions} bind:value={quality} disabled={exporting} />
            <Select label={t('export-flicker')} options={flickerOptions} bind:value={flickerRun} disabled={exporting} />
            <Select label={t('export-stab')} options={stabOptions} bind:value={stabRun} disabled={exporting} />
          </div>
          <p class="app-export__note">{t(`export-codec-${codec}-help`)}</p>
        </details>

        {#if run && summary}
          <StatGrid>
            <Stat label={t('export-stat-photos')} value={formatNumber(summary.photos)} />
            <Stat label={t('export-stat-frames')} value={formatNumber(summary.frames)} />
            <Stat label={t('export-stat-duration')} value={formatDuration(summary.seconds)} />
          </StatGrid>
        {/if}

        <div class="app-export__actions">
          <Button onclick={start} disabled={!canExport}>{exporting ? t('export-exporting') : t('export-start')}</Button>
          {#if exporting}
            <Button variant="secondary" onclick={cancel} disabled={cancelling}>{cancelling ? t('export-cancelling') : t('export-cancel')}</Button>
          {/if}
        </div>
        {#if exporting && progress}
          <div class="app-export__progress">
            <Progress done={progress.done} total={progress.total} label={t('export-exporting')} thumb={mine ? job.thumb : null} />
          </div>
        {/if}
      </Card>
    {/if}

    {#if runs.length > 0 && run}
      <Card>
        <SectionLabel text={t('export-preview-title')} />
        <p class="app-export__note">{t('export-preview-help')}</p>
        {#if previewError}
          <div role="alert" class="app-export__error">
            <p>{previewError}</p>
            <Button onclick={() => previewTry++}>{t('clean-retry')}</Button>
          </div>
        {:else if previewItems === null}
          <Skeleton lines={5} label={t('clean-loading')} />
        {:else}
          <Flipbook {root} frames={previewItems} fps={Number(fps)} speed={Number(speed)} total={run.selected_count} resetKey={`${root}|${run.id}`} />
        {/if}
      </Card>
    {/if}

    {#if error}<Banner tone="error" text={error} />{/if}

    {#if result}
      {#if result.status === 'completed'}
        <Card>
          <div class="app-export__done">
            {#if poster}<img class="app-export__poster" src={poster} alt="" />{/if}
            <div class="app-export__done-body">
              <h2 class="app-export__done-title">{t('export-done-title')}</h2>
              <p>
                {t('export-done', {
                  duration: formatDuration(result.duration_seconds),
                  size: formatBytes(result.file_size ?? 0),
                  codec: t(`export-codec-${result.settings.codec}`)
                })}
              </p>
              <p class="app-export__path">{result.output_path}</p>
              <ExportActions {root} exportId={result.id} size="md" />
              <div>
                <Button size="sm" variant="ghost" onclick={() => exportJob.dismiss()}>{t('clean-dismiss')}</Button>
              </div>
            </div>
          </div>
        </Card>
      {:else}
        <Banner tone={result.status === 'cancelled' ? 'neutral' : 'error'} text={exportFailureText(result.failure)}>
          {#if result.stderr_tail}
            <details>
              <summary>{t('export-technical-details')}</summary>
              <pre class="app-export__tail">{result.stderr_tail}</pre>
            </details>
          {/if}
        </Banner>
      {/if}
    {/if}

    <Card>
      <SectionLabel text={t('export-history-title')} />
      {#if !history || history.length === 0}
        <EmptyState title={t('export-history-empty-title')} description={t('export-history-empty-description')} compact />
      {:else}
        <ul class="app-export__history">
          {#each history as h (h.id)}
            <li>
              <div class="app-export__history-main">
                <span>
                  <Badge variant={statusVariant(h.status)}>{t(`export-status-${h.status}`)}</Badge>
                  {t('export-history-line', {
                    run: historyRun(h.sampling_run_id),
                    codec: t(`export-codec-${h.settings.codec}`),
                    duration: formatDuration(h.duration_seconds)
                  })}
                </span>
                <span class="app-export__note">
                  {formatDate(h.created_at * 1000, { dateStyle: 'medium', timeStyle: 'short' })}
                  {#if h.file_size}· {formatBytes(h.file_size)}{/if}
                  {#if h.failure && h.status !== 'completed'}· {exportFailureText(h.failure)}{/if}
                </span>
                {#if h.status === 'completed'}<span class="app-export__path">{h.output_path}</span>{/if}
              </div>
              <div class="app-export__history-actions">
                <Button size="sm" variant="ghost" onclick={() => toggleRecord(h.id)} disabled={records[h.id]?.busy}>
                  {records[h.id]?.json !== undefined ? t('export-record-hide') : t('export-record-show')}
                </Button>
                <Button size="sm" variant="ghost" onclick={() => saveRecord(h.id)} disabled={records[h.id]?.busy}>{t('export-record-save')}</Button>
                {#if h.status === 'completed'}<ExportActions {root} exportId={h.id} sheet={false} />{/if}
                <Button size="sm" variant="ghost" onclick={() => openContactSheet({ kind: 'export', export: h.id })}>{t('export-sheet')}</Button>
              </div>
              {#if records[h.id]?.error}<p role="alert" class="app-export__error">{records[h.id].error}</p>{/if}
              {#if records[h.id]?.saved}
                <p class="app-export__note">{t('export-record-saved')}</p>
                <span class="app-export__path">{records[h.id].saved}</span>
              {/if}
              {#if records[h.id]?.json !== undefined}
                <p class="app-export__note">{t('export-record-help')}</p>
                <pre class="app-export__record" aria-label={t('export-record-label', { id: h.id })}>{records[h.id].json}</pre>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </Card>
  {/if}
  <NextStep step="export" />
</div>

<style>
  .app-export__note {
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-xs) 0;
  }
  .app-export__error {
    color: var(--ez-color-error);
  }
  .app-export__form {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-export__presets {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-xs);
    margin: var(--ez-space-xs) 0;
  }
  .app-export__summary {
    font-size: var(--ez-fs-md);
    margin: var(--ez-space-sm) 0;
  }
  .app-export__advanced {
    margin: var(--ez-space-sm) 0;
    font-size: var(--ez-fs-sm);
    color: var(--ez-color-text-muted);
  }
  .app-export__actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-md);
  }
  .app-export__progress {
    display: flex;
    margin-top: var(--ez-space-sm);
  }
  .app-export__done {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-md);
    align-items: flex-start;
  }
  .app-export__poster {
    width: var(--app-poster-width);
    aspect-ratio: 3 / 2;
    object-fit: cover;
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-border-soft);
    flex: none;
  }
  .app-export__done-body {
    flex: 1 1 16rem;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-xs);
  }
  .app-export__path {
    font-size: var(--ez-fs-sm);
    word-break: break-all;
    user-select: all;
  }
  .app-export__tail {
    font-size: var(--ez-fs-xs);
    white-space: pre-wrap;
    max-height: 12rem;
    overflow: auto;
  }
  .app-export__history {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .app-export__history li {
    padding: var(--ez-space-sm) 0;
    border-bottom: 1px solid var(--ez-color-border);
  }
  .app-export__history-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-xs);
    margin-top: var(--ez-space-2xs);
  }
  .app-export__record {
    font-size: var(--ez-fs-xs);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 20rem;
    overflow: auto;
    margin: var(--ez-space-xs) 0 0;
    padding: var(--ez-space-sm);
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-border-soft);
  }
  .app-export__history-main {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
  }
</style>
