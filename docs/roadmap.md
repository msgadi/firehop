# Roadmap

Nothing below gets built until the owner has agreed to it (owner instruction 5).

## Prototype (done)

- Strip with account switching, add, remove and reload
- Isolated WebView2 profile per account
- Unread badges from page titles, plus the total in the window title
- Permissions for calls, a navigation lock and external links

## Parity with the official Desktop app (proposed)

| Feature | Official app | Approach | Status |
|---|---|---|---|
| Native toast per account, with sender and account name | `ShowMessageNotification` bridge | WebView2 `NotificationReceived` → Windows toast tagged with the account | proposed |
| Taskbar unread badge | `UpdateTaskbarBadge` | `ITaskbarList3::SetOverlayIcon` with the count | proposed |
| Tray icon + close-to-tray | `TrayIconManager` | Tauri tray API | proposed |
| Start at login | `UpdateStartupTask` | `tauri-plugin-autostart` | proposed |
| `whatsapp://` and `wa.me` links | protocol registration | deep-link plugin → ask which account | proposed |
| "Share to WhatsApp" from Explorer | share target | Windows share target → pick account | research |
| Jump list: New chat / New call | `JumpListManager` | `ICustomDestinationList` | research |
| Zoom steps 80–150% | Zoom extension | WebView2 zoom factor, remembered per account | proposed |
| Spell check | WebView2 | on by default; check languages | verify |
| Screen sharing in calls | native VoIP | `getDisplayMedia` in WebView2; verify the picker | verify |

## Beyond the official app (ideas to discuss)

- ~~Rename and recolour accounts~~ (done: account editor with photo, name, full name, colour)
- Drag to reorder accounts
- Use the account's WhatsApp profile photo automatically (needs reading the page; conflicts with ADR 0003)
- Per-account notification mute and Do Not Disturb schedule
- Show account names in the strip (an expanded mode)
- Lock the app with Windows Hello
- Memory dashboard per account, and auto-sleep for accounts idle a long time
- Global shortcut to bring the app forward
- Portable mode (data next to the exe)
- Auto-update via GitHub Releases (`tauri-plugin-updater`)
- Accent colour that follows WhatsApp's own light or dark theme

## Open questions for the owner

1. Name decided: "Firehop" (`io.github.firehop.desktop`).
2. License: MIT (current placeholder), Apache-2.0 or GPL-3.0?
3. Account-switch shortcut while a chat has focus. WhatsApp uses Ctrl+1…9 for
   chats; Ctrl+Alt+1…9 or Alt+1…9 are free.
4. When the last account is removed, should we link a fresh one (current
   behaviour) or show an empty state?
5. Windows only, or macOS and Linux too? (Linux needs WebKitGTK, and WhatsApp
   Web calling support there is unverified.)
