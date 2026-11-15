<script lang="ts">
  import { Badge, Banner, Button, Card, EmptyState, Input, PageHeader, SectionLabel, Select, Skeleton, Stat, StatGrid, Tabs, Toggle } from '@etzianco/ui';

  import { limited } from '../clean';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { createSamplingRun, getSamplingItems, getThumbnail, listSamplingRuns, previewSampling, timelineDistribution, verifySamplingRun } from '../ipc/app';
  import { openContactSheet } from '../stores/contactSheet.svelte';
  import type { DayCount, RunSummary, SampledItem, SamplingPreview, VerifyReport } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { FPS_OPTIONS, METHODS, MULTIPLIERS, POLICIES, UNIT_SECONDS, buildConfig, defaultForm, estimatedDurationSeconds } from '../sampling';
  import type { IntervalUnit, MethodName } from '../sampling';
  import { project } from '../stores/project.svelte';
  import { barPercent, formatDuration, formatInstant } from '../timeline';

  let { ongohome }: { ongohome: () => void } = $props();

  const STRIP_PAGE = 40;
  const root = $derived(project.root);

  let form = $state(defaultForm());
  let formInvalid = $state(false);
  /** First and last capture day of the project's photographs (`YYYY-MM-DD`, UTC). */
  let span = $state<{ first: string; last: string } | null>(null);
  const wholeSpan = $derived(!!span && form.from === span.first && form.to === span.last);
  let fps = $state('30');
  let multiplier = $state('1');

  let preview = $state<SamplingPreview | null>(null);
  let previewing = $state(false);
  let saving = $state(false);
  let error = $state('');
  let savedNotice = $state<RunSummary | null>(null);

  let runs = $state<RunSummary[] | null>(null);
  let runsError = $state('');
  let verifying = $state<number | null>(null);
  let verified = $state<Record<number, VerifyReport>>({});

  /** What the strip and distribution show: the preview, or a saved run's sequence. */
  let shownItems = $state<SampledItem[]>([]);
  let shownDays = $state<DayCount[]>([]);
  /** Total length of what is shown (a preview carries its first 200 items only). */
  let shownTotal = $state(0);
  let shownRun = $state<number | null>(null);
  let loadingMore = $state(false);
  let stripCount = $state(STRIP_PAGE);
  let thumbs = $state<Record<number, string>>({});
  let requested = new Set<number>();

  const methodItems = $derived(METHODS.map((m) => ({ key: m, label: t(`sampling-method-${m}`) })));
  const unitOptions = $derived((Object.keys(UNIT_SECONDS) as IntervalUnit[]).map((u) => ({ value: u, label: t(`sampling-unit-${u}`) })));
  const policyOptions = $derived(POLICIES.map((p) => ({ value: p, label: t(`sampling-policy-${p}`) })));
  const fpsOptions = $derived(FPS_OPTIONS.map((v) => ({ value: String(v), label: t('sampling-fps-option', { fps: v }) })));
  const multiplierOptions = $derived(
    MULTIPLIERS.map((v) => ({ value: String(v), label: t('sampling-multiplier-option', { multiplier: formatNumber(v) }) }))
  );
  const maxDay = $derived(Math.max(0, ...shownDays.map((d) => d.count)));
  const duration = $derived(preview ? estimatedDurationSeconds(preview.selected_count, Number(fps), Number(multiplier)) : 0);
  const excludedTotal = $derived(
    preview ? Object.values(preview.excluded).reduce((a, b) => a + b, 0) : 0
  );

  async function loadRuns(): Promise<void> {
    const forRoot = root;
    runsError = '';
    try {
      const r = await listSamplingRuns(forRoot);
      if (forRoot === root) runs = r;
    } catch (err) {
      if (forRoot === root) runsError = errorText(err as IpcError);
    }
  }

  /** The project's date range fills From and To, unless the user already chose dates. */
  async function loadSpan(): Promise<void> {
    const forRoot = root;
    try {
      const days = await timelineDistribution(forRoot, 'day');
      if (forRoot !== root || days.length === 0) return;
      span = { first: days[0].bucket, last: days[days.length - 1].bucket };
      if (!form.from && !form.to) useWholeSpan();
    } catch {
      // Without the range the fields stay empty, which means "all dates" anyway.
    }
  }

  function useWholeSpan(): void {
    if (!span) return;
    form.from = span.first;
    form.to = span.last;
  }

  $effect(() => {
    if (!root) return;
    preview = null;
    runs = null;
    span = null;
    form.from = '';
    form.to = '';
    verified = {};
    error = '';
    savedNotice = null;
    formInvalid = false;
    verifying = null;
    shownDays = [];
    shownTotal = 0;
    shownItems = [];
    shownRun = null;
    thumbs = {};
    requested = new Set();
    void loadRuns();
    void loadSpan();
  });

  $effect(() => {
    const missing = shownItems.slice(0, stripCount).filter((i) => !requested.has(i.image_id));
    if (missing.length === 0) return;
    for (const i of missing) requested.add(i.image_id);
    const forRoot = root;
    void limited(
      missing.map((i) => async () => {
        try {
          const url = await getThumbnail(forRoot, i.image_id, 256);
          if (forRoot === root) thumbs = { ...thumbs, [i.image_id]: url };
        } catch {
          /* the tile keeps its file name; nothing else depends on the preview image */
        }
      }),
      4
    );
  });

  function show(items: SampledItem[], days: DayCount[], total: number, run: number | null): void {
    shownItems = items;
    shownDays = days;
    shownTotal = total;
    shownRun = run;
    stripCount = STRIP_PAGE;
  }

  /** Reveal more of the strip; a saved run fetches its next page when needed. */
  async function showMore(): Promise<void> {
    if (stripCount < shownItems.length) {
      stripCount += STRIP_PAGE;
      return;
    }
    if (shownRun === null || shownItems.length >= shownTotal || loadingMore) return;
    const forRoot = root;
    const run = shownRun;
    loadingMore = true;
    try {
      const page = await getSamplingItems(forRoot, run, shownItems.length, 200);
      if (forRoot === root && shownRun === run) {
        shownItems = [...shownItems, ...page.items];
        stripCount += STRIP_PAGE;
      }
    } catch (err) {
      if (forRoot === root) runsError = errorText(err as IpcError);
    } finally {
      loadingMore = false;
    }
  }

  async function runPreview(): Promise<void> {
    const config = buildConfig(form);
    formInvalid = config === null;
    if (!config || previewing) return;
    const forRoot = root;
    previewing = true;
    error = '';
    savedNotice = null;
    try {
      const p = await previewSampling(forRoot, config);
      if (forRoot !== root) return;
      preview = p;
      show(p.items, p.day_counts, p.selected_count, null);
    } catch (err) {
      if (forRoot === root) error = errorText(err as IpcError);
    } finally {
      previewing = false;
    }
  }

  /** Saves exactly what the preview shows, not whatever the form says by now. */
  async function save(): Promise<void> {
    const config = preview?.config;
    if (!config || saving) return;
    const forRoot = root;
    saving = true;
    error = '';
    try {
      const run = await createSamplingRun(forRoot, config);
      if (forRoot !== root) return;
      savedNotice = run;
      await loadRuns();
    } catch (err) {
      if (forRoot === root) error = errorText(err as IpcError);
    } finally {
      saving = false;
    }
  }

  async function verify(run: number): Promise<void> {
    const forRoot = root;
    verifying = run;
    try {
      const v = await verifySamplingRun(forRoot, run);
      if (forRoot === root) verified = { ...verified, [run]: v };
    } catch (err) {
      if (forRoot === root) runsError = errorText(err as IpcError);
    } finally {
      verifying = null;
    }
  }

  async function showRun(run: number): Promise<void> {
    const forRoot = root;
    try {
      const page = await getSamplingItems(forRoot, run, 0, 200);
      if (forRoot === root) show(page.items, page.day_counts, page.total, run);
    } catch (err) {
      if (forRoot === root) runsError = errorText(err as IpcError);
    }
  }

  function runDescription(r: RunSummary): string {
    const c = r.config;
    if (c.method === 'fixed-interval') {
      return t(`sampling-run-fixed-${c.policy}`, { interval: formatDuration(c.interval_seconds) });
    }
    if (c.method === 'every-nth') return t('sampling-run-nth', { n: c.n });
    return t('sampling-run-uniform', { count: c.count });
  }

  const pct = (v: number) => formatNumber(v, { style: 'percent', maximumFractionDigits: 1 });
</script>

<div class="app-page">
  <PageHeader title={t('sampling-title')} />

  {#if !root}
    <Card>
      <EmptyState title={t('clean-no-project-title')} description={t('sampling-no-project-description')}>
        {#snippet action()}
          <Button onclick={ongohome}>{t('clean-go-import')}</Button>
        {/snippet}
      </EmptyState>
    </Card>
  {:else}
    <p class="app-sampling__note">{t('sampling-intro')}</p>

    <Card>
      <SectionLabel text={t('sampling-method-title')} />
      <Tabs items={methodItems} active={form.method} label={t('sampling-method-title')} onchange={(k: string) => (form.method = k as MethodName)} />
      <div class="app-sampling__form">
        {#if form.method === 'fixed-interval'}
          <Input label={t('sampling-interval')} type="number" bind:value={form.interval} />
          <Select label={t('sampling-unit')} options={unitOptions} bind:value={form.unit} />
          <Select label={t('sampling-policy')} options={policyOptions} bind:value={form.policy} />
          <Input label={t('sampling-tolerance')} type="number" bind:value={form.tolerance} placeholder={t('sampling-tolerance-placeholder')} />
        {:else if form.method === 'every-nth'}
          <Input label={t('sampling-n')} type="number" bind:value={form.n} />
          <Input label={t('sampling-offset')} type="number" bind:value={form.offset} />
        {:else}
          <Input label={t('sampling-count')} type="number" bind:value={form.count} />
        {/if}
      </div>
      <p class="app-sampling__note">{t(`sampling-method-${form.method}-help`)}</p>

      <SectionLabel text={t('sampling-input-title')} />
      <div class="app-sampling__form">
        <Input label={t('sampling-from')} type="date" bind:value={form.from} />
        <Input label={t('sampling-to')} type="date" bind:value={form.to} />
      </div>
      {#if span && !wholeSpan}
        <Button size="sm" variant="ghost" onclick={useWholeSpan}>{t('sampling-all-dates')}</Button>
      {/if}
      <Toggle label={t('sampling-include-flagged')} bind:checked={form.includeFlagged} />
      <p class="app-sampling__note">{t('sampling-input-note')}</p>
      {#if formInvalid}<p role="alert" class="app-sampling__error">{t('sampling-invalid')}</p>{/if}

      <div class="app-sampling__actions">
        <Button onclick={runPreview} disabled={previewing || saving}>{previewing ? t('sampling-previewing') : t('sampling-preview')}</Button>
        <Button variant="secondary" onclick={save} disabled={previewing || saving || preview === null}>
          {saving ? t('sampling-saving') : t('sampling-save')}
        </Button>
      </div>
    </Card>

    {#if error}<Banner tone="error" text={error} />{/if}
    {#if savedNotice}
      <Banner
        tone="success"
        text={t('sampling-saved', { id: savedNotice.id, count: savedNotice.selected_count })}
        ondismiss={() => (savedNotice = null)}
        dismissLabel={t('clean-dismiss')}
      />
    {/if}

    <Card>
      <SectionLabel text={t('sampling-preview-title')} />
      {#if previewing && !preview}
        <Skeleton lines={4} label={t('sampling-previewing')} />
      {:else if !preview}
        <EmptyState title={t('sampling-preview-empty-title')} description={t('sampling-preview-empty-description')} compact />
      {:else}
        <div class:app-sampling__refreshing={previewing}>
          <StatGrid>
            <Stat label={t('sampling-stat-input')} value={formatNumber(preview.input_count)} />
            <Stat label={t('sampling-stat-selected')} value={formatNumber(preview.selected_count)} tone="positive" />
            <Stat label={t('sampling-stat-reduction')} value={pct(preview.reduction)} />
            <Stat
              label={t('sampling-stat-interval')}
              value={preview.effective_interval_seconds === null ? t('sampling-stat-none') : formatDuration(preview.effective_interval_seconds)}
            />
            <Stat
              label={t('sampling-stat-unmatched')}
              value={formatNumber(preview.unmatched_count)}
              tone={preview.unmatched_count > 0 ? 'warning' : 'neutral'}
            />
          </StatGrid>
          <div class="app-sampling__form">
            <Select label={t('sampling-fps')} options={fpsOptions} bind:value={fps} />
            <Select label={t('sampling-multiplier')} options={multiplierOptions} bind:value={multiplier} />
          </div>
          <p class="app-sampling__duration">{t('sampling-duration', { duration: formatDuration(duration) })}</p>
          {#if excludedTotal > 0}
            <p class="app-sampling__note">
              {t('sampling-excluded', {
                rejected: preview.excluded.rejected,
                quarantined: preview.excluded.quarantined,
                flagged: preview.excluded.flagged,
                undated: preview.excluded.no_timestamp,
                outside: preview.excluded.out_of_period
              })}
            </p>
          {/if}
          {#if preview.selected_count === 0}
            <EmptyState title={t('sampling-nothing-selected-title')} description={t('sampling-nothing-selected-description')} compact />
          {/if}
        </div>
      {/if}

      {#if shownItems.length > 0}
        <h3 class="app-sampling__heading">
          {shownRun === null
            ? t('sampling-sequence-preview', { count: shownTotal })
            : t('sampling-sequence-run', { id: shownRun, count: shownTotal })}
        </h3>
        <ol class="app-sampling__bars">
          {#each shownDays as d (d.day)}
            <li>
              <span>{formatDate(Date.parse(`${d.day}T00:00:00Z`), { timeZone: 'UTC', dateStyle: 'medium' })}</span>
              <span class="app-sampling__track" aria-hidden="true"><span style:width="{barPercent(d.count, maxDay)}%"></span></span>
              <span>{formatNumber(d.count)}</span>
            </li>
          {/each}
        </ol>
        <ol class="app-sampling__strip">
          {#each shownItems.slice(0, stripCount) as item (item.image_id)}
            <li>
              <span class="app-sampling__thumb">
                {#if thumbs[item.image_id]}<img src={thumbs[item.image_id]} alt="" />{/if}
              </span>
              <span class="app-sampling__index">{formatNumber(item.sequence_index + 1)}</span>
              <span class="app-sampling__path" title={item.relative_path}>{item.relative_path}</span>
              <span class="app-sampling__when">{formatInstant(item.epoch_ms)}</span>
            </li>
          {/each}
        </ol>
        {@const visibleCount = Math.min(stripCount, shownItems.length)}
        {#if visibleCount < shownTotal}
          <p class="app-sampling__note">{t('sampling-strip-shown', { shown: formatNumber(visibleCount), total: formatNumber(shownTotal) })}</p>
          {#if visibleCount < shownItems.length || shownRun !== null}
            <Button variant="secondary" onclick={showMore} disabled={loadingMore}>{t('clean-show-more')}</Button>
          {:else}
            <p class="app-sampling__note">{t('sampling-strip-preview-limit')}</p>
          {/if}
        {/if}
      {/if}
    </Card>

    <Card>
      <SectionLabel text={t('sampling-runs-title')} />
      {#if runsError}
        <div role="alert" class="app-sampling__error">
          <p>{runsError}</p>
          <Button onclick={loadRuns}>{t('clean-retry')}</Button>
        </div>
      {:else if runs === null}
        <Skeleton lines={3} label={t('clean-loading')} />
      {:else if runs.length === 0}
        <EmptyState title={t('sampling-runs-empty-title')} description={t('sampling-runs-empty-description')} compact />
      {:else}
        <ul class="app-sampling__runs">
          {#each runs as r (r.id)}
            <li>
              <div class="app-sampling__run-main">
                <strong>{t('sampling-run-title', { id: r.id })}</strong>
                <span>{runDescription(r)}</span>
                <span class="app-sampling__note">
                  {t('sampling-run-stats', { selected: r.selected_count, input: r.input_count })} · {formatDate(r.created_at * 1000, { dateStyle: 'medium', timeStyle: 'short' })}
                </span>
                {#if verified[r.id]}
                  {@const v = verified[r.id]}
                  {#if v.inputs_unchanged && v.selection_identical && v.algorithm_unchanged}
                    <Badge variant="success">{t('sampling-verify-identical')}</Badge>
                  {:else}
                    <Badge variant="warning">{v.inputs_unchanged ? t('sampling-verify-differs') : t('sampling-verify-inputs-changed')}</Badge>
                  {/if}
                {/if}
              </div>
              <div class="app-sampling__actions">
                <Button size="sm" variant="secondary" onclick={() => showRun(r.id)}>{t('sampling-run-show')}</Button>
                <Button size="sm" variant="ghost" onclick={() => verify(r.id)} disabled={verifying !== null}>
                  {verifying === r.id ? t('sampling-verifying') : t('sampling-run-verify')}
                </Button>
                <Button size="sm" variant="ghost" onclick={() => openContactSheet({ kind: 'sampling', run: r.id })}>{t('sampling-run-sheet')}</Button>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </Card>
  {/if}
</div>

<style>
  .app-sampling__note {
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
    margin: var(--ez-space-xs) 0;
  }
  .app-sampling__error {
    color: var(--ez-color-error);
  }
  .app-sampling__form {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(11rem, 1fr));
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-sampling__actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-sm);
  }
  .app-sampling__refreshing {
    opacity: 0.6;
  }
  .app-sampling__duration {
    font-size: var(--ez-fs-md);
    margin: var(--ez-space-xs) 0;
  }
  .app-sampling__heading {
    font-size: var(--ez-fs-md);
    margin: var(--ez-space-md) 0 var(--ez-space-xs);
  }
  .app-sampling__bars,
  .app-sampling__strip,
  .app-sampling__runs {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .app-sampling__bars li {
    display: grid;
    grid-template-columns: minmax(8rem, auto) minmax(0, 1fr) auto;
    align-items: center;
    gap: var(--ez-space-sm);
    font-size: var(--ez-fs-sm);
    font-variant-numeric: tabular-nums;
    padding: var(--ez-space-2xs) 0;
  }
  .app-sampling__track {
    display: block;
    height: var(--ez-space-xs);
    border-radius: var(--ez-radius-pill);
    background: var(--ez-color-border-soft);
  }
  .app-sampling__track span {
    display: block;
    height: 100%;
    border-radius: var(--ez-radius-pill);
    background: var(--ez-color-accent);
  }
  .app-sampling__strip {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    gap: var(--ez-space-sm);
    margin-top: var(--ez-space-sm);
  }
  .app-sampling__strip li {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
    font-size: var(--ez-fs-sm);
    content-visibility: auto;
    contain-intrinsic-size: auto 10rem;
  }
  .app-sampling__thumb {
    display: block;
    aspect-ratio: 3 / 2;
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-border-soft);
    overflow: hidden;
  }
  .app-sampling__thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .app-sampling__index,
  .app-sampling__when {
    color: var(--ez-color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .app-sampling__path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .app-sampling__runs li {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--ez-space-sm);
    padding: var(--ez-space-sm) 0;
    border-bottom: 1px solid var(--ez-color-border);
  }
  .app-sampling__run-main {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 0;
  }
</style>
