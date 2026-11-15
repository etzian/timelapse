/**
 * A bounded cache of promises, one per key, for frames that are slow to make. A frame is fetched once
 * per key; a failure is forgotten so the next ask retries. `scope` names what the entries belong to
 * (the run shown): asking under another scope drops the old entries.
 */
export function createFrameCache<T>(limit = 240) {
  let scope = '';
  const entries = new Map<string, Promise<T>>();
  return {
    get(scopeKey: string, key: string, fetch: () => Promise<T>): Promise<T> {
      if (scopeKey !== scope) {
        entries.clear();
        scope = scopeKey;
      }
      const hit = entries.get(key);
      if (hit) return hit;
      const p = fetch();
      entries.set(key, p);
      p.catch(() => {
        if (entries.get(key) === p) entries.delete(key);
      });
      while (entries.size > limit) {
        const oldest = entries.keys().next().value as string;
        entries.delete(oldest);
      }
      return p;
    },
    clear(): void {
      entries.clear();
    },
    get size(): number {
      return entries.size;
    }
  };
}
