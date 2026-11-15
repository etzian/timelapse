/**
 * Every key this app writes to `localStorage`, in one place. "Erase all data" removes each
 * of them (settings store); a new key is added here or it is not erased. What they hold is
 * listed in `legal/data-map.yaml`.
 */
import { RECENT_KEY } from './project';

export const STORAGE_KEYS = {
  language: 'timelapse.language',
  density: 'timelapse.density',
  theme: 'timelapse.theme',
  recent: RECENT_KEY,
  /** Which Sidebar groups are folded (written by the design system's Sidebar). */
  sidebar: 'timelapse:sidebar'
} as const;

export const ALL_STORAGE_KEYS: readonly string[] = Object.values(STORAGE_KEYS);
