// The account editor window: photo, name, full name and colour for one account.

import { backend } from "./backend";
import {
  ACCENT_COUNT,
  type AccountView,
  accentVar,
  FULL_NAME_MAX,
  initialsOf,
  NAME_MAX,
  type RosterView,
} from "./model";

const accountId = new URLSearchParams(location.search).get("id") ?? "";

const form = query<HTMLFormElement>(document, "form.editor");
const nameInput = query<HTMLInputElement>(form, 'input[name="name"]');
const fullNameInput = query<HTMLInputElement>(form, 'input[name="fullName"]');
const swatches = query(form, ".swatches");
const avatar = query(form, ".avatar");
const avatarInitials = query(avatar, ".initials");
const avatarImg = query<HTMLImageElement>(avatar, "img");
const removePhoto = query<HTMLButtonElement>(form, '[data-action="remove-photo"]');
const errorLine = query(form, ".error");

nameInput.maxLength = NAME_MAX;
fullNameInput.maxLength = FULL_NAME_MAX;
buildSwatches();

let loaded = false;

// The form is filled once; later roster updates refresh only the photo, so
// typing is never overwritten. If the account disappears, the editor closes.
const onRoster = (view: RosterView): void => {
  const account = view.accounts.find((a) => a.id === accountId);
  if (!account) {
    void backend.closeThisWindow();
    return;
  }
  if (!loaded) {
    fill(account);
    loaded = true;
  }
  showPhoto(account.photo);
};

await backend.onRosterChanged(onRoster);
onRoster(await backend.roster());
nameInput.focus();
nameInput.select();

nameInput.addEventListener("input", refreshPreview);
swatches.addEventListener("change", refreshPreview);

form.addEventListener("submit", (event) => {
  event.preventDefault();
  void save();
});

form.addEventListener("click", (event) => {
  const action = (event.target as Element | null)
    ?.closest<HTMLElement>("[data-action]")
    ?.getAttribute("data-action");
  if (action === "cancel") void backend.closeThisWindow();
  if (action === "choose-photo") void attempt(() => backend.chooseAccountPhoto(accountId));
  if (action === "remove-photo") void attempt(() => backend.clearAccountPhoto(accountId));
});

window.addEventListener("keydown", (event) => {
  if (event.key === "Escape") void backend.closeThisWindow();
});

async function save(): Promise<void> {
  if (!nameInput.value.trim()) {
    showError("Please enter a name.");
    nameInput.focus();
    return;
  }
  const saved = await attempt(() =>
    backend.updateAccount(accountId, {
      name: nameInput.value,
      fullName: fullNameInput.value,
      accent: selectedAccent(),
    }),
  );
  if (saved) await backend.closeThisWindow();
}

/** Runs a backend call, showing its error inline. Resolves to whether it succeeded. */
async function attempt(task: () => Promise<unknown>): Promise<boolean> {
  showError("");
  try {
    await task();
    return true;
  } catch (error) {
    showError(String(error));
    return false;
  }
}

function fill(account: AccountView): void {
  nameInput.value = account.name;
  fullNameInput.value = account.fullName ?? "";
  const swatch = swatches.querySelector<HTMLInputElement>(`input[value="${account.accent}"]`);
  if (swatch) swatch.checked = true;
  refreshPreview();
}

function refreshPreview(): void {
  avatar.style.setProperty("--accent", accentVar(selectedAccent()));
  avatarInitials.textContent = initialsOf(nameInput.value) || "?";
}

function showPhoto(photo: string | null): void {
  avatar.classList.toggle("has-photo", photo !== null);
  avatarInitials.hidden = photo !== null;
  avatarImg.hidden = photo === null;
  removePhoto.hidden = photo === null;
  if (photo === null) avatarImg.removeAttribute("src");
  else if (avatarImg.getAttribute("src") !== photo) avatarImg.src = photo;
}

function selectedAccent(): number {
  const checked = swatches.querySelector<HTMLInputElement>("input:checked");
  return checked ? Number(checked.value) : 0;
}

function buildSwatches(): void {
  for (let accent = 0; accent < ACCENT_COUNT; accent++) {
    const label = document.createElement("label");
    label.className = "swatch";
    label.style.setProperty("--accent", accentVar(accent));
    const input = document.createElement("input");
    input.type = "radio";
    input.name = "accent";
    input.value = String(accent);
    input.setAttribute("aria-label", `Color ${accent + 1}`);
    label.append(input);
    swatches.append(label);
  }
}

function showError(message: string): void {
  errorLine.textContent = message;
  errorLine.hidden = message === "";
}

function query<T extends Element = HTMLElement>(root: ParentNode, selector: string): T {
  const el = root.querySelector<T>(selector);
  if (!el) throw new Error(`editor markup is missing ${selector}`);
  return el;
}
