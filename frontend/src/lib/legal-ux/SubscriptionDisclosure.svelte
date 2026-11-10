<!--
  @etzianco/legal-ux — SubscriptionDisclosure
  The block that sits next to the buy button (Consumer Protection Act distance
  contracts; Apple guideline 3.1.2): price, billing period, renewal, trial
  conversion, how to cancel. Fields come in already formatted by the caller
  (use Intl for money); price, period, renewal and cancel are required, and a
  missing one is logged in development so the paywall can't ship half-disclosed.

  Usage:
    <SubscriptionDisclosure {t} price="4,99 $ CA / an (taxes en sus)" period={…} renewal={…} cancel={…} />
-->
<script>
  import { Card } from '@etzianco/ui';
  import { missingDisclosures } from './subscription.js';

  /** @type {{ t: (key: string, args?: Record<string, unknown>) => string, price: string, period: string, renewal: string, cancel: string, trial?: string, taxes?: string }} */
  let { t, price, period, renewal, cancel, trial = '', taxes = '' } = $props();

  const missing = $derived(missingDisclosures({ price, period, renewal, cancel }));
  $effect(() => {
    if (missing.length && import.meta.env?.DEV) console.error(`SubscriptionDisclosure: missing ${missing.join(', ')}`);
  });
</script>

<Card>
  <dl class="legal-disclosure">
    <dt>{t('sub-price')}</dt><dd>{price}{#if taxes} <small>{taxes}</small>{/if}</dd>
    <dt>{t('sub-period')}</dt><dd>{period}</dd>
    {#if trial}<dt>{t('sub-trial')}</dt><dd>{trial}</dd>{/if}
    <dt>{t('sub-renewal')}</dt><dd>{renewal}</dd>
    <dt>{t('sub-cancel')}</dt><dd>{cancel}</dd>
  </dl>
</Card>
