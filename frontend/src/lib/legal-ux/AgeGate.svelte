<!--
  @etzianco/legal-ux — AgeGate
  Neutral age screen for apps with accounts or data collection. Asks for a
  birth date (never "are you over 13?"), says why, stores nothing but the
  result you pass on. Under `minimumAge`: blocked, and the date cannot be
  retyped in the same session. Under 14 (Québec): a parent or guardian must
  consent; the app shows its own parental-consent step on `onparental`.

  Usage:
    <AgeGate {t} minimumAge={14} onallowed={…} onparental={…} onblocked={…} />
-->
<script>
  import { Card, Input, Button } from '@etzianco/ui';
  import { createAgeGate } from './age.js';

  /** @type {{ t: (key: string, args?: Record<string, unknown>) => string, minimumAge: number, parentalConsentBelow?: number, onallowed: (age: number) => void, onparental?: (age: number) => void, onblocked?: () => void }} */
  let { t, minimumAge, parentalConsentBelow = 14, onallowed, onparental, onblocked } = $props();

  // svelte-ignore state_referenced_locally
  const gate = createAgeGate({ minimumAge, parentalConsentBelow });
  let birth = $state('');
  let error = $state('');
  let blocked = $state(false);

  function submit(e) {
    e.preventDefault();
    error = '';
    if (!birth) {
      error = t('age-error-required');
      return;
    }
    let r;
    try {
      r = gate.check(birth);
    } catch {
      error = t('age-error-invalid');
      return;
    }
    if (r.status === 'allowed') onallowed(r.age);
    else if (r.status === 'parental-consent') onparental ? onparental(r.age) : ((blocked = true), onblocked?.());
    else {
      blocked = true;
      onblocked?.();
    }
  }
</script>

<Card>
  {#if blocked}
    <div role="alert">
      <h2>{t('age-blocked-title')}</h2>
      <p>{t('age-blocked-text', { age: minimumAge })}</p>
    </div>
  {:else}
    <form onsubmit={submit} novalidate>
      <h2>{t('age-title')}</h2>
      <p>{t('age-why')}</p>
      <Input type="date" label={t('age-birth-label')} bind:value={birth} {error} />
      <Button type="submit">{t('action-continue')}</Button>
    </form>
  {/if}
</Card>
