<script lang="ts">
  import { AppShell, Sidebar, TabBar } from '@etzianco/ui';

  import Cleaning from './lib/pages/Cleaning.svelte';
  import Home from './lib/pages/Home.svelte';
  import Sampling from './lib/pages/Sampling.svelte';
  import Settings from './lib/pages/Settings.svelte';
  import { t } from './lib/i18n';

  type Page = 'home' | 'cleaning' | 'sampling' | 'settings';
  let active = $state<Page>('home');

  // A `$derived` so the labels re-render through t() when the language changes.
  const items = $derived(
    (['home', 'cleaning', 'sampling', 'settings'] as const).map((id) => ({ id, label: t(`nav-${id}`), href: `#${id}`, onclick: () => (active = id) }))
  );
  const activeHref = $derived(`#${active}`);
</script>

<AppShell mode="auto">
  {#snippet sidebar()}
    <Sidebar brand={t('app-name')} items={items} active={activeHref} />
  {/snippet}
  {#snippet tabbar()}
    <TabBar items={items.map((i) => ({ id: i.id, label: i.label }))} {active} onselect={(id: string) => (active = id as Page)} />
  {/snippet}

  {#if active === 'home'}
    <Home />
  {:else if active === 'cleaning'}
    <Cleaning ongohome={() => (active = 'home')} />
  {:else if active === 'sampling'}
    <Sampling ongohome={() => (active = 'home')} />
  {:else}
    <Settings />
  {/if}
</AppShell>
