<script lang="ts">
  import { AppShell, Sidebar, TabBar } from '@etzianco/ui';

  import Home from './lib/pages/Home.svelte';
  import Settings from './lib/pages/Settings.svelte';
  import { t } from './lib/i18n';

  type Page = 'home' | 'settings';
  let active = $state<Page>('home');

  // A `$derived` so the labels re-render through t() when the language changes.
  const items = $derived(
    (['home', 'settings'] as const).map((id) => ({ id, label: t(`nav-${id}`), href: `#${id}`, onclick: () => (active = id) }))
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
  {:else}
    <Settings />
  {/if}
</AppShell>
