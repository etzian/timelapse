<script lang="ts">
  import { Button, Card, PageHeader } from '@etzianco/ui';

  import { t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { pickFolder, importRoot, integrityCheck } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import type { ImportReport, IntegrityResult } from '../ipc/app';

  let root = $state('');
  let busy = $state(false);
  let error = $state('');
  let report = $state<ImportReport | null>(null);
  let integrity = $state<IntegrityResult | null>(null);

  async function doImport(): Promise<void> {
    if (busy) return;
    const picked = await pickFolder();
    if (!picked) return; // cancelled — a no-op, not an error
    root = picked;
    busy = true;
    error = '';
    report = null;
    integrity = null;
    try {
      report = await importRoot(root);
      integrity = await integrityCheck(root);
    } catch (err) {
      report = null;
      error = errorText(err as IpcError);
    } finally {
      busy = false;
    }
  }
</script>

<div class="app-page">
  <PageHeader title={t('home-title')} />
  <p class="tagline">{t('app-tagline')}</p>

  <Card>
    <p class="intro">{t('home-intro')}</p>
    <Button onclick={doImport} disabled={busy}>{busy ? t('home-importing') : t('home-import-action')}</Button>
  </Card>

  {#if report}
    <Card class="report">
      <dl>
        <div><dt>{t('home-stat-scanned')}</dt><dd>{report.scanned_count}</dd></div>
        <div><dt>{t('home-stat-hashed')}</dt><dd>{report.hash_computed}</dd></div>
        <div><dt>{t('home-stat-new')}</dt><dd>{report.new_rows}</dd></div>
        <div><dt>{t('home-stat-unchanged')}</dt><dd>{report.unchanged}</dd></div>
        <div><dt>{t('home-stat-duplicates')}</dt><dd>{report.duplicates_found}</dd></div>
      </dl>

      {#if report.skipped_unreadable > 0}
        <p class="note">{t('home-skipped-unreadable', { count: report.skipped_unreadable })}</p>
      {/if}

      {#if integrity && (integrity.missing_files.length > 0 || integrity.duplicates.length > 0)}
        <section class="integrity">
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

  {#if error}<p role="alert" class="error">{error}</p>{/if}

  {#if root}<p class="root-note" title={root}>{t('home-root', { path: root })}</p>{/if}
</div>

<style>
  .tagline {
    color: var(--ez-color-text-muted);
    margin: 0 0 1rem;
  }
  .intro {
    margin: 0 0 1rem;
  }
  dl {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 0.25rem 1rem;
    margin: 0 0 1rem;
  }
  dt { color: var(--ez-color-text-muted); }
  dd { margin: 0; font-variant-numeric: tabular-nums; }
  .note { font-size: 0.9em; color: var(--ez-color-warning); }
  .integrity h3 { font-size: 1em; margin: 0.75rem 0 0.25rem; }
  .error { color: var(--ez-color-error); }
  .root-note { font-size: 0.85em; color: var(--ez-color-text-muted); word-break: break-all; }
</style>
