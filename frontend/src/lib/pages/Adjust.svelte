<script lang="ts">
  import Flipbook from '../components/Flipbook.svelte';
  import NextStep from '../components/NextStep.svelte';
  import Progress from '../components/Progress.svelte';
  import Wipe from '../components/Wipe.svelte';
  import { LOOP_MAX, loopWindow } from '../flipbook';
  import { runLabelWithId } from '../selectionLabel';
  import { untrack } from 'svelte';
  import { Banner, Button, Card, EmptyState, Input, PageHeader, SectionLabel, Select, Skeleton, Stat, StatGrid, Toggle } from '@etzianco/ui';

  import {
    CROPS,
    DEFAULT_DEFLICKER,
    DEFAULT_STABILIZE,
    SMOOTHINGS,
    STRENGTHS,
    WINDOWS,
    adjustFailureText,
    brightnessChart,
    colourChart,
    formatFrame,
    formatStops,
    pathChart
  } from '../adjust';
  import LineChart from '../components/LineChart.svelte';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { flickerSeries, getSamplingItems, listAdjustments, listSamplingRuns, stabilizationSeries } from '../ipc/app';
  import type { Adjustments, FlickerPoint, FlickerRun, RunSummary, SampledItem, StabilizationRun, StabilizationSeries } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { adjustJob, cachedAdjustedFrame, runAdjustment, stopAdjustment } from '../stores/adjust.svelte';
  import type { AdjustKind } from '../stores/adjust.svelte';
  import { nav } from '../stores/nav.svelte';
  import { project } from '../stores/project.svelte';

  let { ongohome }: { ongohome: () => void } = $props();

  const root = $derived(project.root);
  const CHART_POINTS = 600;

  let runs = $state<RunSummary[] | null>(null);
  let adjustments = $state<Adjustments | null>(null);
  let loadError = $state('');
  let runId = $state('');

  let strength = $state(String(DEFAULT_DEFLICKER.strength));
  let windowSize = $state(String(DEFAULT_DEFLICKER.window));
  let colour = $state(DEFAULT_DEFLICKER.colour);
  let smoothing = $state(String(DEFAULT_STABILIZE.smoothing));
  let maxCrop = $state(String(DEFAULT_STABILIZE.max_crop));

  let viewFlicker = $state('');
  let viewStab = $state('');
  let flickerPoints = $state<FlickerPoint[] | null>(null);
  let stabSeries = $state<StabilizationSeries | null>(null);
  let seriesError = $state('');

  let frameNo = $state('1');
  let applyFlicker = $state(true);
  let applyStab = $state(true);
  let before = $state('');
  let after = $state('');
  let frameError = $state('');
  /** The loop around the chosen frame: as shot, or with the applied runs. */
  let playAdjusted = $state(true);
  let loopItems = $state<SampledItem[] | null>(null);
  let loopError = $state('');
  const LOOP_FPS = 12;

  const job = $derived(adjustJob.state);
  const mine = $derived(job.root === root);
  const busy = $derived(job.running);
  const runningKind = $derived<AdjustKind | null>(mine && job.running ? job.kind : null);
  const progress = $derived(mine ? job.progress : null);

  /** Flips once per load; a derived boolean only notifies when it changes. */
  const loaded = $derived(adjustments !== null);
  const run = $derived(runs?.find((r) => String(r.id) === runId) ?? null);
  const flickerRuns = $derived((adjustments?.flicker ?? []).filter((r) => String(r.sampling_run_id) === runId));
  const stabRuns = $derived((adjustments?.stabilization ?? []).filter((r) => String(r.sampling_run_id) === runId));
  const flicker = $derived<FlickerRun | null>(flickerRuns.find((r) => String(r.id) === viewFlicker) ?? null);
  const stab = $derived<StabilizationRun | null>(stabRuns.find((r) => String(r.id) === viewStab) ?? null);
  const frames = $derived(run?.selected_count ?? 0);
  const frameIndex = $derived(Math.min(Math.max(1, Math.round(Number(frameNo)) || 1), Math.max(1, frames)) - 1);

  const runOptions = $derived((runs ?? []).map((r) => ({ value: String(r.id), label: runLabelWithId(r) })));
  const strengthOptions = $derived(STRENGTHS.map((v) => ({ value: String(v), label: formatNumber(v, { style: 'percent' }) })));
  const windowOptions = $derived(WINDOWS.map((v) => ({ value: String(v), label: t('adjust-frames-option', { count: v }) })));
  const smoothingOptions = $derived(SMOOTHINGS.map((v) => ({ value: String(v), label: t('adjust-frames-option', { count: v }) })));
  const cropOptions = $derived(CROPS.map((v) => ({ value: String(v), label: formatNumber(v, { style: 'percent' }) })));
  const flickerOptions = $derived(
    flickerRuns.map((r) => ({
      value: String(r.id),
      label: t('adjust-flicker-option', {
        id: r.id,
        date: formatDate(r.created_at * 1000, { dateStyle: 'medium', timeStyle: 'short' }),
        strength: formatNumber(r.config.strength, { style: 'percent' }),
        window: r.config.window
      })
    }))
  );
  const stabOptions = $derived(
    stabRuns.map((r) => ({
      value: String(r.id),
      label: t('adjust-stab-option', {
        id: r.id,
        date: formatDate(r.created_at * 1000, { dateStyle: 'medium', timeStyle: 'short' }),
        crop: formatNumber(r.crop, { style: 'percent', maximumFractionDigits: 1 })
      })
    }))
  );

  const formatX = (x: number) => formatNumber(x);
  const valueLabel = (label: string, value: string) => t('chart-value', { label, value });

  async function load(): Promise<void> {
    const forRoot = root;
    loadError = '';
    try {
      const [r, a] = await Promise.all([listSamplingRuns(forRoot), listAdjustments(forRoot)]);
      if (forRoot !== root) return;
      runs = r;
      adjustments = a;
      if (!runId && r.length > 0) runId = String(r[0].id);
    } catch (err) {
      if (forRoot === root) loadError = errorText(err as IpcError);
    }
  }

  $effect(() => {
    if (!root) return;
    runs = null;
    adjustments = null;
    runId = '';
    void load();
  });

  // The newest run of each kind is shown when the selection changes (and only then: a
  // refreshed list keeps the result and frame the user chose).
  $effect(() => {
    void runId;
    void loaded;
    untrack(() => {
      viewFlicker = flickerRuns[0] ? String(flickerRuns[0].id) : '';
      viewStab = stabRuns[0] ? String(stabRuns[0].id) : '';
      frameNo = '1';
    });
  });

  // A finished analysis (here or after coming back) refreshes the list and shows its run.
  $effect(() => {
    const result = job.result;
    if (!result || !mine || job.running) return;
    const forRoot = root;
    listAdjustments(forRoot)
      .then((a) => {
        if (forRoot !== root) return;
        adjustments = a;
        if (result.run && result.kind === 'deflicker') viewFlicker = String(result.run.id);
        if (result.run && result.kind === 'stabilize') viewStab = String(result.run.id);
      })
      .catch(() => {});
  });

  let seriesSeq = 0;
  $effect(() => {
    const f = viewFlicker;
    const s = viewStab;
    const forRoot = root;
    if (!forRoot) return;
    const seq = ++seriesSeq;
    seriesError = '';
    flickerPoints = null;
    stabSeries = null;
    (async () => {
      try {
        const [fp, ss] = await Promise.all([
          f ? flickerSeries(forRoot, Number(f), CHART_POINTS) : Promise.resolve(null),
          s ? stabilizationSeries(forRoot, Number(s), CHART_POINTS) : Promise.resolve(null)
        ]);
        if (seq !== seriesSeq) return;
        flickerPoints = fp;
        stabSeries = ss;
      } catch (err) {
        if (seq === seriesSeq) seriesError = errorText(err as IpcError);
      }
    })();
  });

  // The before/after stills follow the frame and the runs applied (cached for the selection shown).
  let frameSeq = 0;
  $effect(() => {
    const forRoot = root;
    const r = run;
    const index = frameIndex;
    const fl = applyFlicker && flicker ? flicker.id : null;
    const st = applyStab && stab ? stab.id : null;
    if (!forRoot || !r) return;
    const seq = ++frameSeq;
    frameError = '';
    const timer = setTimeout(async () => {
      try {
        const [b, a] = await Promise.all([
          cachedAdjustedFrame(forRoot, r.id, index, null, null, false),
          cachedAdjustedFrame(forRoot, r.id, index, fl, st, true)
        ]);
        if (seq !== frameSeq) return;
        before = b;
        after = a;
      } catch (err) {
        if (seq === frameSeq) frameError = errorText(err as IpcError);
      }
    }, 150);
    return () => clearTimeout(timer);
  });

  // The loop: at most 60 frames around the chosen frame; their times come once per window.
  const loopStart = $derived(loopWindow(frameIndex, frames, LOOP_MAX).start);
  const loopLength = $derived(loopWindow(frameIndex, frames, LOOP_MAX).length);
  let loopTry = $state(0);
  const loopFl = $derived(playAdjusted && applyFlicker && flicker ? flicker.id : null);
  const loopSt = $derived(playAdjusted && applyStab && stab ? stab.id : null);
  const loopAdjusted = $derived(playAdjusted && (loopFl !== null || loopSt !== null));
  let loopSeq = 0;
  $effect(() => {
    const forRoot = root;
    const r = run;
    const start = loopStart;
    const length = loopLength;
    void loopTry;
    if (!forRoot || !r || length === 0) {
      loopItems = [];
      return;
    }
    const seq = ++loopSeq;
    loopError = '';
    loopItems = null;
    const timer = setTimeout(async () => {
      try {
        const page = await getSamplingItems(forRoot, r.id, start, length);
        if (seq === loopSeq) loopItems = page.items;
      } catch (err) {
        if (seq === loopSeq) loopError = errorText(err as IpcError);
      }
    }, 250);
    return () => clearTimeout(timer);
  });

  function loopFrame(i: number): Promise<string> {
    const item = loopItems?.[i];
    if (!item || !run) return Promise.reject(new Error('no frame'));
    return cachedAdjustedFrame(root, run.id, item.sequence_index, loopFl, loopSt, loopAdjusted);
  }

  function analyseFlicker(): void {
    if (!run || busy) return;
    void runAdjustment(root, 'deflicker', run.id, { strength: Number(strength), window: Number(windowSize), colour });
  }

  function analyseStab(): void {
    if (!run || busy) return;
    void runAdjustment(root, 'stabilize', run.id, { smoothing: Number(smoothing), max_crop: Number(maxCrop) });
  }

  function step(by: number): void {
    frameNo = String(Math.min(Math.max(1, frameIndex + 1 + by), Math.max(1, frames)));
  }

  const error = $derived(mine && job.error ? (job.error.kind === 'adjust' ? adjustFailureText((job.error.args as { reason?: string })?.reason ?? null) : errorText(job.error)) : '');
  const stopped = $derived(mine && !job.running && job.result !== null && job.result.run === null);
</script>

<div class="app-page">
  <PageHeader title={t('adjust-title')} />

  {#if !root}
    <Card>
      <EmptyState title={t('clean-no-project-title')} description={t('adjust-no-project-description')}>
        {#snippet action()}
          <Button onclick={ongohome}>{t('clean-go-import')}</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else if loadError}
    <Card>
      <div role="alert" class="app-adjust__error">
        <p>{loadError}</p>
        <Button onclick={load}>{t('action-retry')}</Button>
      </div>
    </Card>
  {:else if runs === null || adjustments === null}
    <Card><Skeleton lines={5} label={t('clean-loading')} /></Card>
  {:else if runs.length === 0}
    <Card>
      <EmptyState title={t('export-no-runs-title')} description={t('adjust-no-runs-description')}>
        {#snippet action()}
          <Button onclick={() => nav.go('sampling')}>{t('export-go-sampling')}</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else}
    <p class="app-adjust__note">{t('adjust-intro')}</p>

    <Card>
      <Select label={t('export-run')} options={runOptions} bind:value={runId} disabled={busy} />
    </Card>

    {#if error}<Banner tone="error" text={error} ondismiss={() => adjustJob.dismiss()} dismissLabel={t('clean-dismiss')} />{/if}
    {#if stopped}<Banner tone="neutral" text={t('adjust-stopped')} ondismiss={() => adjustJob.dismiss()} dismissLabel={t('clean-dismiss')} />{/if}
    {#if busy && !mine}<p class="app-adjust__note" aria-live="polite">{t('adjust-other-running')}</p>{/if}

    <Card>
      <SectionLabel text={t('adjust-flicker-title')} />
      <p class="app-adjust__note">{t('adjust-flicker-help')}</p>
      <div class="app-adjust__form">
        <Select label={t('adjust-strength')} options={strengthOptions} bind:value={strength} disabled={busy} />
        <Select label={t('adjust-window')} options={windowOptions} bind:value={windowSize} disabled={busy} />
      </div>
      <Toggle label={t('adjust-colour')} bind:checked={colour} disabled={busy} />
      <div class="app-adjust__actions">
        <Button onclick={analyseFlicker} disabled={!run || busy}>{runningKind === 'deflicker' ? t('adjust-analysing') : t('adjust-analyse')}</Button>
        {#if runningKind === 'deflicker'}
          <Button variant="secondary" onclick={() => void stopAdjustment()} disabled={job.cancelling}>{job.cancelling ? t('clean-stopping') : t('clean-stop')}</Button>
        {/if}
      </div>
      {#if runningKind === 'deflicker' && progress}
        <div class="app-adjust__progress">
          <Progress done={progress.done} total={progress.total} label={t('adjust-analysing')} />
        </div>
      {/if}

      {#if flickerRuns.length > 0}
        <div class="app-adjust__form">
          <Select label={t('adjust-result')} options={flickerOptions} bind:value={viewFlicker} />
        </div>
        {#if flicker}
          <StatGrid>
            <Stat label={t('adjust-stat-largest')} value={formatStops(flicker.results.max_stops)} />
            <Stat label={t('adjust-stat-mean')} value={formatStops(flicker.results.mean_stops)} />
            <Stat label={t('adjust-stat-colour')} value={formatStops(flicker.results.max_colour_stops)} />
            <Stat label={t('adjust-stat-measured')} value={t('adjust-measured-of', { measured: formatNumber(flicker.results.measured), frames: formatNumber(flicker.results.frames) })} />
          </StatGrid>
          {#if seriesError}
            <Banner tone="error" text={seriesError} />
          {:else if flickerPoints === null}
            <Skeleton lines={4} label={t('clean-loading')} />
          {:else}
            {@const b = brightnessChart(flickerPoints)}
            <LineChart title={t('adjust-chart-brightness')} xs={b.xs} series={b.series} {formatX} formatY={formatStops} xLabel={t('adjust-axis-frame')} tableLabel={t('adjust-show-table')} {valueLabel} noValue={t('chart-no-value')} />
            {#if flicker.config.colour}
              {@const red = colourChart(flickerPoints, 'red')}
              {@const blue = colourChart(flickerPoints, 'blue')}
              <LineChart title={t('adjust-chart-red')} xs={red.xs} series={red.series} {formatX} formatY={formatStops} xLabel={t('adjust-axis-frame')} tableLabel={t('adjust-show-table')} {valueLabel} noValue={t('chart-no-value')} />
              <LineChart title={t('adjust-chart-blue')} xs={blue.xs} series={blue.series} {formatX} formatY={formatStops} xLabel={t('adjust-axis-frame')} tableLabel={t('adjust-show-table')} {valueLabel} noValue={t('chart-no-value')} />
            {/if}
          {/if}
        {/if}
      {:else}
        <EmptyState title={t('adjust-none-title')} description={t('adjust-flicker-none')} compact />
      {/if}
    </Card>

    <Card>
      <SectionLabel text={t('adjust-stab-title')} />
      <p class="app-adjust__note">{t('adjust-stab-help')}</p>
      <div class="app-adjust__form">
        <Select label={t('adjust-smoothing')} options={smoothingOptions} bind:value={smoothing} disabled={busy} />
        <Select label={t('adjust-max-crop')} options={cropOptions} bind:value={maxCrop} disabled={busy} />
      </div>
      <div class="app-adjust__actions">
        <Button onclick={analyseStab} disabled={!run || busy}>{runningKind === 'stabilize' ? t('adjust-analysing') : t('adjust-analyse')}</Button>
        {#if runningKind === 'stabilize'}
          <Button variant="secondary" onclick={() => void stopAdjustment()} disabled={job.cancelling}>{job.cancelling ? t('clean-stopping') : t('clean-stop')}</Button>
        {/if}
      </div>
      {#if runningKind === 'stabilize' && progress}
        <div class="app-adjust__progress">
          <Progress done={progress.done} total={progress.total} label={t('adjust-analysing')} />
        </div>
      {/if}

      {#if stabRuns.length > 0}
        <div class="app-adjust__form">
          <Select label={t('adjust-result')} options={stabOptions} bind:value={viewStab} />
        </div>
        {#if stab}
          <StatGrid>
            <Stat label={t('adjust-stat-crop')} value={formatNumber(stab.crop, { style: 'percent', maximumFractionDigits: 1 })} />
            <Stat label={t('adjust-stat-shift')} value={formatFrame(stab.results.max_shift)} />
            <Stat label={t('adjust-stat-pairs')} value={t('adjust-measured-of', { measured: formatNumber(stab.results.measured), frames: formatNumber(Math.max(0, stab.results.frames - 1)) })} />
          </StatGrid>
          {#if stab.results.measured < Math.max(0, stab.results.frames - 1) / 2}
            <Banner tone="warning" text={t('adjust-stab-few')} />
          {/if}
          {#if seriesError}
            <Banner tone="error" text={seriesError} />
          {:else if stabSeries === null}
            <Skeleton lines={4} label={t('clean-loading')} />
          {:else}
            {@const x = pathChart(stabSeries.points, 0)}
            {@const y = pathChart(stabSeries.points, 1)}
            <LineChart title={t('adjust-chart-horizontal')} xs={x.xs} series={x.series} {formatX} formatY={formatFrame} xLabel={t('adjust-axis-frame')} tableLabel={t('adjust-show-table')} {valueLabel} noValue={t('chart-no-value')} />
            <LineChart title={t('adjust-chart-vertical')} xs={y.xs} series={y.series} {formatX} formatY={formatFrame} xLabel={t('adjust-axis-frame')} tableLabel={t('adjust-show-table')} {valueLabel} noValue={t('chart-no-value')} />
          {/if}
        {/if}
      {:else}
        <EmptyState title={t('adjust-none-title')} description={t('adjust-stab-none')} compact />
      {/if}
    </Card>

    <Card>
      <SectionLabel text={t('adjust-compare-title')} />
      <div class="app-adjust__frame-controls">
        <Button size="sm" variant="ghost" onclick={() => step(-1)} disabled={frameIndex === 0}>{t('adjust-previous')}</Button>
        <Input type="number" label={t('adjust-frame', { count: frames })} bind:value={frameNo} />
        <Button size="sm" variant="ghost" onclick={() => step(1)} disabled={frameIndex + 1 >= frames}>{t('adjust-next')}</Button>
        {#if flicker}<Toggle label={t('adjust-apply-flicker')} bind:checked={applyFlicker} />{/if}
        {#if stab}<Toggle label={t('adjust-apply-stab')} bind:checked={applyStab} />{/if}
      </div>
      {#if !flicker && !stab}
        <p class="app-adjust__note">{t('adjust-compare-none')}</p>
      {/if}
      {#if frameError}
        <Banner tone="error" text={frameError} />
      {:else}
        <Wipe {before} {after} beforeAlt={t('adjust-as-shot', { frame: frameIndex + 1 })} afterAlt={t('adjust-adjusted', { frame: frameIndex + 1 })} />
      {/if}
    </Card>

    <Card>
      <SectionLabel text={t('adjust-loop-title')} />
      <p class="app-adjust__note">{t('adjust-loop-help', { count: formatNumber(loopLength) })}</p>
      {#if flicker || stab}
        <Toggle label={t('adjust-loop-adjusted')} bind:checked={playAdjusted} />
      {/if}
      {#if loopError}
        <div role="alert" class="app-adjust__error">
          <p>{loopError}</p>
          <Button onclick={() => loopTry++}>{t('action-retry')}</Button>
        </div>
      {:else if loopItems === null}
        <Skeleton variant="rect" size="12rem" label={t('clean-loading')} />
      {:else}
        <Flipbook
          {root}
          frames={loopItems}
          fps={LOOP_FPS}
          total={loopLength}
          loadFrame={loopFrame}
          resetKey={`${root}|${run?.id}|${loopStart}|${loopAdjusted}|${loopFl}|${loopSt}`}
        />
      {/if}
    </Card>
  {/if}
  <NextStep step="adjust" />
</div>

<style>
  .app-adjust__note {
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-xs) 0;
  }
  .app-adjust__error {
    color: var(--ez-color-error);
  }
  .app-adjust__form {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr));
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-adjust__actions,
  .app-adjust__frame-controls {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-sm);
  }
  .app-adjust__progress {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-sm);
  }
</style>
