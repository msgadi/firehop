// The only module that talks to the Rust side. Everything else in the UI is
// plain DOM and pure functions with no Tauri dependency.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { Image } from "@tauri-apps/api/image";
import { Menu } from "@tauri-apps/api/menu";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask } from "@tauri-apps/plugin-dialog";
import type { AccountView, MoveActions, ProfileForm, RosterView } from "./model";

/** Must match `ROSTER_CHANGED` in src-tauri/src/observer.rs. */
const ROSTER_CHANGED = "roster-changed";

/** A square RGBA bitmap, `size` × `size` pixels. */
export interface SquareImage {
  readonly rgba: Uint8Array;
  readonly size: number;
}

export const backend = {
  roster: () => invoke<RosterView>("roster"),
  addAccount: () => invoke<void>("add_account"),
  activateAccount: (id: string) => invoke<void>("activate_account", { id }),
  moveAccount: (id: string, index: number) => invoke<void>("move_account", { id, index }),
  /** The Windows accent colour as `#rrggbb`, or null if it cannot be read. */
  accentColor: () => invoke<string | null>("accent_color"),
  reloadAccount: (id: string) => invoke<void>("reload_account", { id }),
  removeAccount: (id: string) => invoke<void>("remove_account", { id }),
  openAccountEditor: (id: string) => invoke<void>("open_account_editor", { id }),
  updateAccount: (id: string, form: ProfileForm) => invoke<void>("update_account", { id, form }),
  /** Resolves to false when the user cancels the file picker. */
  chooseAccountPhoto: (id: string) => invoke<boolean>("choose_account_photo", { id }),
  clearAccountPhoto: (id: string) => invoke<void>("clear_account_photo", { id }),
  closeThisWindow: () => getCurrentWindow().close(),

  /** Shows an image over Firehop's taskbar button, or clears it with null. */
  async setTaskbarOverlay(icon: SquareImage | null): Promise<void> {
    const window = getCurrentWindow();
    if (!icon) return window.setOverlayIcon(undefined);
    const image = await Image.new(icon.rgba, icon.size, icon.size);
    try {
      await window.setOverlayIcon(image);
    } finally {
      await image.close();
    }
  },

  onRosterChanged: (handler: (view: RosterView) => void): Promise<UnlistenFn> =>
    listen<RosterView>(ROSTER_CHANGED, (event) => handler(event.payload)),

  /** Native right-click menu for an account in the strip. */
  async showAccountMenu(account: AccountView, moves: MoveActions): Promise<void> {
    const menu = await Menu.new({
      items: [
        {
          id: "edit",
          text: "Edit account…",
          action: () => void backend.openAccountEditor(account.id),
        },
        { id: "reload", text: "Reload", action: () => void backend.reloadAccount(account.id) },
        { item: "Separator" },
        {
          id: "move-up",
          text: "Move up\tAlt+↑",
          enabled: moves.up !== null,
          action: () => moves.up?.(),
        },
        {
          id: "move-down",
          text: "Move down\tAlt+↓",
          enabled: moves.down !== null,
          action: () => moves.down?.(),
        },
        { item: "Separator" },
        {
          id: "remove",
          text: "Log out and remove…",
          action: () => void confirmRemoval(account),
        },
      ],
    });
    await menu.popup();
  },
};

async function confirmRemoval(account: AccountView): Promise<void> {
  const confirmed = await ask(
    `"${account.name}" will be logged out on this computer and its local data erased.`,
    { title: "Remove account?", kind: "warning", okLabel: "Remove", cancelLabel: "Cancel" },
  );
  if (confirmed) await backend.removeAccount(account.id);
}
