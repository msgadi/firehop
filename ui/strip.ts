// Renders the account strip. Rows are keyed by account id and updated in
// place, so CSS transitions (active pill, badge pop) animate instead of jumping.
// Reordering animates too: rows glide to their new slots (FLIP).

import {
  type AccountView,
  accentVar,
  accessibleLabel,
  badgeText,
  type MoveActions,
  moved,
  type RosterView,
  tooltip,
} from "./model";
import { DragReorder } from "./reorder";

export interface StripActions {
  activate(id: string): void;
  openMenu(account: AccountView): void;
  add(): void;
  /** Persist a new position. The strip has already shown it. */
  move(id: string, index: number): void;
}

/** Matches the strip's other motion: under the ~240 ms budget, same easing as `--ease-out`. */
const GLIDE: KeyframeAnimationOptions = { duration: 200, easing: "cubic-bezier(0.22, 1, 0.36, 1)" };

export class Strip {
  readonly #list: HTMLUListElement;
  readonly #add: HTMLButtonElement;
  readonly #rows = new Map<string, HTMLLIElement>();
  readonly #actions: StripActions;
  readonly #drag: DragReorder;
  #view: RosterView | null = null;
  /** An update that arrived mid-drag; applied when the drag ends. */
  #deferred: RosterView | null = null;

  constructor(root: HTMLElement, actions: StripActions) {
    this.#list = query(root, "ul.accounts");
    this.#add = query(root, "button.add");
    this.#actions = actions;
    this.#drag = new DragReorder(this.#list, {
      drop: (id, index) => this.#move(id, index),
      end: () => this.#flushDeferred(),
    });

    // One delegated listener per gesture instead of one per row.
    this.#list.addEventListener("click", (event) => {
      const id = rowId(event.target);
      if (id) actions.activate(id);
    });
    this.#list.addEventListener("contextmenu", (event) => {
      event.preventDefault();
      const account = this.#account(rowId(event.target));
      if (account) actions.openMenu(account);
    });
    // Alt+↑/↓ moves the focused account: reordering without a mouse.
    this.#list.addEventListener("keydown", (event) => {
      const direction = event.key === "ArrowUp" ? "up" : event.key === "ArrowDown" ? "down" : null;
      const id = rowId(event.target);
      if (!event.altKey || event.ctrlKey || event.shiftKey || !direction || !id) return;
      event.preventDefault();
      this.moveActions(id)[direction]?.();
    });
    this.#add.addEventListener("click", () => actions.add());
  }

  get view(): RosterView | null {
    return this.#view;
  }

  render(view: RosterView): void {
    if (this.#drag.dragging) {
      this.#deferred = view;
      return;
    }
    this.#apply(view);
  }

  /** "Move up" / "Move down" for an account, or null where it cannot go further. */
  moveActions(id: string): MoveActions {
    const accounts = this.#view?.accounts ?? [];
    const index = accounts.findIndex((a) => a.id === id);
    const to = (target: number) =>
      index >= 0 && target >= 0 && target < accounts.length ? () => this.#move(id, target) : null;
    return { up: to(index - 1), down: to(index + 1) };
  }

  /** Shows the new order at once, then asks the backend to keep it. */
  #move(id: string, index: number): void {
    // An update deferred during the drag is newer (badges, names) but has the old order.
    const base = this.#deferred ?? this.#view;
    this.#deferred = null;
    if (!base) return;
    this.#apply({ ...base, accounts: moved(base.accounts, id, index) });
    this.#actions.move(id, index);
  }

  #flushDeferred(): void {
    const view = this.#deferred;
    this.#deferred = null;
    if (view) this.#apply(view);
  }

  #apply(view: RosterView): void {
    this.#view = view;
    const wanted = new Set(view.accounts.map((a) => a.id));
    for (const [id, row] of this.#rows) {
      if (!wanted.has(id)) {
        row.remove();
        this.#rows.delete(id);
      }
    }
    const reordering = view.accounts.some(
      (a, i) => this.#list.children[i] !== this.#rows.get(a.id),
    );
    // Measure where rows are on screen (drag offsets included) before anything moves.
    const before = reordering ? this.#positions() : null;
    const focused = rowId(document.activeElement);
    if (before) {
      for (const row of this.#rows.values()) {
        row.style.transition = "none";
        row.style.transform = "";
      }
    }
    view.accounts.forEach((account, index) => {
      const row = this.#rows.get(account.id) ?? this.#createRow(account.id);
      updateRow(row, account, account.id === view.activeId);
      // Only move nodes that are out of place; moving restarts CSS transitions.
      if (this.#list.children[index] !== row) {
        this.#list.insertBefore(row, this.#list.children[index] ?? null);
      }
    });
    if (before) this.#glideFrom(before);
    // Moving a node drops its focus; keyboard users keep their place.
    const refocus = focused ? this.#rows.get(focused)?.querySelector<HTMLElement>(".avatar") : null;
    if (refocus && document.activeElement !== refocus) refocus.focus({ preventScroll: true });
    this.#add.disabled = !view.canAdd;
    this.#add.title = view.canAdd ? "Link another account" : "Account limit reached";
  }

  #positions(): Map<string, number> {
    const tops = new Map<string, number>();
    for (const [id, row] of this.#rows) tops.set(id, row.getBoundingClientRect().top);
    return tops;
  }

  /** FLIP: start each moved row where it was and let it glide into its new slot. */
  #glideFrom(before: Map<string, number>): void {
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    for (const [id, row] of this.#rows) {
      const from = before.get(id);
      // Reading the position also flushes styles, so restoring the transition cannot animate the reset.
      const dy = from === undefined ? 0 : from - row.getBoundingClientRect().top;
      row.style.transition = "";
      if (!still && Math.abs(dy) > 0.5) {
        row.animate([{ transform: `translateY(${dy}px)` }, { transform: "none" }], GLIDE);
      }
    }
  }

  #account(id: string | null): AccountView | undefined {
    return id ? this.#view?.accounts.find((a) => a.id === id) : undefined;
  }

  #createRow(id: string): HTMLLIElement {
    const row = document.createElement("li");
    row.className = "account";
    row.dataset.id = id;
    row.innerHTML = `
      <span class="pill" aria-hidden="true"></span>
      <button class="avatar" type="button">
        <span class="initials"></span>
        <img alt="" hidden />
      </button>
      <span class="badge" aria-hidden="true"></span>`;
    this.#rows.set(id, row);
    return row;
  }
}

function updateRow(row: HTMLLIElement, account: AccountView, active: boolean): void {
  const avatar = query<HTMLButtonElement>(row, ".avatar");
  const badge = query(row, ".badge");
  row.classList.toggle("active", active);
  row.style.setProperty("--accent", accentVar(account.accent));
  if (active) avatar.setAttribute("aria-current", "true");
  else avatar.removeAttribute("aria-current");
  avatar.setAttribute("aria-label", accessibleLabel(account));
  avatar.title = tooltip(account);
  setText(query(row, ".initials"), account.initials);
  showPhoto(avatar, account.photo);
  const text = badgeText(account.unread);
  if (badge.textContent !== text) {
    setText(badge, text);
    badge.classList.toggle("visible", text !== "");
    // Re-trigger the pop animation whenever the count changes.
    badge.classList.remove("pop");
    void badge.offsetWidth;
    if (text) badge.classList.add("pop");
  }
}

/** Shows the photo over the initials, touching the DOM only when it changes. */
function showPhoto(avatar: HTMLButtonElement, photo: string | null): void {
  const img = query<HTMLImageElement>(avatar, "img");
  avatar.classList.toggle("has-photo", photo !== null);
  query(avatar, ".initials").hidden = photo !== null;
  img.hidden = photo === null;
  if (photo === null) img.removeAttribute("src");
  else if (img.getAttribute("src") !== photo) img.src = photo;
}

function setText(el: Element, text: string): void {
  if (el.textContent !== text) el.textContent = text;
}

function rowId(target: EventTarget | null): string | null {
  return target instanceof Element
    ? (target.closest<HTMLElement>("li.account")?.dataset.id ?? null)
    : null;
}

function query<T extends Element = HTMLElement>(root: ParentNode, selector: string): T {
  const el = root.querySelector<T>(selector);
  if (!el) throw new Error(`strip markup is missing ${selector}`);
  return el;
}
