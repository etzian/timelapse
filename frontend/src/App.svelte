<script lang="ts">
  import { flushSync, tick, untrack } from 'svelte';
  import { AppShell, Sidebar, TabBar, Toast } from '@etzianco/ui';

  import NextStep from './lib/components/NextStep.svelte';
  import Adjust from './lib/pages/Adjust.svelte';
  import Cleaning from './lib/pages/Cleaning.svelte';
  import ContactSheets from './lib/pages/ContactSheets.svelte';
  import Export from './lib/pages/Export.svelte';
  import Home from './lib/pages/Home.svelte';
  import Organize from './lib/pages/Organize.svelte';
  import Sampling from './lib/pages/Sampling.svelte';
  import Settings from './lib/pages/Settings.svelte';
  import { getLocale, t } from './lib/i18n';
  import { installMenu } from './lib/menuNative';
  import type { MenuAction } from './lib/menu';
  import { NAV_ORDER, nav } from './lib/stores/nav.svelte';
  import { JOB_IDS, JOB_PAGE, STEP_IDS, NO_JOBS, endedJobs, startedJobs, stepStates } from './lib/pipeline';
  import type { Running, StepId, StepState } from './lib/pipeline';
  import { STORAGE_KEYS } from './lib/storageKeys';
  import { pipeline } from './lib/stores/pipeline.svelte';
  import { project } from './lib/stores/project.svelte';
  import type { Page } from './lib/stores/nav.svelte';

  // The page shown lives in a store, so a page can send the user to another one.
  const active = $derived(nav.active);

  // The page drawn follows `active` through a view transition where the webview has one
  // (GUIDELINES §5.4); under reduced motion, or without the API, it swaps at once.
  let shown = $state<Page>(nav.active);
  $effect(() => {
    const next = nav.active;
    if (next === untrack(() => shown)) return;
    const focus = nav.takeFocusRequest();
    const swap = () => {
      shown = next;
      flushSync();
      // A change from the menu, Next or a page's own button: the control that asked is gone,
      // so focus goes to the new page's heading (a sidebar or tab link keeps its focus).
      if (focus) focusHeading();
    };
    const calm = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches;
    if (!calm && typeof document.startViewTransition === 'function') document.startViewTransition(swap);
    else swap();
  });

  function focusHeading(): void {
    const h = document.querySelector<HTMLElement>('main .ez-page-header h1, main h1');
    if (!h) return;
    h.tabIndex = -1;
    h.focus();
  }

  // The native menu: rebuilt when the language changes so its labels follow.
  function runMenu(action: MenuAction): void {
    if (action.type === 'go') nav.go(action.page);
    else nav.requestImport();
  }
  $effect(() => {
    void getLocale();
    installMenu(runMenu).catch(() => {}); // no menu outside the desktop shell
  });

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

  // Announce a job's start and end once, in a live region that exists from the start. The
  // region is emptied first so the same sentence twice in a row is read again.
  let announcement = $state('');
  let previous: Running = { ...NO_JOBS };
  async function announce(text: string): Promise<void> {
    announcement = '';
    await tick();
    announcement = text;
  }
  $effect(() => {
    const next = { ...running };
    const ended = endedJobs(previous, next);
    const started = startedJobs(previous, next);
    previous = next;
    const lines = [
      ...started.map((j) => t('pipeline-job-started', { job: t(`pipeline-job-${j}`) })),
      ...ended.map((j) => t('pipeline-job-ended', { job: t(`pipeline-job-${j}`) }))
    ];
    if (lines.length > 0) void announce(lines.join(' '));
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

  const navItem = (id: Page) => ({ label: `${t(`nav-${id}`)}${suffix(id)}`, href: `#${id}`, onclick: () => nav.go(id, { focus: false }) });
  const groups = $derived([
    { id: 'workflow', label: t('nav-group-workflow'), items: (['home', 'cleaning', 'sampling', 'adjust', 'export'] as Page[]).map(navItem) },
    { id: 'tools', label: t('nav-group-tools'), items: (['organize', 'sheets'] as Page[]).map(navItem) },
    { id: 'app', label: t('nav-group-app'), items: [navItem('settings')] }
  ]);
  const tabs = $derived(NAV_ORDER.map((id) => ({ id, label: t(`nav-${id}`) })));
  const showNext = $derived(!!project.root && (STEP_IDS as readonly string[]).includes(shown));
  const activeHref = $derived(`#${active}`);
</script>

{#snippet topbar()}
  <!-- One fixed place for the way forward, on every workflow page. -->
  <div class="app-topbar"><NextStep step={shown as StepId} /></div>
{/snippet}

<AppShell mode="auto" header={showNext ? topbar : undefined}>
  {#snippet sidebar()}
    <Sidebar brand={t('app-name')} {groups} active={activeHref} storageKey={STORAGE_KEYS.sidebar}>
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
    <TabBar items={tabs} {active} onselect={(id: string) => nav.go(id as Page, { focus: false })} />
  {/snippet}

  {#if shown === 'home'}
    <Home />
  {:else if shown === 'cleaning'}
    <Cleaning ongohome={() => nav.go('home')} />
  {:else if shown === 'organize'}
    <Organize ongohome={() => nav.go('home')} />
  {:else if shown === 'sampling'}
    <Sampling ongohome={() => nav.go('home')} />
  {:else if shown === 'adjust'}
    <Adjust ongohome={() => nav.go('home')} />
  {:else if shown === 'export'}
    <Export ongohome={() => nav.go('home')} ongosampling={() => nav.go('sampling')} />
  {:else if shown === 'sheets'}
    <ContactSheets ongohome={() => nav.go('home')} />
  {:else}
    <Settings />
  {/if}
  <div class="app-sr-only" role="status" aria-live="polite">{announcement}</div>
</AppShell>
<Toast />
