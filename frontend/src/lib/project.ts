/**
 * Recent projects: pure helpers over a list of `{ path, openedAt }`, newest first.
 * The list lives in `localStorage` (see the project store); nothing here touches storage.
 */
export interface RecentProject {
  path: string;
  /** Epoch milliseconds of the last time the folder was opened. */
  openedAt: number;
}

export const RECENT_KEY = 'timelapse.recent-projects';
export const MAX_RECENT = 8;

/** Put `path` first with the given time; an earlier entry for the same path is replaced; capped. */
export function addRecent(list: readonly RecentProject[], path: string, openedAt: number, max = MAX_RECENT): RecentProject[] {
  if (!path) return list.slice(0, max);
  return [{ path, openedAt }, ...list.filter((r) => r.path !== path)].slice(0, max);
}

export function removeRecent(list: readonly RecentProject[], path: string): RecentProject[] {
  return list.filter((r) => r.path !== path);
}

/** Read a stored value defensively: anything malformed is dropped, duplicates keep the newest. */
export function parseRecent(raw: string | null, max = MAX_RECENT): RecentProject[] {
  if (!raw) return [];
  let data: unknown;
  try {
    data = JSON.parse(raw);
  } catch {
    return [];
  }
  if (!Array.isArray(data)) return [];
  const valid: RecentProject[] = [];
  for (const item of data) {
    if (item && typeof item === 'object' && typeof item.path === 'string' && item.path && typeof item.openedAt === 'number' && Number.isFinite(item.openedAt)) {
      valid.push({ path: item.path, openedAt: item.openedAt });
    }
  }
  valid.sort((a, b) => b.openedAt - a.openedAt);
  const seen = new Set<string>();
  return valid.filter((r) => !seen.has(r.path) && !!seen.add(r.path)).slice(0, max);
}

export const serializeRecent = (list: readonly RecentProject[]): string => JSON.stringify(list);
