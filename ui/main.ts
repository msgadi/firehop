import { backend } from "./backend";
import { shortcutTarget } from "./model";
import { Strip } from "./strip";

const strip = new Strip(document.body, {
  activate: (id) => run(backend.activateAccount(id)),
  openMenu: (account) => run(backend.showAccountMenu(account)),
  add: () => run(backend.addAccount()),
});

// Subscribe before the first fetch, so an update that lands in between is not lost.
await backend.onRosterChanged((view) => strip.render(view));
strip.render(await backend.roster());

// Ctrl+1…9 switch accounts while the strip has focus. (While a chat has focus,
// those keys belong to WhatsApp.)
window.addEventListener("keydown", (event) => {
  if (!event.ctrlKey || event.altKey || event.shiftKey) return;
  const accounts = strip.view?.accounts ?? [];
  const index = shortcutTarget(Number(event.key), accounts.length);
  const target = index === null ? undefined : accounts[index];
  if (target) {
    event.preventDefault();
    run(backend.activateAccount(target.id));
  }
});

function run(task: Promise<unknown>): void {
  task.catch((error: unknown) => console.error("[strip]", error));
}
