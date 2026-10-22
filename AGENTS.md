# AGENTS.md

Instructions for any AI agent or human contributor working in this repository.
This is the single source of truth. `CLAUDE.md` only points here. Read this file
fully before changing anything, and keep it up to date: when the owner gives a
new instruction, add it to **Owner instructions** below in the same change.

## What we are building

**Firehop** is an open-source desktop app that shows the real
WhatsApp Web UI, unchanged, for several accounts in one window. A slim vertical
account strip on the left switches between accounts. Apart from that strip, the
app should look and behave exactly like the official WhatsApp Desktop app.

## Owner instructions (keep every entry, newest last)

1. The app must look and behave **exactly like the official WhatsApp Desktop app**:
   same fonts, design, lines, screens, navigation and features. The only addition
   is a vertical strip on one side for switching WhatsApp accounts.
2. Use best-in-class technology. The UI must feel **sleek, "like butter"**: smooth,
   instant, no flashes or jank.
3. The repo will be **public and open source on GitHub**. Keep it clean and presentable.
4. Follow world-class architecture and clean-code principles (SOLID, clean or
   hexagonal architecture, DRY/KISS/YAGNI, intention-revealing names, small focused units).
5. **Do not assume.** For new features or improvements, suggest them, research
   them online and discuss with the owner first. Build only after agreement.
6. Record the owner's instructions in this file (and keep `CLAUDE.md` pointing
   here), so no agent has to rely on memory.
7. **Do not write test cases.** No unit, integration or UI tests, no test
   tooling (no vitest, no `#[cfg(test)]` modules), unless the owner explicitly
   asks for them later.
8. Calling (voice/video) must work. It comes from WhatsApp Web's own browser
   calling, launched July 2026.
9. The product name is **Firehop** (October 2026). It replaced the earlier
   working names "Switchboard" and "Whatzapp"; "Whatzapp" was dropped because it
   was too close to the WhatsApp trademark. Never put "WhatsApp", "Whats", "WA"
   or similar into the product name, repo name, app id or icon text.
10. App icon: a WhatsApp-style chat bubble (round, tail at the lower left, white
    inner ring) with a **flame instead of the phone**, filled with a **fire
    gradient** (yellow → orange → red). Source: `assets/icon.svg`. It is our own
    drawing: never copy WhatsApp's logo file or an emoji vendor's artwork.
    Regenerate with `npx tauri icon assets/icon.svg -o src-tauri/icons`, then
    delete the `android/` and `ios/` subfolders.
11. Every account is **customisable**: photo (any image; we crop it square and
    store a 128 px PNG), short name (shown in the strip; its initials are the
    fallback avatar), optional full name (tooltip) and colour. Edited from the
    strip: right-click → "Edit account…".
12. Like WAHA, state clearly everywhere users look (README, releases, first
    run later) that Firehop is **unofficial, not affiliated with WhatsApp or
    Meta**, that WhatsApp does not endorse third-party clients, and that
    **accounts can be restricted or banned**: use at your own risk.
13. The GitHub repo must be **outstanding** ("drool-worthy"): a polished README
    with banner and preview art, badges, clear features, how-it-works,
    architecture, FAQ, disclaimer, and full community files (contributing,
    security, issue/PR templates, CI).
14. Accounts in the strip can be **reordered** (October 2026): drag and drop
    with our own pointer-event gesture (no HTML5 drag ghost), rows gliding
    aside; Esc or releasing outside the strip cancels. Without a mouse:
    Alt+↑/↓ on a focused account, or right-click → Move up / Move down. The
    order is saved and Ctrl+1…9 follow it. Clicking still just switches.
15. **One design file for everything** (October 2026): the Claude Design
    canvas "Firehop Design" (private to the owner; its link is kept out of
    this public repo, so ask the owner for it). Never create a new design artifact per request:
    add or update boards in that canvas, on the right page (00 Cover, 01
    Foundations, 02 Components, 03 Screens, 04 Flows, 05 Brand, 06
    Proposals), tag each board SHIPPED / BUILT / PROPOSED, and add a line to
    the Cover changelog. Keep it in sync with `ui/theme.css` and the code.
16. The **taskbar button shows the total unread count** (October 2026), like
    WhatsApp Desktop: a Windows overlay badge in the system accent colour
    (Windows paints app badges in it), hidden at 0, "99+" past 99. Flashing
    the taskbar was proposed and is not built.
17. **Closing the window keeps Firehop running in the tray** (October 2026).
    The X button only hides the window; Firehop quits only from the tray
    icon's menu ("Quit Firehop"). Left click on the tray icon (or launching
    Firehop again) brings the window back. The tooltip shows the unread total.

## Non-negotiable product rules

- Load the genuine `https://web.whatsapp.com/`. Never restyle, inject into or
  scrape WhatsApp's page. Its look is the product.
- Never present the app to WhatsApp as the official client (no `win_hybrid`
  subplatform, no spoofed bridges or user agents). That risks users' numbers
  being banned. See `docs/decisions/0003-no-official-client-impersonation.md`.
- Never ship WhatsApp's assets (fonts, emoji, icons, logos) in this repo. The web
  app loads its own. Never copy WhatsApp's logo file; our icon is our own
  drawing (instruction 10). The product name is "Firehop" (instruction 9).
- WhatsApp's pages get **no IPC access**. Only the local strip webview may call
  Rust commands (`src-tauri/capabilities/strip.json`).

## Architecture in one screen

```
crates/core      firehop-core: no Tauri, no platform code
  domain/          Account, AccountName, Roster (aggregate root), unread parsing
  application/     AccountService (use cases), ports (traits), RosterView (DTO)
  policy.rs        navigation, external-link and permission rules
  layout.rs        strip/content geometry
  storage.rs       JsonRosterStore adapter (atomic JSON file)
  avatars.rs       FsAvatarStore adapter (decode, crop, 128 px PNG, data URL)
src-tauri        firehop (Tauri shell): adapters and composition root
  lib.rs           composition root: builds the window and wires adapters into the service
  sessions.rs      WebviewSessions: one WebView2 child webview and profile per account
  observer.rs      StripObserver: pushes RosterView to the strip and editor, sets the window title
  editor.rs        the account editor window (owned by the main window)
  commands.rs      IPC commands (async, so webview work never blocks the UI thread); native file picker
  platform.rs      the only raw Windows API code (WebView2 memory target, accent colour)
  tray.rs          the tray icon and its Open / Quit menu; closing the window hides to it
ui/              the account strip and editor: vanilla TypeScript, no framework
  backend.ts       the only module that talks to Rust
  strip.ts         keyed DOM rendering, so CSS transitions animate; FLIP glide on reorder
  reorder.ts       drag-and-drop gesture for reordering the strip (pointer events)
  taskbar.ts       draws the unread total as the taskbar overlay badge
  model.ts         types mirroring RosterView and ProfileForm, plus pure helpers
  editor.ts        the account editor page (photo, name, full name, colour)
  theme.css        shared design tokens for every local page
```

Dependency rule: `ui` → IPC → `src-tauri` → `firehop-core`. Core never
depends outward; the crate boundary makes the compiler enforce this.

Full detail: `docs/architecture.md`. Decisions: `docs/decisions/`. Findings
from analysing the official app: `docs/research/whatsapp-desktop-teardown.md`.

## Coding conventions

- **Rust**: edition 2024, `cargo fmt`, `cargo clippy --workspace` clean with
  `clippy::pedantic`. Errors use `thiserror`, never `unwrap()` outside
  provably-infallible spots (and then `expect("why")`). Keep `unsafe` confined
  to `src-tauri/src/platform.rs`, each block with a `// SAFETY:` comment.
- **TypeScript**: strict mode plus `noUncheckedIndexedAccess` and
  `exactOptionalPropertyTypes`. Biome for lint and format. No UI framework
  unless agreed with the owner.
- **Comments** explain *why*, not *what*. Doc comments on every public item.
- **Smoothness budget**: no layout thrash in the strip (keyed updates only), no
  white flashes (every webview has a background colour), transitions at most
  ~240 ms, and `prefers-reduced-motion` honoured.
- Keep IPC types in sync by hand: `crates/core/src/application/view.rs` and
  `src-tauri/src/commands.rs` (`ProfileForm`) ↔
  `ui/model.ts`, and event names `src-tauri/src/observer.rs` ↔ `ui/backend.ts`.

## Commands

Requires Rust (stable) and Node 24 (`.node-version`; `fnm use` picks it up).

| Task | Command |
|---|---|
| Install JS deps | `npm install` |
| Run the app (dev) | `npm run dev` |
| Release build | `npm run build` |
| Lint everything | `npm run lint` and `cargo clippy --workspace` |
| Format everything | `npm run format` and `cargo fmt --all` |

Toolchain note (owner's machine): Rust is `stable-x86_64-pc-windows-gnu` with
WinLibs MinGW; there is no MSVC. The default Node is 14, so always use Node 24.
