<script lang="ts">
  import NextStep from '../components/NextStep.svelte';
  import { onMount, tick } from 'svelte';
  import { Button, Card, PageHeader } from '@etzianco/ui';
  import { getCurrentWebview } from '@tauri-apps/api/webview';

  import ProjectSteps from '../components/ProjectSteps.svelte';

  import { folderName } from '../erase';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { backfillMetadata, pickFolder, importRoot, integrityCheck } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { nav } from '../stores/nav.svelte';
  import { forgetProject, openProject, project, rememberProject, resetProjectState } from '../stores/project.svelte';
  import TimelineSection from './TimelineSection.svelte';

  // The folder, the import report, the integrity result and the backfill live in the
  // project store, so they survive leaving Home and are restored on launch.
  const root = $derived(project.root);
  const report = $derived(project.report);
  const integrity = $derived(project.integrity);
  const backfill = $derived(project.backfill);
  let busy = $state(false);
  let error = $state('');
  let backfillError = $state('');
  let dating = $state(false);
  const opening = $derived(project.status === 'opening');
  const others = $derived(project.recent.filter((r) => r.path !== root));
  const title = $derived(root ? folderName(root) : t('home-welcome-title'));

  /** Phase 2: resolve capture times once the import is in. Its failure never hides the import report. */
  async function doBackfill(): Promise<void> {
    if (dating) return;
    const forRoot = root;
    dating = true;
    backfillError = '';
    try {
      const result = await backfillMetadata(forRoot);
      if (forRoot === project.root) project.backfill = result; // never show one folder's dates under another
    } catch (err) {
      if (forRoot === project.root) backfillError = errorText(err as IpcError);
    } finally {
      dating = false;
    }
  }

  const idle = $derived(!busy && !dating && !opening);

  /** Set when a folder is dropped while an import is still running; the drop is not taken. */
  let dropRefused = $state(false);
  $effect(() => {
    if (idle) dropRefused = false;
  });
  // One status region, in the page from the start, says what Home is doing (GUIDELINES §2.2).
  const status = $derived(
    dropRefused
      ? t('home-drop-busy')
      : busy
        ? t('home-importing')
        : opening
          ? t('project-opening')
          : dating
            ? t('timeline-dating')
            : ''
  );

  async function doImport(): Promise<void> {
    if (!idle) return;
    const picked = await pickFolder();
    if (!picked) return; // cancelled — a no-op, not an error
    await importPath(picked);
  }

  async function importPath(picked: string): Promise<void> {
    if (!idle) return;
    project.root = picked;
    rememberProject(picked);
    busy = true;
    error = '';
    resetProjectState();
    backfillError = '';
    try {
      project.report = await importRoot(picked);
      project.integrity = await integrityCheck(picked);
    } catch (err) {
      project.report = null;
      error = errorText(err as IpcError);
    } finally {
      busy = false;
    }
    if (project.report) await doBackfill();
    scanVersion += 1;
  }

  // File > Open folder (⌘O) lands here and starts the same picker as the button.
  $effect(() => {
    if (nav.importRequested && idle) {
      nav.clearImportRequest();
      void doImport();
    }
  });

  // The folder can also be dropped on the window (webview drag-and-drop; needs no extra permission).
  let dragging = $state(false);
  let scanVersion = $state(0);
  onMount(() => {
    let off: (() => void) | undefined;
    let gone = false;
    try {
      void getCurrentWebview()
        .onDragDropEvent((event) => {
          const p = event.payload;
          if (p.type === 'enter' || p.type === 'over') dragging = true;
          else if (p.type === 'drop') {
            dragging = false;
            const first = p.paths[0];
            if (first && !idle) dropRefused = true;
            else if (first) void importPath(first);
          } else dragging = false;
        })
        .then((fn) => (gone ? fn() : (off = fn)))
        .catch(() => {}); // outside the desktop shell there is no drop event; the picker stays
    } catch {
      /* same */
    }
    return () => {
      gone = true;
      off?.();
    };
  });

  async function reopen(path: string): Promise<void> {
    if (!idle) return;
    error = '';
    backfillError = '';
    await openProject(path);
  }

  let recentHeading = $state<HTMLElement>();
  let recentList = $state<HTMLElement>();

  /** Remove a recent project, then put focus on the row that took its place (or the heading). */
  async function removeRecent(path: string, index: number): Promise<void> {
    forgetProject(path);
    await tick();
    const rows = recentList?.querySelectorAll<HTMLElement>('.app-home__recent-open') ?? [];
    const target = rows[Math.min(index, rows.length - 1)] ?? recentHeading ?? document.querySelector<HTMLElement>('main h1');
    if (target && !target.matches('button')) target.tabIndex = -1;
    target?.focus();
  }
</script>

<div class="app-page">
  <PageHeader {title} />
  {#if !root}<p class="app-home__tagline">{t('app-tagline')}</p>{/if}

  {#if !root && !opening}
    <Card class="app-home__welcome {dragging ? 'app-home__drop--active' : ''}">
      <p class="app-home__intro">{t('home-welcome-intro')}</p>
      <div class="app-home__actions">
        <button type="button" class="ez-btn ez-btn--primary ez-btn--md" aria-disabled={!idle} onclick={doImport}>
          {busy ? t('home-importing') : t('home-import-action')}
        </button>
      </div>
      <p class="app-home__drop">{dragging ? t('home-drop-active') : t('home-drop-hint')}</p>
    </Card>
  {:else if root}
    <div class="app-home__actions app-home__toolbar {dragging ? 'app-home__drop--active' : ''}">
      <button type="button" class="ez-btn ez-btn--secondary ez-btn--md" aria-disabled={!idle} onclick={() => importPath(root)}>
        {busy ? t('home-importing') : t('home-rescan')}
      </button>
      <button type="button" class="ez-btn ez-btn--ghost ez-btn--md" aria-disabled={!idle} onclick={doImport}>{t('home-switch')}</button>
      {#if dragging}<span class="app-home__drop">{t('home-drop-active')}</span>{/if}
    </div>
    <p class="app-home__root" title={root}>{root}</p>
  {/if}

  <p class="app-sr-only" role="status">{status}</p>
  {#if dropRefused}
    <p class="app-home__note">{t('home-drop-busy')}</p>
  {/if}
  {#if busy}
    <p class="app-home__progress">{t('home-importing')}</p>
  {/if}

  {#if project.status === 'unreachable'}
    <Card class="app-home__unreachable">
      <div role="alert">
        <p class="app-home__error"><strong>{folderName(project.unreachable)}</strong> · {t('project-unreachable')}</p>
        <p class="app-home__path" title={project.unreachable}>{project.unreachable}</p>
        <p>{t('project-unreachable-hint')}</p>
      </div>
      <div class="app-home__actions">
        <Button onclick={() => reopen(project.unreachable)}>{t('action-retry')}</Button>
        <Button variant="ghost" onclick={() => forgetProject(project.unreachable)}>{t('project-remove')}</Button>
      </div>
    </Card>
  {:else if opening}
    <p class="app-home__progress">{t('project-opening')}</p>
  {/if}

  {#if error}<p role="alert" class="app-home__error">{error}</p>{/if}

  {#if root && !opening}
    <ProjectSteps {root} version={scanVersion} />
  {/if}

  <!-- Above the report and the file list, so the way forward is visible without scrolling. -->
  <NextStep step="home" />

  {#if report}
    <Card class="app-home__report">
      <p class="app-home__summary">{t('home-summary', { new: report.new_rows, unchanged: report.unchanged })}</p>

      {#if report.hidden_removed > 0}
        <p class="app-home__note">{t('home-hidden-removed', { count: report.hidden_removed })}</p>
      {/if}

      {#if report.skipped_unreadable > 0}
        <p class="app-home__note">{t('home-skipped-unreadable', { count: report.skipped_unreadable })}</p>
      {/if}

      <details>
        <summary>{t('home-details-summary')}</summary>
        <dl class="app-home__stats">
          <div><dt>{t('home-stat-scanned')}</dt><dd>{formatNumber(report.scanned_count)}</dd></div>
          <div><dt>{t('home-stat-duplicates')}</dt><dd>{formatNumber(report.duplicates_found)}</dd></div>
        </dl>

        {#if integrity && integrity.missing_files.length > 0}
          <h3 class="app-home__subtitle">{t('home-integrity-missing', { count: integrity.missing_files.length })}</h3>
          <ul>
            {#each integrity.missing_files as p}
              <li>{p}</li>
            {/each}
          </ul>
        {/if}
        {#if integrity && integrity.duplicates.length > 0}
          <h3 class="app-home__subtitle">{t('home-integrity-dups', { count: integrity.duplicates.length })}</h3>
          <ul>
            {#each integrity.duplicates as group}
              <li>{group.join(', ')}</li>
            {/each}
          </ul>
        {/if}
        {#if report.duplicate_groups.length > 0}
          <h3 class="app-home__subtitle">{t('home-duplicate-groups-summary')}</h3>
          <ul>
            {#each report.duplicate_groups as group}
              <li>{group.join(', ')}</li>
            {/each}
          </ul>
        {/if}
      </details>
    </Card>
  {/if}

  {#if dating}
    <p class="app-home__progress">{t('timeline-dating')}</p>
  {:else if backfillError}
    <div role="alert" class="app-home__error">
      <p>{backfillError}</p>
      <Button onclick={doBackfill}>{t('action-retry')}</Button>
    </div>
  {:else if backfill}
    {#if backfill.newly_resolved + backfill.unresolved_this_run > 0}
      <p class="app-home__intro">
        {t('timeline-backfill-result', { resolved: backfill.newly_resolved, unresolved: backfill.unresolved_this_run })}
      </p>
    {/if}
    <TimelineSection {root} {backfill} />
  {/if}

  {#if others.length > 0}
    <Card>
      <h2 class="app-home__recent-title" bind:this={recentHeading}>{t('project-recent')}</h2>
      <ul class="app-home__recent" bind:this={recentList}>
        {#each others as r, i (r.path)}
          {@const name = folderName(r.path)}
          <li>
            <div class="app-home__recent-text">
              <span class="app-home__recent-name">{name}</span>
              <span class="app-home__path" title={r.path}>{r.path}</span>
              <span class="app-home__path">{t('project-last-opened', { date: formatDate(r.openedAt, { dateStyle: 'medium', timeStyle: 'short' }) })}</span>
              {#if project.status === 'unreachable' && project.unreachable === r.path}
                <span class="app-home__error">{t('project-unreachable')}</span>
              {/if}
            </div>
            <div class="app-home__actions">
              <button
                type="button"
                class="ez-btn ez-btn--secondary ez-btn--sm app-home__recent-open"
                aria-disabled={!idle}
                aria-label={project.unreachable === r.path ? undefined : t('project-open-named', { name })}
                onclick={() => reopen(r.path)}
              >
                {project.unreachable === r.path ? t('action-retry') : t('project-open')}
              </button>
              <button type="button" class="ez-btn ez-btn--ghost ez-btn--sm" aria-label={t('project-remove-named', { name })} onclick={() => removeRecent(r.path, i)}>
                {t('project-remove')}
              </button>
            </div>
          </li>
        {/each}
      </ul>
    </Card>
  {/if}
</div>

<style>
  .app-home__tagline {
    color: var(--ez-color-text-muted);
    margin: 0 0 var(--ez-space-md);
  }
  .app-home__intro {
    margin: 0 0 var(--ez-space-md);
  }
  .app-home__stats {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: var(--ez-space-2xs) var(--ez-space-md);
    margin: 0 0 var(--ez-space-md);
  }
  .app-home__stats dt { color: var(--ez-color-text-muted); }
  .app-home__stats dd { margin: 0; font-variant-numeric: tabular-nums; }
  .app-home__note { font-size: var(--ez-fs-sm); color: var(--ez-color-warning); }
  .app-home__error { color: var(--ez-color-error); }
  .app-home__path { display: block; font-size: var(--ez-fs-sm); color: var(--ez-color-text-muted); word-break: break-all; }
  .app-home__actions { display: flex; flex-wrap: wrap; gap: var(--ez-space-sm); }
  .app-home__recent-title { font-size: var(--ez-fs-md); margin: 0 0 var(--ez-space-sm); }
  .app-home__recent { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--ez-space-md); }
  .app-home__recent li { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--ez-space-sm); }
  .app-home__recent-text { min-width: 0; flex: 1 1 16rem; }
  .app-home__recent-name { font-weight: 600; }
  .app-home__progress { color: var(--ez-color-text-muted); }
  .app-home__summary { margin: 0 0 var(--ez-space-sm); }
  .app-home__subtitle { font-size: var(--ez-fs-md); margin: var(--ez-space-sm) 0 var(--ez-space-2xs); }
  .app-home__drop { color: var(--ez-color-text-muted); margin: var(--ez-space-md) 0 0; }
  .app-home__toolbar { margin-bottom: var(--ez-space-xs); }
  :global(.app-home__drop--active), .app-home__drop--active { outline: var(--ez-space-2xs) solid var(--ez-color-accent-ring); outline-offset: var(--ez-space-2xs); border-radius: var(--ez-radius-md); }
  .app-home__root { font-size: var(--ez-fs-sm); color: var(--ez-color-text-muted); word-break: break-all; }
</style>
