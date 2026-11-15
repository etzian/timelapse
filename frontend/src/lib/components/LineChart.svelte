<script lang="ts">
  /**
   * A small line chart (dataviz skill): one y-axis, thin lines, a legend plus direct
   * labels at the line ends, a hover/keyboard crosshair with a tooltip, and a table view.
   * Colours come from --app-chart-* tokens; series are told apart by colour *and* by
   * width/dash, so colour is never the only cue. All text is passed in translated.
   */
  export interface Series {
    key: string;
    label: string;
    /** CSS colour, normally `var(--app-chart-…)`. */
    color: string;
    dashed?: boolean;
    /** Emphasised (the "after" line): drawn last and thicker. */
    strong?: boolean;
    values: (number | null)[];
  }

  let {
    title,
    xs,
    series,
    formatX,
    formatY,
    tableLabel,
    xLabel
  }: {
    title: string;
    /** One x per point (a frame number), ascending. */
    xs: number[];
    series: Series[];
    formatX: (x: number) => string;
    formatY: (y: number) => string;
    tableLabel: string;
    xLabel: string;
  } = $props();

  const HEIGHT = 200;
  const PAD = { top: 12, right: 72, bottom: 24, left: 52 };

  let width = $state(600);
  let hover = $state<number | null>(null);

  const plotW = $derived(Math.max(40, width - PAD.left - PAD.right));
  const plotH = HEIGHT - PAD.top - PAD.bottom;

  const domain = $derived.by(() => {
    let lo = Infinity;
    let hi = -Infinity;
    for (const s of series) for (const v of s.values) if (v !== null && Number.isFinite(v)) (lo = Math.min(lo, v)), (hi = Math.max(hi, v));
    if (!Number.isFinite(lo)) return { lo: 0, hi: 1 };
    if (hi - lo < 1e-9) return { lo: lo - 0.5, hi: hi + 0.5 };
    const pad = (hi - lo) * 0.08;
    return { lo: lo - pad, hi: hi + pad };
  });

  /** About four round ticks across the domain. */
  const ticks = $derived.by(() => {
    const span = domain.hi - domain.lo;
    const raw = span / 4;
    const mag = 10 ** Math.floor(Math.log10(raw));
    const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? raw;
    const out: number[] = [];
    for (let v = Math.ceil(domain.lo / step) * step; v <= domain.hi + 1e-12; v += step) out.push(Math.abs(v) < step * 1e-9 ? 0 : v);
    return out;
  });

  const x0 = $derived(xs[0] ?? 0);
  const x1 = $derived(xs[xs.length - 1] ?? 1);
  const px = (x: number) => PAD.left + (x1 === x0 ? plotW / 2 : ((x - x0) / (x1 - x0)) * plotW);
  const py = (y: number) => PAD.top + (1 - (y - domain.lo) / (domain.hi - domain.lo)) * plotH;

  /** The SVG path of a series; a gap where a value is missing. */
  function path(values: (number | null)[]): string {
    let d = '';
    let pen = false;
    values.forEach((v, i) => {
      if (v === null || !Number.isFinite(v)) {
        pen = false;
        return;
      }
      d += `${pen ? 'L' : 'M'}${px(xs[i]).toFixed(1)},${py(v).toFixed(1)}`;
      pen = true;
    });
    return d;
  }

  /** The last value of a series, for its direct label. */
  function last(values: (number | null)[]): { i: number; v: number } | null {
    for (let i = values.length - 1; i >= 0; i -= 1) {
      const v = values[i];
      if (v !== null && Number.isFinite(v)) return { i, v };
    }
    return null;
  }

  /** Direct labels at the right end, nudged apart so they never collide. */
  const endLabels = $derived.by(() => {
    const items = series
      .map((s) => ({ s, end: last(s.values) }))
      .filter((e): e is { s: Series; end: { i: number; v: number } } => e.end !== null)
      .map((e) => ({ key: e.s.key, label: e.s.label, y: py(e.end.v) }))
      .sort((a, b) => a.y - b.y);
    for (let k = 1; k < items.length; k += 1) items[k].y = Math.max(items[k].y, items[k - 1].y + 13);
    return items;
  });

  function nearest(clientX: number, rect: DOMRect): number {
    const x = clientX - rect.left - PAD.left;
    const target = x0 + (x / plotW) * (x1 - x0);
    let best = 0;
    for (let i = 1; i < xs.length; i += 1) if (Math.abs(xs[i] - target) < Math.abs(xs[best] - target)) best = i;
    return best;
  }

  function onmove(e: PointerEvent): void {
    hover = nearest(e.clientX, (e.currentTarget as SVGElement).getBoundingClientRect());
  }

  function onkey(e: KeyboardEvent): void {
    if (xs.length === 0) return;
    const step = e.shiftKey ? 10 : 1;
    if (e.key === 'ArrowRight') hover = Math.min(xs.length - 1, (hover ?? -1) + step);
    else if (e.key === 'ArrowLeft') hover = Math.max(0, (hover ?? xs.length) - step);
    else if (e.key === 'Home') hover = 0;
    else if (e.key === 'End') hover = xs.length - 1;
    else if (e.key === 'Escape') hover = null;
    else return;
    e.preventDefault();
  }

  const tipLeft = $derived(hover === null ? 0 : px(xs[hover]));
  /** What a screen reader announces as the cursor moves: the frame and every value. */
  const valueText = $derived(
    hover === null
      ? title
      : [formatX(xs[hover]), ...series.map((s) => `${s.label}: ${s.values[hover!] === null ? '—' : formatY(s.values[hover!] as number)}`)].join(', ')
  );
</script>

<figure class="app-chart">
  <figcaption class="app-chart__title">{title}</figcaption>
  <ul class="app-chart__legend">
    {#each series as s (s.key)}
      <li>
        <svg width="22" height="10" aria-hidden="true">
          <line x1="1" y1="5" x2="21" y2="5" stroke={s.color} stroke-width={s.strong ? 2.5 : 1.5} stroke-dasharray={s.dashed ? '4 3' : undefined} stroke-linecap="round" />
        </svg>
        {s.label}
      </li>
    {/each}
  </ul>
  <div class="app-chart__plot" bind:clientWidth={width}>
    <svg
      width={width}
      height={HEIGHT}
      role="slider"
      aria-label={title}
      aria-valuemin={xs[0] ?? 0}
      aria-valuemax={xs[xs.length - 1] ?? 0}
      aria-valuenow={hover === null ? undefined : xs[hover]}
      aria-valuetext={valueText}
      tabindex="0"
      onpointermove={onmove}
      onpointerleave={() => (hover = null)}
      onkeydown={onkey}
      onblur={() => (hover = null)}
    >
      {#each ticks as t (t)}
        <line class="app-chart__grid" x1={PAD.left} x2={PAD.left + plotW} y1={py(t)} y2={py(t)} />
        <text class="app-chart__tick" x={PAD.left - 6} y={py(t)} text-anchor="end" dominant-baseline="middle">{formatY(t)}</text>
      {/each}
      <text class="app-chart__tick" x={PAD.left} y={HEIGHT - 6}>{formatX(x0)}</text>
      <text class="app-chart__tick" x={PAD.left + plotW} y={HEIGHT - 6} text-anchor="end">{formatX(x1)}</text>
      <text class="app-chart__tick" x={PAD.left + plotW / 2} y={HEIGHT - 6} text-anchor="middle">{xLabel}</text>
      {#each series.filter((s) => !s.strong) as s (s.key)}
        <path d={path(s.values)} fill="none" stroke={s.color} stroke-width="1.5" stroke-dasharray={s.dashed ? '4 3' : undefined} stroke-linejoin="round" stroke-linecap="round" />
      {/each}
      {#each series.filter((s) => s.strong) as s (s.key)}
        <path d={path(s.values)} fill="none" stroke={s.color} stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
      {/each}
      {#each endLabels as l (l.key)}
        <text class="app-chart__label" x={PAD.left + plotW + 6} y={l.y} dominant-baseline="middle">{l.label}</text>
      {/each}
      {#if hover !== null}
        <line class="app-chart__cross" x1={tipLeft} x2={tipLeft} y1={PAD.top} y2={PAD.top + plotH} />
        {#each series as s (s.key)}
          {@const v = s.values[hover]}
          {#if v !== null && Number.isFinite(v)}
            <circle cx={tipLeft} cy={py(v)} r="4" fill={s.color} class="app-chart__dot" />
          {/if}
        {/each}
      {/if}
    </svg>
    {#if hover !== null}
      <div class="app-chart__tip" style:left="{Math.min(tipLeft, width - 160)}px" aria-live="polite">
        <strong>{formatX(xs[hover])}</strong>
        {#each series as s (s.key)}
          {@const v = s.values[hover]}
          <span><i style:background={s.color} aria-hidden="true"></i>{s.label}: {v === null ? '—' : formatY(v)}</span>
        {/each}
      </div>
    {/if}
  </div>
  <details class="app-chart__table">
    <summary>{tableLabel}</summary>
    <div class="app-chart__scroll">
      <table>
        <thead>
          <tr><th scope="col">{xLabel}</th>{#each series as s (s.key)}<th scope="col">{s.label}</th>{/each}</tr>
        </thead>
        <tbody>
          {#each xs as x, i (x)}
            <tr><th scope="row">{formatX(x)}</th>{#each series as s (s.key)}<td>{s.values[i] === null ? '—' : formatY(s.values[i] as number)}</td>{/each}</tr>
          {/each}
        </tbody>
      </table>
    </div>
  </details>
</figure>

<style>
  .app-chart {
    margin: var(--ez-space-sm) 0;
  }
  .app-chart__title {
    font-size: var(--ez-fs-sm);
    font-weight: 600;
  }
  .app-chart__legend {
    display: flex;
    flex-wrap: wrap;
    gap: var(--ez-space-md);
    list-style: none;
    margin: var(--ez-space-2xs) 0;
    padding: 0;
    font-size: var(--ez-fs-xs);
    color: var(--ez-color-text-muted);
  }
  .app-chart__legend li {
    display: flex;
    align-items: center;
    gap: var(--ez-space-2xs);
  }
  .app-chart__plot {
    position: relative;
    width: 100%;
  }
  .app-chart__plot svg {
    display: block;
    border-radius: var(--ez-radius-sm);
  }
  .app-chart__grid {
    stroke: var(--app-chart-grid);
    stroke-width: 1;
  }
  .app-chart__tick,
  .app-chart__label {
    font-size: var(--ez-fs-xs);
    fill: var(--ez-color-text-muted);
    font-variant-numeric: tabular-nums;
  }
  .app-chart__label {
    fill: var(--ez-color-text);
  }
  .app-chart__cross {
    stroke: var(--ez-color-text-muted);
    stroke-width: 1;
  }
  .app-chart__dot {
    stroke: var(--ez-color-surface);
    stroke-width: 2;
  }
  .app-chart__tip {
    position: absolute;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: var(--ez-space-2xs);
    min-width: 9rem;
    padding: var(--ez-space-xs) var(--ez-space-sm);
    border: 1px solid var(--ez-color-border);
    border-radius: var(--ez-radius-sm);
    background: var(--ez-color-surface);
    box-shadow: var(--ez-shadow-sm);
    font-size: var(--ez-fs-xs);
    pointer-events: none;
    font-variant-numeric: tabular-nums;
  }
  .app-chart__tip i {
    display: inline-block;
    width: 0.6rem;
    height: 0.6rem;
    margin-right: var(--ez-space-2xs);
    border-radius: var(--ez-radius-pill);
  }
  .app-chart__table summary {
    font-size: var(--ez-fs-xs);
    color: var(--ez-color-text-muted);
    cursor: pointer;
  }
  .app-chart__scroll {
    max-height: 16rem;
    overflow: auto;
  }
  .app-chart__table table {
    border-collapse: collapse;
    font-size: var(--ez-fs-xs);
    font-variant-numeric: tabular-nums;
  }
  .app-chart__table th,
  .app-chart__table td {
    padding: var(--ez-space-2xs) var(--ez-space-sm);
    text-align: right;
    border-bottom: 1px solid var(--ez-color-border-soft);
  }
</style>
