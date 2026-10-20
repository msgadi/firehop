// Renders the account strip. Rows are keyed by account id and updated in
// place, so CSS transitions (active pill, badge pop) animate instead of jumping.

import {
  type AccountView,
  accentVar,
  accessibleLabel,
  badgeText,
  type RosterView,
  tooltip,
} from "./model";

export interface StripActions {
  activate(id: string): void;
  openMenu(account: AccountView): void;
  add(): void;
}

export class Strip {
  readonly #list: HTMLUListElement;
  readonly #add: HTMLButtonElement;
  readonly #rows = new Map<string, HTMLLIElement>();
  #view: RosterView | null = null;

  constructor(root: HTMLElement, actions: StripActions) {
    this.#list = query(root, "ul.accounts");
    this.#add = query(root, "button.add");

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
    this.#add.addEventListener("click", () => actions.add());
  }

  get view(): RosterView | null {
    return this.#view;
  }

  render(view: RosterView): void {
    this.#view = view;
    const wanted = new Set(view.accounts.map((a) => a.id));
    for (const [id, row] of this.#rows) {
      if (!wanted.has(id)) {
        row.remove();
        this.#rows.delete(id);
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
    this.#add.disabled = !view.canAdd;
    this.#add.title = view.canAdd ? "Link another account" : "Account limit reached";
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
