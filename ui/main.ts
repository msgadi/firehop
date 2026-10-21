import { backend } from "./backend";
import { type RosterView, shortcutTarget, totalUnread } from "./model";
import { Strip } from "./strip";
import { TaskbarBadge } from "./taskbar";

const strip = new Strip(document.body, {
  activate: (id) => run(backend.activateAccount(id)),
  openMenu: (account) => run(backend.showAccountMenu(account, strip.moveActions(account.id))),
  add: () => run(backend.addAccount()),
  // The strip shows the new order before it is saved; if saving fails, show the real one again.
  move: (id, index) => run(backend.moveAccount(id, index).catch(resync)),
});

const taskbar = new TaskbarBadge();

/** Everything that shows the roster: the strip and the taskbar badge. */
function show(view: RosterView): void {
  strip.render(view);
  taskbar.show(totalUnread(view));
}

// Subscribe before the first fetch, so an update that lands in between is not lost.
await backend.onRosterChanged(show);
show(await backend.roster());

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

async function resync(error: unknown): Promise<void> {
  console.error("[strip]", error);
  show(await backend.roster());
}

function run(task: Promise<unknown>): void {
  task.catch((error: unknown) => console.error("[strip]", error));
}
