/** Mirrors `RosterView` in crates/core/src/application/view.rs. */
export interface RosterView {
  readonly accounts: readonly AccountView[];
  readonly activeId: string | null;
  readonly canAdd: boolean;
}

export interface AccountView {
  readonly id: string;
  readonly name: string;
  readonly fullName: string | null;
  readonly initials: string;
  readonly accent: number;
  /** A `data:image/png` URL, or null when the account has no custom photo. */
  readonly photo: string | null;
  readonly unread: number;
}

/** Mirrors `ProfileForm` in src-tauri/src/commands.rs. */
export interface ProfileForm {
  readonly name: string;
  readonly fullName: string;
  readonly accent: number;
}

/** Reorder actions for one account; null where it is already first or last. */
export interface MoveActions {
  readonly up: (() => void) | null;
  readonly down: (() => void) | null;
}

/** The accounts with `id` moved to `index`. Mirrors `Roster::move_to` in the core. */
export function moved(
  accounts: readonly AccountView[],
  id: string,
  index: number,
): readonly AccountView[] {
  const rest = accounts.filter((a) => a.id !== id);
  const account = accounts.find((a) => a.id === id);
  if (!account) return accounts;
  const at = Math.min(Math.max(index, 0), rest.length);
  return [...rest.slice(0, at), account, ...rest.slice(at)];
}

/** Must match `Accent::COUNT`, `AccountName::MAX_CHARS` and `FullName::MAX_CHARS` in the core. */
export const ACCENT_COUNT = 8;
export const NAME_MAX = 32;
export const FULL_NAME_MAX = 64;

const BADGE_CAP = 99;

/** Text for an unread badge: nothing at zero, "99+" past the cap. */
export function badgeText(unread: number): string {
  if (unread <= 0) return "";
  return unread > BADGE_CAP ? `${BADGE_CAP}+` : String(unread);
}

/** Hover text for an account: "Work · Mohammed Gadi". */
export function tooltip(account: AccountView): string {
  return account.fullName ? `${account.name} · ${account.fullName}` : account.name;
}

/** What a screen reader announces for an account button. */
export function accessibleLabel(account: AccountView): string {
  const label = tooltip(account);
  if (account.unread <= 0) return label;
  const chats = account.unread === 1 ? "unread chat" : "unread chats";
  return `${label}, ${badgeText(account.unread)} ${chats}`;
}

/**
 * Live preview of the avatar initials while typing. Mirrors
 * `AccountName::initials` in the core, which stays the source of truth.
 */
export function initialsOf(name: string): string {
  return name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((word) => [...word][0] ?? "")
    .join("")
    .toUpperCase();
}

/** Index of the account a Ctrl+digit shortcut points at (1-based; 9 means "last"). */
export function shortcutTarget(digit: number, count: number): number | null {
  if (count === 0 || digit < 1 || digit > 9) return null;
  if (digit === 9) return count - 1;
  return digit <= count ? digit - 1 : null;
}

export function accentVar(accent: number): string {
  return `var(--accent-${accent})`;
}
