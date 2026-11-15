<script lang="ts">
  import { Badge, Banner, Button, Card, EmptyState, PageHeader, SectionLabel, Select, Skeleton, Stat, StatGrid } from '@etzianco/ui';

  import { CODECS, QUALITIES, RESOLUTIONS, durationSeconds, exportFailureText, formatBytes, outputFrames, supports } from '../export';
  import type { ResolutionKey } from '../export';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { ffmpegInfo, listExports, listSamplingRuns } from '../ipc/app';
  import type { Codec, ExportRecord, FfmpegInfo, Quality, RunSummary } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { FPS_OPTIONS, MULTIPLIERS } from '../sampling';
  import { exportJob, runExport, stopExport } from '../stores/export.svelte';
  import { project } from '../stores/project.svelte';
  import { formatDuration } from '../timeline';

  let { ongohome, ongosampling }: { ongohome: () => void; ongosampling: () => void } = $props();

  const root = $derived(project.root);

  let info = $state<FfmpegInfo | null>(null);
  let runs = $state<RunSummary[] | null>(null);
  let loadError = $state('');

  let runId = $state('');
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

  const run = $derived(runs?.find((r) => String(r.id) === runId) ?? null);
  const frames = $derived(run ? outputFrames(run.selected_count, Number(speed)) : 0);
  const duration = $derived(run ? durationSeconds(run.selected_count, Number(fps), Number(speed)) : 0);
  const canExport = $derived(!!run && supports(info, codec) && !exporting);

  const runOptions = $derived(
    (runs ?? []).map((r) => ({ value: String(r.id), label: t('export-run-option', { id: r.id, count: r.selected_count }) }))
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
      const [i, r, h] = await Promise.all([ffmpegInfo(), listSamplingRuns(forRoot), listExports(forRoot)]);
      if (forRoot !== root) return;
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

  function start(): void {
    if (!run || !canExport) return;
    const r = RESOLUTIONS.find((x) => x.key === resolution) ?? RESOLUTIONS[2];
    void runExport(root, run.id, { codec, fps: Number(fps), speed: Number(speed), width: r.width, height: r.height, quality });
  }

  function cancel(): void {
    void stopExport();
  }

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
          <Select label={t('export-codec')} options={codecOptions} bind:value={codec} disabled={exporting} />
          <Select label={t('sampling-fps')} options={fpsOptions} bind:value={fps} disabled={exporting} />
          <Select label={t('sampling-multiplier')} options={speedOptions} bind:value={speed} disabled={exporting} />
          <Select label={t('export-resolution')} options={resolutionOptions} bind:value={resolution} disabled={exporting} />
          <Select label={t('export-quality')} options={qualityOptions} bind:value={quality} disabled={exporting} />
        </div>
        <p class="app-export__note">{t(`export-codec-${codec}-help`)}</p>

        {#if run}
          <StatGrid>
            <Stat label={t('export-stat-photos')} value={formatNumber(run.selected_count)} />
            <Stat label={t('export-stat-frames')} value={formatNumber(frames)} />
            <Stat label={t('export-stat-duration')} value={formatDuration(duration)} />
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
            <progress max={Math.max(progress.total, 1)} value={progress.done} aria-label={t('export-exporting')}></progress>
            <span aria-live="polite">{t('clean-progress', { done: formatNumber(progress.done), total: formatNumber(progress.total) })}</span>
          </div>
        {/if}
      </Card>
    {/if}

    {#if error}<Banner tone="error" text={error} />{/if}

    {#if result}
      {#if result.status === 'completed'}
        <Banner
          tone="success"
          title={t('export-done-title')}
          text={t('export-done', {
            duration: formatDuration(result.duration_seconds),
            size: formatBytes(result.file_size ?? 0),
            codec: t(`export-codec-${result.settings.codec}`)
          })}
          ondismiss={() => exportJob.dismiss()}
          dismissLabel={t('clean-dismiss')}
        >
          <p class="app-export__path">{result.output_path}</p>
        </Banner>
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
                    run: h.sampling_run_id,
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
            </li>
          {/each}
        </ul>
      {/if}
    </Card>
  {/if}
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
  .app-export__actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-md);
  }
  .app-export__progress {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-sm);
  }
  .app-export__progress progress {
    flex: 1 1 12rem;
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
  .app-export__history-main {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
  }
</style>
