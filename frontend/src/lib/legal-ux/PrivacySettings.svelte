<!--
  @etzianco/legal-ux — PrivacySettings
  Settings › Privacy (docs/screens.md): one toggle per optional data use,
  matching the consent choices; where data is stored; export my data; link to
  the policy. Changing a toggle is saved at once with a new date.

  Usage:
    <PrivacySettings {consent} {purposes} {t} locale="fr-CA"
      storageNote={t('privacy-storage-device')} policyHref="/privacy" onexport={exportAll} />
-->
<script>
  import { SettingsSection, SettingsRow, Toggle, Button } from '@etzianco/ui';

  /** @type {{ consent: ReturnType<typeof import('./consent.js').createConsent>, purposes: import('./consent.js').Purpose[], t: (key: string, args?: Record<string, unknown>) => string, locale: string, storageNote?: string, policyHref?: string, onexport?: () => void | Promise<void> }} */
  let { consent, purposes, t, locale, storageNote = '', policyHref = '', onexport } = $props();

  /** @type {Record<string, boolean>} */
  let choices = $state({});
  /** @type {string | null} */
  let decidedAt = $state(null);
  $effect(() => {
    const sync = () => {
      choices = consent.choices();
      decidedAt = consent.decidedAt();
    };
    sync();
    return consent.subscribe(sync);
  });

  const when = $derived(decidedAt ? new Intl.DateTimeFormat(locale, { dateStyle: 'long' }).format(new Date(decidedAt)) : '');
  const optional = $derived(purposes.filter((p) => !p.necessary));
</script>

<SettingsSection title={t('privacy-settings-title')} description={decidedAt ? t('privacy-settings-decided', { date: when }) : t('privacy-settings-undecided')}>
  {#each optional as p (p.id)}
    <SettingsRow label={t(`consent-purpose-${p.id}`)} hint={t(`consent-purpose-${p.id}-desc`)}>
      <Toggle checked={choices[p.id]} label={t(`consent-purpose-${p.id}`)} onchange={(v) => consent.save({ ...choices, [p.id]: v })} />
    </SettingsRow>
  {/each}
  {#if storageNote}
    <SettingsRow label={t('privacy-storage-label')} hint={storageNote}><span></span></SettingsRow>
  {/if}
  {#if onexport}
    <SettingsRow label={t('privacy-export-label')} hint={t('privacy-export-hint')}>
      <Button variant="secondary" onclick={onexport}>{t('privacy-export-action')}</Button>
    </SettingsRow>
  {/if}
  {#if policyHref}
    <SettingsRow label={t('legal-privacy-policy')}><a href={policyHref}>{t('privacy-policy-open')}</a></SettingsRow>
  {/if}
</SettingsSection>
