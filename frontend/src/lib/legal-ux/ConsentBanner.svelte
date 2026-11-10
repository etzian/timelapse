<!--
  @etzianco/legal-ux — ConsentBanner
  Cookie / optional-data consent (docs/screens.md "Consent"). Nothing optional
  may run before a choice, so gate every optional feature on
  `consent.allowed('<purpose>')`. Accept all and Reject all share one button
  style on purpose (Law 25: refusing is as easy as accepting); "Choose" opens
  a dialog with one toggle per purpose, all off.

  Usage:
    const consent = createConsent({ purposes, version: '2026-10-17', storage });
    <ConsentBanner {consent} {purposes} {t} policyHref="/privacy" />

  `t(key)` is the app's Fluent lookup; keys are in
  _foundation/i18n/common/*/legal-ux.ftl. Purposes: { id, necessary? } with
  `consent-purpose-<id>` and `consent-purpose-<id>-desc` messages.
-->
<script>
  import { Banner, Button, Modal, Toggle } from '@etzianco/ui';

  /** @type {{ consent: ReturnType<typeof import('./consent.js').createConsent>, purposes: import('./consent.js').Purpose[], t: (key: string, args?: Record<string, unknown>) => string, policyHref?: string }} */
  let { consent, purposes, t, policyHref = '' } = $props();

  let prompt = $state(false);
  let choosing = $state(false);
  /** Choices being edited in the dialog; a purpose not in here is off. */
  /** @type {Record<string, boolean>} */
  let draft = $state({});

  $effect(() => {
    prompt = consent.needsPrompt();
    return consent.subscribe(() => (prompt = consent.needsPrompt()));
  });

  function save() {
    consent.save(draft);
    choosing = false;
  }
</script>

{#if prompt}
  <Banner placement="fixed" tone="neutral" title={t('consent-banner-title')}>
    <p class="ez-banner__text">
      {t('consent-banner-text')}
      {#if policyHref}<a href={policyHref}>{t('consent-banner-policy')}</a>{/if}
    </p>
    {#snippet actions()}
      <Button variant="secondary" onclick={() => consent.rejectAll()}>{t('consent-reject-all')}</Button>
      <Button variant="secondary" onclick={() => consent.acceptAll()}>{t('consent-accept-all')}</Button>
      <Button variant="ghost" onclick={() => (choosing = true)}>{t('consent-choose')}</Button>
    {/snippet}
  </Banner>
{/if}

<Modal open={choosing} title={t('consent-dialog-title')} onclose={() => (choosing = false)}>
  <p>{t('consent-dialog-text')}</p>
  <ul>
    {#each purposes as p (p.id)}
      <li>
        {#if p.necessary}
          <Toggle checked={true} disabled label={t(`consent-purpose-${p.id}`)} />
          <p>{t(`consent-purpose-${p.id}-desc`)} <em>{t('consent-always-on')}</em></p>
        {:else}
          <Toggle bind:checked={draft[p.id]} label={t(`consent-purpose-${p.id}`)} />
          <p>{t(`consent-purpose-${p.id}-desc`)}</p>
        {/if}
      </li>
    {/each}
  </ul>
  {#snippet footer()}
    <Button variant="secondary" onclick={() => {
        consent.rejectAll();
        choosing = false;
      }}>{t('consent-reject-all')}</Button>
    <Button variant="secondary" onclick={save}>{t('consent-save')}</Button>
  {/snippet}
</Modal>
