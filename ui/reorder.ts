// Drag and drop to reorder the strip. Built on pointer events instead of HTML5
// drag and drop, whose ghost image and snapping feel janky in WebView2. Rows
// only ever move by `transform`, so a drag never triggers layout.

/** Pointer travel (px) before a press becomes a drag, so clicks stay clicks. */
const DRAG_THRESHOLD = 5;

export interface DragHandlers {
  /** The row was released at a new index. Its drag transforms are still applied. */
  drop(id: string, index: number): void;
  /** The gesture is over, whether it was dropped or cancelled. */
  end(): void;
}

interface Gesture {
  readonly pointerId: number;
  readonly row: HTMLLIElement;
  readonly rows: readonly HTMLLIElement[];
  readonly from: number;
  readonly startY: number;
  /** Distance between two neighbouring rows, gap included. */
  readonly slot: number;
  dragging: boolean;
  to: number;
}

export class DragReorder {
  readonly #list: HTMLUListElement;
  readonly #handlers: DragHandlers;
  #gesture: Gesture | null = null;
  #swallowClick = false;

  constructor(list: HTMLUListElement, handlers: DragHandlers) {
    this.#list = list;
    this.#handlers = handlers;
    list.addEventListener("pointerdown", (event) => this.#press(event));
    list.addEventListener("pointermove", (event) => this.#move(event));
    list.addEventListener("pointerup", (event) => this.#release(event));
    list.addEventListener("pointercancel", () => this.#finish(false));
    list.addEventListener("lostpointercapture", () => this.#finish(false));
    // A drag ends with a click on the avatar it started from; that click must not switch accounts.
    list.addEventListener(
      "click",
      (event) => {
        if (this.#swallowClick) event.stopImmediatePropagation();
      },
      { capture: true },
    );
    window.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && this.#gesture?.dragging) this.#finish(false);
    });
  }

  get dragging(): boolean {
    return this.#gesture?.dragging ?? false;
  }

  #press(event: PointerEvent): void {
    // A press that never became a drag may have been released outside the
    // list, so only a drag in progress blocks a new one.
    if (event.button !== 0 || this.dragging) return;
    const target = event.target instanceof Element ? event.target.closest("li.account") : null;
    const rows = [...this.#list.querySelectorAll<HTMLLIElement>(":scope > li.account")];
    const from = target instanceof HTMLLIElement ? rows.indexOf(target) : -1;
    const [first, second] = rows;
    const row = rows[from];
    if (!row || !first || !second) return;
    this.#gesture = {
      pointerId: event.pointerId,
      row,
      rows,
      from,
      startY: event.clientY,
      slot: second.getBoundingClientRect().top - first.getBoundingClientRect().top,
      dragging: false,
      to: from,
    };
  }

  #move(event: PointerEvent): void {
    const g = this.#gesture;
    if (!g || event.pointerId !== g.pointerId) return;
    const travel = event.clientY - g.startY;
    if (!g.dragging) {
      if (Math.abs(travel) < DRAG_THRESHOLD) return;
      g.dragging = true;
      this.#list.setPointerCapture(g.pointerId);
      g.row.classList.add("dragging");
    }
    // Keep the dragged row inside the list, then open a gap where it would land.
    const offset = clamp(travel, -g.from * g.slot, (g.rows.length - 1 - g.from) * g.slot);
    g.to = clamp(Math.round(g.from + offset / g.slot), 0, g.rows.length - 1);
    g.row.style.transform = `translateY(${offset}px)`;
    g.rows.forEach((row, index) => {
      if (row !== g.row) row.style.transform = shiftFor(index, g.from, g.to, g.slot);
    });
  }

  #release(event: PointerEvent): void {
    const g = this.#gesture;
    if (!g || event.pointerId !== g.pointerId) return;
    if (!g.dragging) {
      this.#gesture = null;
      return;
    }
    // Swallow only the click that belongs to this release.
    this.#swallowClick = true;
    setTimeout(() => {
      this.#swallowClick = false;
    });
    // Letting go outside the strip cancels, like dropping a file outside a window.
    const outside = event.clientX < 0 || event.clientX > window.innerWidth;
    this.#finish(!outside);
  }

  #finish(commit: boolean): void {
    const g = this.#gesture;
    if (!g) return;
    this.#gesture = null;
    if (g.dragging) {
      if (commit && g.to !== g.from && g.row.dataset.id) {
        this.#handlers.drop(g.row.dataset.id, g.to);
      }
      // Clearing the transforms lets every row glide back (or stay, after a drop).
      for (const row of g.rows) row.style.transform = "";
      g.row.classList.remove("dragging");
      if (this.#list.hasPointerCapture(g.pointerId)) this.#list.releasePointerCapture(g.pointerId);
    }
    this.#handlers.end();
  }
}

/** How far a non-dragged row moves aside to make room. */
function shiftFor(index: number, from: number, to: number, slot: number): string {
  if (from < index && index <= to) return `translateY(${-slot}px)`;
  if (to <= index && index < from) return `translateY(${slot}px)`;
  return "";
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
