/** Pure rules of the before/after wipe: where the divider sits and how keys and the pointer move it. */

export const WIPE_STEP = 1;
export const WIPE_BIG_STEP = 10;

/** Keeps a divider position inside 0..100 (a non-number reads as the middle). */
export function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 50;
  return Math.min(100, Math.max(0, value));
}

/** The position after a key, or `null` when the key does not move the divider. */
export function wipeKey(position: number, key: string, shift = false): number | null {
  const step = shift ? WIPE_BIG_STEP : WIPE_STEP;
  switch (key) {
    case 'ArrowLeft':
    case 'ArrowDown':
      return clampPercent(position - step);
    case 'ArrowRight':
    case 'ArrowUp':
      return clampPercent(position + step);
    case 'PageDown':
      return clampPercent(position - WIPE_BIG_STEP);
    case 'PageUp':
      return clampPercent(position + WIPE_BIG_STEP);
    case 'Home':
      return 0;
    case 'End':
      return 100;
    default:
      return null;
  }
}

/** Divider position for a pointer at `clientX` over a frame starting at `left` and `width` wide. */
export function percentFromPointer(clientX: number, left: number, width: number): number {
  if (!(width > 0)) return 50;
  return clampPercent(((clientX - left) / width) * 100);
}
