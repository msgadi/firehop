# Architecture

## Goal

Show the genuine WhatsApp Web UI for several accounts in one native window, with
an account strip on the left. Everything WhatsApp shows comes straight from
`web.whatsapp.com`, so the app matches the official client by construction. Our
code owns only the window, the strip and the glue between them.

## Runtime picture

```
┌──────────── main window (Tauri, WindowBuilder + add_child) ───────────────┐
│ ┌─ strip ─┐ ┌─ account-<id> (visible) ───────────────────────────────────┐ │
│ │ local   │ │ https://web.whatsapp.com/                                   │ │
│ │ HTML/TS │ │ WebView2 profile: <app-data>/profiles/<id>/                 │ │
│ │ IPC ✔   │ │ IPC ✘ (no capability)                                       │ │
│ └─────────┘ └─────────────────────────────────────────────────────────────┘ │
│             account-<id2>, account-<id3> … hidden, connected, low-memory   │
└────────────────────────────────────────────────────────────────────────────┘
```

- **One child webview per account**, each with its own WebView2 user-data
  folder. Cookies, IndexedDB and service workers are fully separate, as if every
  account had its own browser.
- **Hidden sessions keep running**, so messages, notifications and incoming
  calls arrive for every account. Hidden sessions get WebView2's `Low` memory
  target. The visible one gets `Normal`.
- **Switching** shows the target first, then hides the rest, so no frame is empty.

## Layers

| Layer | Crate / folder | Knows about | Must not know about |
|---|---|---|---|
| Domain | `crates/core/src/domain` | accounts, roster rules | I/O, Tauri, WebView2 |
| Application | `crates/core/src/application` | use cases, ports | Tauri, WebView2, files |
| Policy, layout | `crates/core/src/{policy,layout}.rs` | URLs, geometry | Tauri |
| Adapters | `crates/core/src/storage.rs`, `src-tauri/src/{sessions,observer,platform}.rs` | one technology each | each other |
| Interface | `src-tauri/src/commands.rs`, `ui/` | IPC | business rules |
| Composition root | `src-tauri/src/lib.rs` | everything, to wire it | — |

Ports defined by the application, implemented outside:

| Port | Implementation | Responsibility |
|---|---|---|
| `RosterStore` | `JsonRosterStore` | load and save the roster atomically |
| `Sessions` | `WebviewSessions` | open, focus, reload and destroy account webviews |
| `AvatarStore` | `FsAvatarStore` | import (decode, crop square, 128 px PNG), load and delete account photos |
| `RosterObserver` | `StripObserver` | push `RosterView` to the strip and editor, and set the window title |

## Data flow

**Command** (user clicks an account):
`strip.ts` → `backend.ts` `invoke("activate_account")` → `commands.rs` →
`AccountService::activate` → `Roster::activate` (rules) → `RosterStore::save` →
`Sessions::focus` → `RosterObserver::roster_changed` → event `roster-changed` →
`Strip.render`.

**Session event** (new message arrives in a hidden account):
WebView2 `DocumentTitleChanged` "(3) WhatsApp" → `SessionEvent::TitleChanged`
over an mpsc channel → worker thread → `AccountService::title_changed` →
`unread_from_title` → if changed, publish → badge pops in the strip, and the
window title shows the total.

The channel decouples the UI thread (where WebView2 calls back) from the service
lock, so a busy service never stalls rendering.

## Concurrency rules

- The service mutates and persists under its mutex, then **releases it before
  calling `Sessions`**. Webview operations dispatch to the UI thread and must
  never wait on our lock.
- All commands are `async`. Tauri runs them off the UI thread, because creating
  a webview from a synchronous command deadlocks on Windows.
- `Roster` changes are copy, change, save, then swap. A failed save leaves
  memory and disk unchanged.

## Security model

- WhatsApp pages have **no IPC capability**. Only the `strip` webview has one (`src-tauri/capabilities/strip.json`).
- Navigation is pinned to `https://*.whatsapp.com` / `*.whatsapp.net`, plus
  `blob:` and `about:`. Other links open in the system browser, and only
  `http(s)` and `mailto` links are handed over, so a chat message cannot launch
  local programs through custom URL schemes.
- Permissions granted automatically: microphone, camera, notifications and
  clipboard read. Everything else gets WebView2's default prompt.
- The strip page has a strict CSP.

## Performance techniques

- WebView2 flags borrowed from the official client:
  `--js-flags=--scavenger_max_new_space_capacity_mb=8` and
  `--msWebView2SimulateMemoryPressureWhenInactive=1`.
- `MemoryUsageTargetLevel::Low` for hidden sessions.
- A background colour on every webview, so there is never a white flash.
- The strip updates keyed DOM rows in place, so CSS transitions run on the
  compositor and nothing is rebuilt.
- Release profile: LTO, `codegen-units = 1`, `panic = "abort"`, stripped symbols.
