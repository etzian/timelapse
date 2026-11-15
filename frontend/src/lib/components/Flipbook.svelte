<script lang="ts">
  import { untrack } from 'svelte';
  import { Button, EmptyState } from '@etzianco/ui';

  import { limited } from '../clean';
  import { advance, clampIndex, frameIntervalMs, planLoad, wrapIndex } from '../flipbook';
  import type { FrameState } from '../flipbook';
  import { formatNumber, t } from '../i18n';
  import { getThumbnail } from '../ipc/app';
  import { formatInstant } from '../timeline';

  interface Frame {
    image_id: number;
    epoch_ms: number;
  }

  /**
   * Plays the 256 px thumbnails of `frames` at `fps` x `speed`. No video is decoded (ADR-027).
   * It never starts by itself, so `prefers-reduced-motion` is respected by construction.
   * `resetKey` changes when the sequence is a different one (not when it only grows).
   */
  let {
    root,
    frames,
    fps,
    speed = 1,
    total = frames.length,
    resetKey = '',
    loadFrame
  }: {
    root: string;
    frames: Frame[];
    fps: number;
    speed?: number;
    total?: number;
    resetKey?: string;
    /** Makes frame `i` of `frames` as an image URL; the default is the cached thumbnail. */
    loadFrame?: (i: number) => Promise<string>;
  } = $props();

  type Cached = { state: FrameState; url?: string };
  const CONCURRENCY = 4;

  let index = $state(0);
  let playing = $state(false);
  let buffering = $state(false);
  let cache = $state<Record<number, Cached>>({});
  let token = 0;

  const count = $derived(frames.length);
  const interval = $derived(frameIntervalMs(fps, speed));
  const current = $derived(count > 0 ? cache[clampIndex(index, count)] : undefined);
  const frame = $derived(count > 0 ? frames[clampIndex(index, count)] : undefined);
  const positionText = $derived(t('flipbook-frame', { n: formatNumber(index + 1), total: formatNumber(count) }));

  // A different sequence: start over.
  $effect(() => {
    void resetKey;
    void root;
    untrack(() => {
      token++;
      cache = {};
      index = 0;
      playing = false;
      buffering = false;
    });
  });

  // Keep a window of frames loaded around the playhead, one batch at a time.
  $effect(() => {
    const here = clampIndex(index, count);
    const keys = Object.keys(cache).map(Number);
    const inFlight = keys.some((k) => cache[k].state === 'loading');
    const plan = planLoad(here, count, keys);
    if (plan.evict.length > 0) {
      for (const k of plan.evict) delete cache[k];
      return;
    }
    if (inFlight || plan.load.length === 0) return;
    const mine = token;
    for (const i of plan.load) cache[i] = { state: 'loading' };
    void limited(
      plan.load.map((i) => async () => {
        let result: Cached;
        try {
          result = { state: 'ready', url: await (loadFrame ? loadFrame(i) : getThumbnail(root, frames[i].image_id, 256)) };
        } catch {
          result = { state: 'failed' };
        }
        // Dropped meanwhile (evicted or another sequence): discard.
        if (mine === token && cache[i]?.state === 'loading') cache[i] = result;
      }),
      CONCURRENCY
    );
  });

  // Playback: waits for a frame that is still loading rather than skipping it.
  $effect(() => {
    if (!playing) {
      buffering = false;
      return;
    }
    let timer: ReturnType<typeof setTimeout>;
    const tick = () => {
      const r = advance(index, count, (i) => cache[i]?.state);
      index = r.index;
      buffering = r.waiting;
      timer = setTimeout(tick, r.waiting ? 80 : interval);
    };
    timer = setTimeout(tick, interval);
    return () => clearTimeout(timer);
  });

  // A hidden window (another app in front, minimized) pauses playback; it never resumes
  // on its own.
  $effect(() => {
    const onVisibility = () => {
      if (document.hidden) playing = false;
    };
    document.addEventListener('visibilitychange', onVisibility);
    return () => document.removeEventListener('visibilitychange', onVisibility);
  });

  function go(i: number): void {
    index = clampIndex(i, count);
  }
  function step(delta: number): void {
    go(wrapIndex(index, delta, count));
  }
  function toggle(): void {
    if (count > 1) playing = !playing;
  }

  function onkeydown(e: KeyboardEvent): void {
    if (count === 0 || e.metaKey || e.ctrlKey || e.altKey) return;
    const target = e.target as HTMLElement;
    const onButton = target.closest('button') !== null;
    const onRange = target instanceof HTMLInputElement && target.type === 'range';
    if (e.key === ' ' || e.code === 'Space') {
      if (onButton) return; // the button's own press handles it
      e.preventDefault();
      toggle();
      return;
    }
    if (onRange) return; // the range already steps and jumps natively
    if (e.key === 'ArrowLeft') step(-1);
    else if (e.key === 'ArrowRight') step(1);
    else if (e.key === 'Home') go(0);
    else if (e.key === 'End') go(count - 1);
    else return;
    e.preventDefault();
  }
</script>

<!-- Keys bubble up from the focusable controls inside; the section itself is not a tab stop. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section class="app-flipbook" aria-label={t('flipbook-title')} {onkeydown}>
  {#if count === 0}
    <EmptyState title={t('flipbook-empty-title')} description={t('flipbook-empty-description')} compact />
  {:else}
    <div class="app-flipbook__screen" aria-busy={current?.state === 'loading'}>
      {#if current?.state === 'ready' && current.url}
        <img src={current.url} alt={positionText} />
      {:else if current?.state === 'failed'}
        <p class="app-flipbook__placeholder">{t('flipbook-failed')}</p>
      {:else}
        <p class="app-flipbook__placeholder app-flipbook__placeholder--wait">{t('flipbook-buffering')}</p>
      {/if}
    </div>

    <div class="app-flipbook__info">
      <span class="app-flipbook__frame">{positionText}</span>
      <span class="app-flipbook__when">{frame ? formatInstant(frame.epoch_ms) : ''}</span>
      <span class="app-flipbook__rate">{t('flipbook-rate', { fps: formatNumber(fps * speed) })}</span>
      <span role="status" class="app-flipbook__buffering">{buffering ? t('flipbook-buffering') : ''}</span>
    </div>

    <input
      class="app-flipbook__scrub"
      type="range"
      min="0"
      max={count - 1}
      step="1"
      value={index}
      aria-label={t('flipbook-scrub')}
      aria-valuetext={`${positionText}, ${frame ? formatInstant(frame.epoch_ms) : ''}`}
      oninput={(e) => go(Number(e.currentTarget.value))}
    />

    <div class="app-flipbook__controls">
      <Button size="sm" variant="secondary" onclick={() => go(0)} disabled={count < 2}>{t('flipbook-first')}</Button>
      <Button size="sm" variant="secondary" onclick={() => step(-1)} disabled={count < 2}>{t('flipbook-prev')}</Button>
      <Button size="sm" onclick={toggle} disabled={count < 2}>{playing ? t('flipbook-pause') : t('flipbook-play')}</Button>
      <Button size="sm" variant="secondary" onclick={() => step(1)} disabled={count < 2}>{t('flipbook-next')}</Button>
      <Button size="sm" variant="secondary" onclick={() => go(count - 1)} disabled={count < 2}>{t('flipbook-last')}</Button>
    </div>
    {#if count < total}
      <p class="app-flipbook__note">{t('flipbook-partial', { shown: formatNumber(count), total: formatNumber(total) })}</p>
    {/if}
  {/if}
</section>

<style>
  .app-flipbook {
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-sm);
    margin: var(--ez-space-sm) 0;
  }
  .app-flipbook__screen {
    display: grid;
    place-items: center;
    aspect-ratio: 3 / 2;
    max-height: 60vh;
    width: 100%;
    background: var(--ez-color-border-soft);
    border-radius: var(--ez-radius-sm);
    overflow: hidden;
  }
  .app-flipbook__screen img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .app-flipbook__placeholder {
    margin: 0;
    padding: var(--ez-space-md);
    text-align: center;
    color: var(--ez-color-text-muted);
    font-size: var(--ez-fs-sm);
  }
  .app-flipbook__info {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-xs) var(--ez-space-md);
    font-size: var(--ez-fs-sm);
    font-variant-numeric: tabular-nums;
  }
  .app-flipbook__when,
  .app-flipbook__rate,
  .app-flipbook__buffering,
  .app-flipbook__note {
    color: var(--ez-color-text-muted);
  }
  .app-flipbook__note {
    margin: 0;
    font-size: var(--ez-fs-sm);
  }
  .app-flipbook__scrub {
    width: 100%;
    accent-color: var(--ez-color-accent);
  }
  .app-flipbook__scrub:focus-visible {
    outline: var(--ez-focus-ring);
    outline-offset: var(--ez-focus-ring-offset);
  }
  .app-flipbook__controls {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-xs);
  }
</style>
