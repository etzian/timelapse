/**
 * Builds the native menu from the pure model in `menu.ts`. Needs only `core:default`
 * (it includes `core:menu:default`). Outside the desktop shell it does nothing.
 */
import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu';

import { t } from './i18n';
import { buildMenuModel } from './menu';
import type { MenuAction, MenuEntry } from './menu';

export async function installMenu(run: (action: MenuAction) => void): Promise<void> {
  const make = async (e: MenuEntry) =>
    e.kind === 'separator'
      ? PredefinedMenuItem.new({ item: 'Separator' })
      : e.kind === 'predefined'
        ? PredefinedMenuItem.new({ item: e.item })
        : MenuItem.new({ id: e.id, text: e.text, accelerator: e.accelerator, action: () => run(e.action) });
  const submenus = await Promise.all(
    buildMenuModel(t).map(async (m) => Submenu.new({ id: m.id, text: m.text, items: await Promise.all(m.entries.map(make)) }))
  );
  const menu = await Menu.new({ items: submenus });
  await menu.setAsAppMenu();
}
