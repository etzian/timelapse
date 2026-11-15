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
  import { PAGES, nav } from './lib/stores/nav.svelte';
  import type { Page } from './lib/stores/nav.svelte';

  // The page shown lives in a store, so a page can send the user to another one.
  const active = $derived(nav.active);

  // A `$derived` so the labels re-render through t() when the language changes.
  const items = $derived(
    PAGES.map((id) => ({ id, label: t(`nav-${id}`), href: `#${id}`, onclick: () => nav.go(id) }))
  );
  const activeHref = $derived(`#${active}`);
</script>

<AppShell mode="auto">
  {#snippet sidebar()}
    <Sidebar brand={t('app-name')} items={items} active={activeHref}>
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
    <TabBar items={items.map((i) => ({ id: i.id, label: i.label }))} {active} onselect={(id: string) => nav.go(id as Page)} />
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
</AppShell>
