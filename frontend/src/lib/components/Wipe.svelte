<script lang="ts">
  import { Skeleton } from '@etzianco/ui';

  import { formatNumber, t } from '../i18n';
  import { clampPercent, percentFromPointer, wipeKey } from '../wipe';

  /**
   * One frame in two states with a divider: the left of the divider shows `before`, the right `after`.
   * The divider is a slider (pointer and keyboard); Before and After are named in text, not by colour.
   * An empty URL means that side is still loading.
   */
  let {
    before,
    after,
    beforeAlt,
    afterAlt
  }: { before: string; after: string; beforeAlt: string; afterAlt: string } = $props();

  let position = $state(50);
  let frame: HTMLDivElement | undefined = $state();
  let dragging = $state(false);

  const percent = $derived(formatNumber(Math.round(position) / 100, { style: 'percent' }));
  const valueText = $derived(t('wipe-value', { before: percent, after: formatNumber(Math.round(100 - position) / 100, { style: 'percent' }) }));

  function fromPointer(e: PointerEvent): void {
    if (!frame) return;
    const r = frame.getBoundingClientRect();
    position = percentFromPointer(e.clientX, r.left, r.width);
  }
  function onpointerdown(e: PointerEvent): void {
    dragging = true;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    fromPointer(e);
  }
  function onpointermove(e: PointerEvent): void {
    if (dragging) fromPointer(e);
  }
  function onpointerup(e: PointerEvent): void {
    dragging = false;
    (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
  }
  function onkeydown(e: KeyboardEvent): void {
    if (e.metaKey || e.ctrlKey || e.altKey) return;
    const next = wipeKey(position, e.key, e.shiftKey);
    if (next === null) return;
    e.preventDefault();
    position = next;
  }
</script>

<div class="app-wipe">
  <div
    class="app-wipe__frame"
    bind:this={frame}
    {onpointerdown}
    {onpointermove}
    {onpointerup}
    onpointercancel={onpointerup}
    role="presentation"
  >
    <div class="app-wipe__layer">
      {#if after}<img src={after} alt={afterAlt} draggable="false" />{:else}<Skeleton variant="rect" size="12rem" label={t('clean-loading')} />{/if}
    </div>
    <div class="app-wipe__layer app-wipe__layer--before" style:clip-path="inset(0 {100 - clampPercent(position)}% 0 0)">
      {#if before}<img src={before} alt={beforeAlt} draggable="false" />{:else}<Skeleton variant="rect" size="12rem" label={t('clean-loading')} />{/if}
    </div>
    <span class="app-wipe__tag app-wipe__tag--before">{t('adjust-label-before')}</span>
    <span class="app-wipe__tag app-wipe__tag--after">{t('adjust-label-after')}</span>
    <div
      class="app-wipe__divider"
      style:left="{clampPercent(position)}%"
      role="slider"
      tabindex="0"
      aria-label={t('wipe-label')}
      aria-orientation="horizontal"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={Math.round(position)}
      aria-valuetext={valueText}
      {onkeydown}
    >
      <span class="app-wipe__grip" aria-hidden="true">↔</span>
    </div>
  </div>
  <p class="app-wipe__hint">{t('wipe-hint')}</p>
</div>

<style>
  .app-wipe {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-xs);
  }
  .app-wipe__frame {
    position: relative;
    display: grid;
    overflow: hidden;
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-border-soft);
    touch-action: none;
    cursor: ew-resize;
    user-select: none;
  }
  .app-wipe__layer {
    grid-area: 1 / 1;
    display: grid;
    place-items: center;
  }
  .app-wipe__layer img {
    width: 100%;
    height: auto;
    display: block;
    pointer-events: none;
  }
  .app-wipe__tag {
    position: absolute;
    top: var(--ez-space-xs);
    padding: var(--ez-space-2xs) var(--ez-space-xs);
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-surface);
    color: var(--ez-color-text);
    border: 1px solid var(--ez-color-border);
    font-size: var(--ez-fs-sm);
    pointer-events: none;
  }
  .app-wipe__tag--before {
    left: var(--ez-space-xs);
  }
  .app-wipe__tag--after {
    right: var(--ez-space-xs);
  }
  .app-wipe__divider {
    position: absolute;
    top: 0;
    bottom: 0;
    width: var(--ez-space-lg);
    transform: translateX(-50%);
    display: grid;
    place-items: center;
    cursor: ew-resize;
  }
  .app-wipe__divider::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    left: 50%;
    width: 2px;
    transform: translateX(-50%);
    background: var(--ez-color-surface);
    box-shadow: 0 0 0 1px var(--ez-color-border);
  }
  .app-wipe__grip {
    position: relative;
    display: grid;
    place-items: center;
    width: var(--ez-space-lg);
    height: var(--ez-space-lg);
    border-radius: 50%;
    background: var(--ez-color-surface);
    color: var(--ez-color-text);
    border: 1px solid var(--ez-color-border);
    font-size: var(--ez-fs-sm);
  }
  .app-wipe__divider:focus-visible {
    outline: 2px solid var(--ez-color-accent);
    outline-offset: 2px;
  }
  .app-wipe__hint {
    margin: 0;
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
  }
</style>
