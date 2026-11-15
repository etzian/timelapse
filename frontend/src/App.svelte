<script lang="ts">
  import { AppShell, Sidebar, TabBar } from '@etzianco/ui';

  import Adjust from './lib/pages/Adjust.svelte';
  import Cleaning from './lib/pages/Cleaning.svelte';
  import ContactSheets from './lib/pages/ContactSheets.svelte';
  import Export from './lib/pages/Export.svelte';
  import Home from './lib/pages/Home.svelte';
  import Organize from './lib/pages/Organize.svelte';
  import Sampling from './lib/pages/Sampling.svelte';
  import Settings from './lib/pages/Settings.svelte';
  import { t } from './lib/i18n';
  import { NAV_ORDER, nav } from './lib/stores/nav.svelte';
  import { JOB_IDS, JOB_PAGE, NO_JOBS, endedJobs, stepStates } from './lib/pipeline';
  import type { Running, StepId, StepState } from './lib/pipeline';
  import { pipeline } from './lib/stores/pipeline.svelte';
  import { project } from './lib/stores/project.svelte';
  import type { Page } from './lib/stores/nav.svelte';

  // The page shown lives in a store, so a page can send the user to another one.
  const active = $derived(nav.active);

  const GLYPH_RUNNING = '↻';
  const STATE_GLYPH: Record<StepState, string> = { 'not-started': '○', 'in-progress': '◐', done: '✓' };

  // Refresh the step figures when the project, the page or a running job changes.
  const running = $derived(pipeline.running);
  const runningKey = $derived(JOB_IDS.map((j) => (running[j] ? '1' : '0')).join(''));
  $effect(() => {
    void active;
    void runningKey;
    void pipeline.refresh(project.root);
  });

  // Announce a job's end once, in a live region that exists from the start.
  let announcement = $state('');
  let previous: Running = { ...NO_JOBS };
  $effect(() => {
    const next = { ...running };
    const ended = endedJobs(previous, next);
    previous = next;
    if (ended.length > 0) {
      announcement = ended.map((j) => t('pipeline-job-ended', { job: t(`pipeline-job-${j}`) })).join(' ');
    }
  });

  const steps = $derived(stepStates(pipeline.figures, running));
  const pageJob = (page: Page) => JOB_IDS.find((j) => JOB_PAGE[j] === page);

  // State by glyph and word, never by colour alone; a running job replaces the state.
  function suffix(id: Page): string {
    const job = pageJob(id);
    if (job && running[job]) return ` · ${GLYPH_RUNNING} ${t('pipeline-state-running')}`;
    if (!project.root || !(id in steps)) return '';
    const s = steps[id as StepId];
    return ` · ${STATE_GLYPH[s]} ${t(`pipeline-state-${s}`)}`;
  }

  const navItem = (id: Page) => ({ label: `${t(`nav-${id}`)}${suffix(id)}`, href: `#${id}`, onclick: () => nav.go(id) });
  const groups = $derived([
    { id: 'workflow', label: t('nav-group-workflow'), items: (['home', 'cleaning', 'sampling', 'adjust', 'export'] as Page[]).map(navItem) },
    { id: 'tools', label: t('nav-group-tools'), items: (['organize', 'sheets'] as Page[]).map(navItem) },
    { id: 'app', label: t('nav-group-app'), items: [navItem('settings')] }
  ]);
  const tabs = $derived(NAV_ORDER.map((id) => ({ id, label: t(`nav-${id}`) })));
  const activeHref = $derived(`#${active}`);
</script>

<AppShell mode="auto">
  {#snippet sidebar()}
    <Sidebar brand={t('app-name')} {groups} active={activeHref} storageKey="timelapse:sidebar">
      {#snippet footer()}
        <div class="app-sidebar-footer">
          <svg class="app-sidebar-footer__mark" viewBox="0 0 200 200" aria-hidden="true">
            <defs>
              <clipPath id="app-sun-clip"><circle cx="100" cy="100" r="70" /></clipPath>
            </defs>
            <circle class="app-sidebar-footer__sun" cx="100" cy="100" r="70" />
            <g clip-path="url(#app-sun-clip)">
              <rect class="app-sidebar-footer__stripe" x="30" y="70" width="140" height="16" />
              <rect class="app-sidebar-footer__stripe" x="30" y="98" width="140" height="14" />
              <rect class="app-sidebar-footer__stripe" x="30" y="122" width="140" height="12" />
            </g>
          </svg>
          <span>{t('app-endorsement')}</span>
        </div>
      {/snippet}
    </Sidebar>
  {/snippet}
  {#snippet tabbar()}
    <TabBar items={tabs} {active} onselect={(id: string) => nav.go(id as Page)} />
  {/snippet}

  {#if active === 'home'}
    <Home />
  {:else if active === 'cleaning'}
    <Cleaning ongohome={() => nav.go('home')} />
  {:else if active === 'organize'}
    <Organize ongohome={() => nav.go('home')} />
  {:else if active === 'sampling'}
    <Sampling ongohome={() => nav.go('home')} />
  {:else if active === 'adjust'}
    <Adjust ongohome={() => nav.go('home')} />
  {:else if active === 'export'}
    <Export ongohome={() => nav.go('home')} ongosampling={() => nav.go('sampling')} />
  {:else if active === 'sheets'}
    <ContactSheets ongohome={() => nav.go('home')} />
  {:else}
    <Settings />
  {/if}
  <div class="app-sr-only" role="status" aria-live="polite">{announcement}</div>
</AppShell>
