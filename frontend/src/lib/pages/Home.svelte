<script lang="ts">
  import { Button, Card, Input, PageHeader } from '@etzianco/ui';

  import { t } from '../i18n';
  import { errorText } from '../i18n/errors';
  import { greet } from '../ipc/app';
  import type { IpcError } from '../ipc/invoke';

  // The sample feature that proves the bridge end to end. Replace this page.
  let name = $state('');
  let greeting = $state('');
  let error = $state('');

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    error = '';
    try {
      const g = await greet(name);
      greeting = t('home-greeting', { name: g.name });
    } catch (err) {
      greeting = '';
      error = errorText(err as IpcError);
    }
  }
</script>

<div class="app-page">
  <PageHeader title={t('home-title')} />
  <Card>
    <form onsubmit={submit} novalidate>
      <Input label={t('home-name-label')} bind:value={name} {error} />
      <Button type="submit">{t('home-greet-action')}</Button>
    </form>
    {#if greeting}<p role="status">{greeting}</p>{/if}
  </Card>
</div>
