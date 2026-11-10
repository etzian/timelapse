// Data export (portability): one action, a structured common format, all of
// the person's data. The envelope says what it is so it can be re-imported.

/**
 * @param {{ app: string, appVersion: string, data: Record<string, unknown>, now?: Date }} o
 */
export function buildExport({ app, appVersion, data, now = new Date() }) {
  return {
    format: 'etzian-export',
    formatVersion: 1,
    app,
    appVersion,
    exportedAt: now.toISOString(),
    data
  };
}

/** File name like `exemple-export-2026-10-17.json`. */
export function exportFileName(app, now = new Date()) {
  return `${app}-export-${now.toISOString().slice(0, 10)}.json`;
}

/** Browser download. Desktop apps use the Tauri dialog + fs plugins instead. */
export function downloadJson(name, payload) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' }));
  const a = Object.assign(document.createElement('a'), { href: url, download: name });
  document.body.append(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}
