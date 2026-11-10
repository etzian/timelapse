// Consent state for optional data uses (cookies, analytics, crash reports, sync).
//
// Law 25 / CASL rules this encodes:
//   - nothing optional runs before a choice: `allowed()` is false until decided;
//   - every optional purpose is OFF by default, never pre-ticked;
//   - rejecting is exactly as easy as accepting (`rejectAll` mirrors `acceptAll`);
//   - the choice is stored with its date and the policy version it was given for,
//     and a new policy version asks again;
//   - the user can change it at any time (`save`, `reset`).
//
// Storage is injected: localStorage on the web, the Tauri store in desktop apps.

/** @typedef {{ id: string, necessary?: boolean }} Purpose */
/** @typedef {{ version: string, decidedAt: string, choices: Record<string, boolean> }} Decision */

/**
 * @param {{ purposes: Purpose[], version: string, storage: { get(key: string): any, set(key: string, value: any): void, remove?(key: string): void }, key?: string, now?: () => Date }} opts
 */
export function createConsent({ purposes, version, storage, key = 'etzian.consent', now = () => new Date() }) {
  const optional = purposes.filter((p) => !p.necessary).map((p) => p.id);
  const known = new Set(purposes.map((p) => p.id));
  /** @type {Set<() => void>} */
  const listeners = new Set();

  /** @returns {Decision | null} */
  function read() {
    let d;
    try {
      d = storage.get(key);
      if (typeof d === 'string') d = JSON.parse(d);
    } catch {
      return null;
    }
    if (!d || typeof d !== 'object' || typeof d.version !== 'string' || typeof d.decidedAt !== 'string' || typeof d.choices !== 'object' || d.choices === null) return null;
    return d;
  }

  let decision = read();
  const emit = () => listeners.forEach((f) => f());

  /** @param {Record<string, boolean>} wanted */
  function write(wanted) {
    const choices = Object.fromEntries(purposes.map((p) => [p.id, p.necessary ? true : wanted[p.id] === true]));
    decision = { version, decidedAt: now().toISOString(), choices };
    storage.set(key, JSON.stringify(decision));
    emit();
  }

  return {
    optional,
    /** True when the user has not chosen yet, or chose under an older policy version. */
    needsPrompt: () => !decision || decision.version !== version,
    /** True only when the user has explicitly allowed this purpose under the current version. */
    allowed(id) {
      if (!known.has(id)) return false;
      if (purposes.find((p) => p.id === id)?.necessary) return true;
      return !!decision && decision.version === version && decision.choices[id] === true;
    },
    /** The current choices for display (all false before a decision). */
    choices() {
      const out = {};
      for (const p of purposes) out[p.id] = p.necessary ? true : !!decision && decision.version === version && decision.choices[p.id] === true;
      return out;
    },
    decidedAt: () => (decision && decision.version === version ? decision.decidedAt : null),
    acceptAll: () => write(Object.fromEntries(optional.map((id) => [id, true]))),
    rejectAll: () => write({}),
    /** @param {Record<string, boolean>} choices */
    save: (choices) => write(choices),
    reset() {
      decision = null;
      storage.remove?.(key);
      emit();
    },
    subscribe(fn) {
      listeners.add(fn);
      return () => listeners.delete(fn);
    }
  };
}
