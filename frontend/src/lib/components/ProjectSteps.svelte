<script lang="ts">
  import { Button, Card } from '@etzianco/ui';

  import { formatNumber, t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { classifyImages, listExports, listSamplingRuns, projectSummary } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';
  import { nav } from '../stores/nav.svelte';
  import type { Page } from '../stores/nav.svelte';

  /** One line per pipeline step with its figure; `version` changes when the project was re-read (rescan). */
  let { root, version = 0 }: { root: string; version?: number } = $props();

  interface Figures {
    photos: number;
    flagged: number;
    analyzed: boolean;
    selections: number;
    exports: number;
  }

  let figures = $state<Figures | null>(null);
  let loading = $state(false);
  let error = $state('');
  let seq = 0;

  async function load(forRoot: string): Promise<void> {
    const mine = ++seq;
    loading = true;
    error = '';
    try {
      const [summary, counts, runs, exports] = await Promise.all([
        projectSummary(forRoot),
        classifyImages(forRoot),
        listSamplingRuns(forRoot),
        listExports(forRoot)
      ]);
      if (mine !== seq) return; // a newer load (another folder) owns the state
      figures = {
        photos: summary.total_images,
        flagged: counts.flagged,
        analyzed: counts.unanalyzed < summary.total_images,
        selections: runs.length,
        exports: exports.filter((e) => e.status === 'completed').length
      };
    } catch (err) {
      if (mine !== seq) return;
      error = errorText(err as IpcError);
    } finally {
      if (mine === seq) loading = false;
    }
  }

  $effect(() => {
    void version;
    figures = null;
    void load(root);
  });

  const rows = $derived(
    figures
      ? ([
          { id: 'photos', label: t('home-step-photos'), value: formatNumber(figures.photos), page: null },
          {
            id: 'flagged',
            label: t('home-step-flagged'),
            value: figures.analyzed ? formatNumber(figures.flagged) : t('home-step-not-analyzed'),
            page: 'cleaning'
          },
          { id: 'selections', label: t('home-step-selections'), value: formatNumber(figures.selections), page: 'sampling' },
          { id: 'exports', label: t('home-step-exports'), value: formatNumber(figures.exports), page: 'export' }
        ] as { id: string; label: string; value: string; page: Page | null }[])
      : []
  );
</script>

<Card class="app-steps">
  <h2 class="app-steps__title">{t('home-steps-title')}</h2>
  {#if error}
    <div role="alert" class="app-steps__error">
      <p>{t('home-steps-error')} {error}</p>
      <Button size="sm" onclick={() => load(root)}>{t('timeline-retry')}</Button>
    </div>
  {:else if loading && !figures}
    <ul class="app-steps__list" aria-busy="true" aria-label={t('project-opening')}>
      {#each [0, 1, 2, 3] as i (i)}
        <li class="app-steps__row app-steps__row--skeleton" aria-hidden="true"><span class="app-steps__bone"></span></li>
      {/each}
    </ul>
  {:else if figures}
    <ul class="app-steps__list" class:app-steps__list--refreshing={loading}>
      {#each rows as row (row.id)}
        <li class="app-steps__row">
          <span class="app-steps__label">{row.label}</span>
          <span class="app-steps__value">{row.value}</span>
          {#if row.page}
            {@const page = row.page}
            <Button size="sm" variant="ghost" onclick={() => nav.go(page)}>{t('home-step-open')}<span class="app-sr-only"> · {row.label}</span></Button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</Card>

<style>
  .app-steps__title { font-size: var(--ez-fs-md); margin: 0 0 var(--ez-space-sm); }
  .app-steps__list { list-style: none; margin: 0; padding: 0; display: grid; gap: var(--ez-space-xs); }
  .app-steps__list--refreshing { opacity: 0.6; }
  .app-steps__row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    gap: var(--ez-space-md);
    min-height: var(--ez-space-xl);
  }
  .app-steps__label { min-width: 0; }
  .app-steps__value { font-variant-numeric: tabular-nums; font-weight: 600; text-align: end; }
  .app-steps__error { color: var(--ez-color-error); display: grid; gap: var(--ez-space-sm); justify-items: start; }
  .app-sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  .app-steps__bone {
    grid-column: 1 / -1;
    height: var(--ez-space-md);
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-surface-hover);
  }
</style>
