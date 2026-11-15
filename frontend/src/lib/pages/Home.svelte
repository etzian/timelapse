<script lang="ts">
  import { Button, Card, PageHeader } from '@etzianco/ui';

  import { folderName } from '../erase';
  import { formatDate, formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { backfillMetadata, pickFolder, importRoot, integrityCheck } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
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

  async function doImport(): Promise<void> {
    if (busy || dating || opening) return;
    const picked = await pickFolder();
    if (!picked) return; // cancelled — a no-op, not an error
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
  }

  async function reopen(path: string): Promise<void> {
    if (busy || dating || opening) return;
    error = '';
    backfillError = '';
    await openProject(path);
  }
</script>

<div class="app-page">
  <PageHeader title={t('home-title')} />
  <p class="app-home__tagline">{t('app-tagline')}</p>

  <Card>
    <p class="app-home__intro">{t('home-intro')}</p>
    <Button onclick={doImport} disabled={busy || dating || opening}>{busy ? t('home-importing') : t('home-import-action')}</Button>
  </Card>

  {#if project.status === 'unreachable'}
    <Card class="app-home__unreachable">
      <div role="alert">
        <p class="app-home__error"><strong>{folderName(project.unreachable)}</strong> · {t('project-unreachable')}</p>
        <p class="app-home__path" title={project.unreachable}>{project.unreachable}</p>
        <p>{t('project-unreachable-hint')}</p>
      </div>
      <div class="app-home__actions">
        <Button onclick={() => reopen(project.unreachable)}>{t('project-retry')}</Button>
        <Button variant="ghost" onclick={() => forgetProject(project.unreachable)}>{t('project-remove')}</Button>
      </div>
    </Card>
  {:else if opening}
    <p class="app-home__progress" aria-live="polite">{t('project-opening')}</p>
  {/if}

  {#if others.length > 0}
    <Card>
      <h2 class="app-home__recent-title">{t('project-recent')}</h2>
      <ul class="app-home__recent">
        {#each others as r (r.path)}
          <li>
            <div class="app-home__recent-text">
              <span class="app-home__recent-name">{folderName(r.path)}</span>
              <span class="app-home__path" title={r.path}>{r.path}</span>
              <span class="app-home__path">{t('project-last-opened', { date: formatDate(r.openedAt, { dateStyle: 'medium', timeStyle: 'short' }) })}</span>
              {#if project.status === 'unreachable' && project.unreachable === r.path}
                <span class="app-home__error">{t('project-unreachable')}</span>
              {/if}
            </div>
            <div class="app-home__actions">
              <Button size="sm" variant="secondary" onclick={() => reopen(r.path)} disabled={busy || dating || opening}>{project.unreachable === r.path ? t('project-retry') : t('project-open')}</Button>
              <Button size="sm" variant="ghost" onclick={() => forgetProject(r.path)}>{t('project-remove')}</Button>
            </div>
          </li>
        {/each}
      </ul>
    </Card>
  {/if}

  {#if report}
    <Card class="app-home__report">
      <dl class="app-home__stats">
        <div><dt>{t('home-stat-scanned')}</dt><dd>{formatNumber(report.scanned_count)}</dd></div>
        <div><dt>{t('home-stat-hashed')}</dt><dd>{formatNumber(report.hash_computed)}</dd></div>
        <div><dt>{t('home-stat-new')}</dt><dd>{formatNumber(report.new_rows)}</dd></div>
        <div><dt>{t('home-stat-unchanged')}</dt><dd>{formatNumber(report.unchanged)}</dd></div>
        <div><dt>{t('home-stat-duplicates')}</dt><dd>{formatNumber(report.duplicates_found)}</dd></div>
      </dl>

      {#if report.hidden_removed > 0}
        <p class="app-home__note">{t('home-hidden-removed', { count: report.hidden_removed })}</p>
      {/if}

      {#if report.skipped_unreadable > 0}
        <p class="app-home__note">{t('home-skipped-unreadable', { count: report.skipped_unreadable })}</p>
      {/if}

      {#if integrity && (integrity.missing_files.length > 0 || integrity.duplicates.length > 0)}
        <section class="app-home__integrity">
          {#if integrity.missing_files.length > 0}
            <h3>{t('home-integrity-missing', { count: integrity.missing_files.length })}</h3>
            <ul>
              {#each integrity.missing_files as p}
                <li>{p}</li>
              {/each}
            </ul>
          {/if}
          {#if integrity.duplicates.length > 0}
            <h3>{t('home-integrity-dups', { count: integrity.duplicates.length })}</h3>
            <ul>
              {#each integrity.duplicates as group, i}
                <li>{group.join(', ')}</li>
              {/each}
            </ul>
          {/if}
        </section>
      {/if}

      {#if report.duplicate_groups.length > 0}
        <details>
          <summary>{t('home-duplicate-groups-summary')}</summary>
          <ul>
            {#each report.duplicate_groups as group, i}
              <li>{group.join(', ')}</li>
            {/each}
          </ul>
        </details>
      {/if}
    </Card>
  {/if}

  {#if dating}
    <p class="app-home__progress" aria-live="polite">{t('timeline-dating')}</p>
  {:else if backfillError}
    <div role="alert" class="app-home__error">
      <p>{backfillError}</p>
      <Button onclick={doBackfill}>{t('timeline-retry')}</Button>
    </div>
  {:else if backfill}
    {#if backfill.newly_resolved + backfill.unresolved_this_run > 0}
      <p class="app-home__intro">
        {t('timeline-backfill-result', { resolved: backfill.newly_resolved, unresolved: backfill.unresolved_this_run })}
      </p>
    {/if}
    <TimelineSection {root} {backfill} />
  {/if}

  {#if error}<p role="alert" class="app-home__error">{error}</p>{/if}

  {#if root}<p class="app-home__root" title={root}>{t('home-root', { path: root })}</p>{/if}
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
  .app-home__integrity h3 { font-size: var(--ez-fs-md); margin: var(--ez-space-sm) 0 var(--ez-space-2xs); }
  .app-home__error { color: var(--ez-color-error); }
  .app-home__path { display: block; font-size: var(--ez-fs-sm); color: var(--ez-color-text-muted); word-break: break-all; }
  .app-home__actions { display: flex; flex-wrap: wrap; gap: var(--ez-space-sm); }
  .app-home__recent-title { font-size: var(--ez-fs-md); margin: 0 0 var(--ez-space-sm); }
  .app-home__recent { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--ez-space-md); }
  .app-home__recent li { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--ez-space-sm); }
  .app-home__recent-text { min-width: 0; flex: 1 1 16rem; }
  .app-home__recent-name { font-weight: 600; }
  .app-home__progress { color: var(--ez-color-text-muted); }
  .app-home__root { font-size: var(--ez-fs-sm); color: var(--ez-color-text-muted); word-break: break-all; }
</style>
