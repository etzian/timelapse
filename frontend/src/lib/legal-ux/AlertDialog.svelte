<!--
  @etzianco/legal-ux — AlertDialog
  The irreversible-action dialog GUIDELINES §2.2 and docs/screens.md ask for:
  role="alertdialog", focus starts on Cancel (never on the destructive
  button), Escape and backdrop dismiss, background inert, focus returns to
  the trigger. @etzianco/ui's Modal can't do this yet (no `role`, no initial
  focus, English-only close label), so this composes the package's own
  utilities and `ez-modal*` classes. When Modal gains those, replace this.

  Usage:
    <AlertDialog open={open} onclose={() => open = false} title="Delete my account"
      cancelLabel="Cancel" confirmLabel="Delete my account" onconfirm={doDelete}>
      …what will be deleted…
    </AlertDialog>
-->
<script>
  import { portal, trapFocus, lockScroll, pushLayer, Button } from '@etzianco/ui';

  /** @type {{ open?: boolean, title: string, cancelLabel: string, confirmLabel: string, busy?: boolean, onclose: () => void, onconfirm: () => void, children: import('svelte').Snippet }} */
  let { open = false, title, cancelLabel, confirmLabel, busy = false, onclose, onconfirm, children } = $props();

  const uid = $props.id();
  const titleId = `${uid}-title`;
  const descId = `${uid}-desc`;
  /** @type {HTMLElement | undefined} */
  let dialogEl = $state();
  /** @type {HTMLElement | undefined} */
  let cancelWrap = $state();

  $effect(() => {
    if (!open || !dialogEl) return;
    const unlock = lockScroll();
    const untrap = trapFocus(dialogEl, { initial: cancelWrap?.querySelector('button') ?? undefined });
    const pop = pushLayer({ onDismiss: () => onclose() });
    return () => {
      pop();
      untrap();
      unlock();
    };
  });

  function backdrop(e) {
    if (e.target === e.currentTarget) onclose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="ez-modal-backdrop" use:portal onclick={backdrop}>
    <div class="ez-modal ez-modal--sm" role="alertdialog" aria-modal="true" aria-labelledby={titleId} aria-describedby={descId} bind:this={dialogEl}>
      <div class="ez-modal__header">
        <h2 class="ez-modal__title" id={titleId}>{title}</h2>
      </div>
      <div class="ez-modal__body" id={descId}>
        {@render children()}
      </div>
      <div class="ez-modal__footer">
        <span bind:this={cancelWrap}><Button variant="secondary" onclick={onclose}>{cancelLabel}</Button></span>
        <Button variant="danger" disabled={busy} onclick={onconfirm}>{confirmLabel}</Button>
      </div>
    </div>
  </div>
{/if}
