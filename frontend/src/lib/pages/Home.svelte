<script lang="ts">
  import { Button, Card, PageHeader } from '@etzianco/ui';

  import { formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { backfillMetadata, pickFolder, importRoot, integrityCheck } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import type { BackfillReport, ImportReport, IntegrityResult } from '../ipc/app';
  import { project } from '../stores/project.svelte';
  import TimelineSection from './TimelineSection.svelte';

  // The folder picked here is the project Cleaning works on too.
  let root = $state(project.root);
  let busy = $state(false);
  let error = $state('');
  let report = $state<ImportReport | null>(null);
  let integrity = $state<IntegrityResult | null>(null);
  let backfill = $state<BackfillReport | null>(null);
  let backfillError = $state('');
  let dating = $state(false);

  /** Phase 2: resolve capture times once the import is in. Its failure never hides the import report. */
  async function doBackfill(): Promise<void> {
    if (dating) return;
    const forRoot = root;
    dating = true;
    backfillError = '';
    try {
      const result = await backfillMetadata(forRoot);
      if (forRoot === root) backfill = result; // never show one folder's dates under another
    } catch (err) {
      if (forRoot === root) backfillError = errorText(err as IpcError);
    } finally {
      dating = false;
    }
  }

  async function doImport(): Promise<void> {
    if (busy || dating) return;
    const picked = await pickFolder();
    if (!picked) return; // cancelled — a no-op, not an error
    root = picked;
    project.root = picked;
    busy = true;
    error = '';
    report = null;
    integrity = null;
    backfill = null;
    backfillError = '';
    try {
      report = await importRoot(root);
      integrity = await integrityCheck(root);
    } catch (err) {
      report = null;
      error = errorText(err as IpcError);
    } finally {
      busy = false;
    }
    if (report) await doBackfill();
  }
</script>

<div class="app-page">
  <PageHeader title={t('home-title')} />
  <p class="app-home__tagline">{t('app-tagline')}</p>

  <Card>
    <p class="app-home__intro">{t('home-intro')}</p>
    <Button onclick={doImport} disabled={busy || dating}>{busy ? t('home-importing') : t('home-import-action')}</Button>
  </Card>

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
    <p class="app-home__intro">
      {t('timeline-backfill-result', { resolved: backfill.newly_resolved, unresolved: backfill.unresolved_this_run })}
    </p>
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
  .app-home__progress { color: var(--ez-color-text-muted); }
  .app-home__root { font-size: var(--ez-fs-sm); color: var(--ez-color-text-muted); word-break: break-all; }
</style>
