<!--
  @etzianco/legal-ux — DangerZone
  Account deletion (SaaS; Apple requires it in-app wherever accounts exist) or
  "erase all local data" (standalone). Last section of Settings. The dialog
  names exactly what is deleted and what is kept by law, focus starts on
  Cancel, and the destructive button repeats the action. Deletion must really
  happen in `ondelete`, not hide a flag (legal-compliance checks the backend).

  Usage:
    <DangerZone kind="account" {t} deleted={[t('del-profile'), t('del-content')]}
      kept={[t('kept-invoices')]} ondelete={deleteAccount} />
-->
<script>
  import { SettingsSection, SettingsRow, Button } from '@etzianco/ui';
  import AlertDialog from './AlertDialog.svelte';

  /** @type {{ kind: 'account' | 'local-data', t: (key: string, args?: Record<string, unknown>) => string, deleted: string[], kept?: string[], ondelete: () => void | Promise<void> }} */
  let { kind, t, deleted, kept = [], ondelete } = $props();

  let open = $state(false);
  let busy = $state(false);
  let failed = $state(false);
  const k = $derived(kind === 'account' ? 'account' : 'local');

  async function confirm() {
    busy = true;
    failed = false;
    try {
      await ondelete();
      open = false;
    } catch {
      failed = true;
    } finally {
      busy = false;
    }
  }
</script>

<SettingsSection danger title={t('danger-title')} description={t(`danger-${k}-description`)}>
  <SettingsRow label={t(`danger-${k}-label`)} hint={t(`danger-${k}-hint`)}>
    <Button variant="danger" onclick={() => (open = true)}>{t(`danger-${k}-action`)}</Button>
  </SettingsRow>
</SettingsSection>

<AlertDialog {open} title={t(`danger-${k}-action`)} cancelLabel={t('action-cancel')} confirmLabel={t(`danger-${k}-action`)} {busy} onclose={() => (open = false)} onconfirm={confirm}>
  <p>{t('danger-will-delete')}</p>
  <ul>{#each deleted as d}<li>{d}</li>{/each}</ul>
  {#if kept.length}
    <p>{t('danger-will-keep')}</p>
    <ul>{#each kept as d}<li>{d}</li>{/each}</ul>
  {/if}
  <p><strong>{t('danger-irreversible')}</strong></p>
  {#if failed}<p role="alert">{t('danger-failed')}</p>{/if}
</AlertDialog>
