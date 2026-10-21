// The unread total on Firehop's taskbar button, as a Windows overlay icon.
// Official WhatsApp Desktop shows its count as a Windows badge, which Windows
// paints in the system accent colour, so ours is drawn the same way. Drawing
// happens here, with the system font, so the number stays sharp at any scale.

import { backend, type SquareImage } from "./backend";
import { badgeText } from "./model";

/** Logical size of a taskbar overlay icon on Windows. */
const OVERLAY_SIZE = 16;
/** Windows' default accent, for when the system colour cannot be read. */
const FALLBACK_ACCENT = "#0078d4";

export class TaskbarBadge {
  /** What is on the taskbar now, as text and colour; "" when there is nothing. */
  #shown: string | null = null;
  /** Updates run one after another, so a slow one never overwrites a newer count. */
  #queue: Promise<void> = Promise.resolve();

  show(total: number): void {
    this.#queue = this.#queue
      .then(() => this.#apply(badgeText(total)))
      .catch((error: unknown) => console.error("[taskbar]", error));
  }

  async #apply(text: string): Promise<void> {
    // Read the accent each time: it follows the user's Windows colour settings.
    const accent = text === "" ? "" : ((await backend.accentColor()) ?? FALLBACK_ACCENT);
    const key = text === "" ? "" : `${text} ${accent}`;
    if (key === this.#shown) return;
    await backend.setTaskbarOverlay(text === "" ? null : draw(text, accent));
    this.#shown = key;
  }
}

/** A round badge (a rounded square for 2+ characters) with the count centred on it. */
function draw(text: string, accent: string): SquareImage {
  // Whole-pixel scale keeps the edges crisp; Windows maps it onto the overlay slot.
  const size = OVERLAY_SIZE * Math.max(1, Math.round(window.devicePixelRatio));
  const canvas = new OffscreenCanvas(size, size);
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("2D canvas is unavailable");
  ctx.fillStyle = accent;
  ctx.beginPath();
  ctx.roundRect(0, 0, size, size, text.length === 1 ? size / 2 : size * 0.32);
  ctx.fill();
  const fontScale = text.length === 1 ? 0.72 : text.length === 2 ? 0.62 : 0.46;
  ctx.font = `700 ${Math.round(size * fontScale)}px "Segoe UI Variable Text", "Segoe UI", sans-serif`;
  ctx.fillStyle = readableOn(accent);
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  // Segoe's digits sit slightly high on the middle baseline; nudge them to optical centre.
  ctx.fillText(text, size / 2, size * 0.54);
  const pixels = ctx.getImageData(0, 0, size, size).data;
  return { rgba: new Uint8Array(pixels.buffer), size };
}

/** Black or white, whichever reads better on a `#rrggbb` colour. */
function readableOn(hex: string): string {
  const value = Number.parseInt(hex.slice(1), 16);
  const r = (value >> 16) & 0xff;
  const g = (value >> 8) & 0xff;
  const b = value & 0xff;
  const luminance = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
  return luminance > 0.6 ? "#000000" : "#ffffff";
}
