# How the official WhatsApp Desktop for Windows is built

Read-only analysis of the installed Microsoft Store package
`5319275A.WhatsAppDesktop` **2.2639.100.0** (October 2026), from file listings,
the package manifest and strings embedded in its binaries. Nothing was modified,
patched or redistributed. The WebView2 profile (which holds session keys) was
deliberately not inspected.

## Stack

| Piece | Evidence | Role |
|---|---|---|
| .NET 8 + WinUI 3 (C#) | `WhatsApp.Root.runtimeconfig.json` (net8.0), `Microsoft.WinUI.dll` | Native shell and window |
| WebView2 | `Microsoft.Web.WebView2.Core.dll`, manifest dependency `Microsoft.WebView2` | Renders the whole chat UI |
| WhatsApp Web, hybrid mode | `https://web.whatsapp.com/spin?subplatform=win_hybrid&revision=` | The actual app UI |
| Native VoIP (C++) | `WhatsAppNative.Voip.dll` (12 MB), `WhatsApp.VoIP.dll` | Calling engine and native call windows |
| Rust | `WhatsAppRust.dll` (`libwamediastreams-rs`, `libwamediadetection-rs`, `wa_log-rs`) | Media parsing and streams |
| ONNX Runtime + DirectML | `onnxruntime.dll`, `DirectML.dll` | On-device ML (probably call effects) |
| XMPP/"Smax" protocol code | `WhatsApp.Networking.dll` (generated handlers: groups, prekeys, blocklist, bots) | Native protocol pieces |
| Chrome extensions inside WebView2 | `Extensions/CodeVerify`, `Extensions/Zoom` | Code integrity checks; zoom steps 0.8–1.5 |

## The JS ↔ native bridge surface

WinRT host objects (`WinRTAdapter.winmd`) exposed to the web app, about 30 in total:
`VoipBridge`, `VoipSignalingBridge`, `NotificationsBridge` (via
`SystemIntegrationsBridge`), `SQLiteBridge`, `MediaFilesBridge`,
`MediaTranscodingBridge`, `ContactsBridge`, `PicturesBridge`,
`PreferencesBridge`, `SharesheetBridge`, `RingtoneBridge`, `ClientKeyBridge`,
`TcTokenBridge`, `AbPropsBridge`, `MetaConfigBridge`, `WamBridge`, `OdsBridge`,
`LifecycleBridge`, `AppActivationBridge`, `ScalingControlBridge`,
`TouchpadFixBridge`, `WebUpdateBridge`, `SeamlessMigrationBridge`, and others.

Examples of the methods they expose: `ShowMessageNotification`,
`UpdateTaskbarBadge`, `UpdateStartupTask`, `StartCall`, `StartGroupCall`,
`PreviewAndJoinCallLink`, `ExecuteSqlite`, `TrySaveMediaFileFromSharedBuffer`,
`PerformVideoTranscodingFromSharedBuffer`, `SetHideMyIp`, `ZoomIn` / `ZoomOut`.

**These bridges belong to the official client. We do not imitate them** (see
ADR 0003). Plain WhatsApp Web provides the same features through standard
browser APIs.

## OS integration declared in the manifest

- Protocols: `whatsapp:`, `tel:`, `im:`, `ms-ipmessaging:`, `ms-voip-call:`, `ms-voip-video:`
- Share target ("Share files to WhatsApp")
- Startup task (launch at login)
- Toast notification activation (COM activator)
- Contacts capability; jump list ("New chat", "New call")

These are the native features we re-create ourselves, one at a time, as agreed with the owner.

## WebView2 tuning we reuse

- `--js-flags=--scavenger_max_new_space_capacity_mb=8`
- `--msWebView2SimulateMemoryPressureWhenInactive=1`
- `--edge-webview-is-background` when starting in the tray
- Lower the memory usage target level while inactive; suspend and resume around app visibility

## Per-account storage

`LocalState\sessions\<hash>\` holds `contacts.db`, `nativeSettings.db`,
`mediaDownloads.db`, `abprops.db`, `metaconfig.db` and others. The layout is
per-account, but the UI exposes only one account.

## Consequences for Firehop

1. The chat UI is WhatsApp Web, so wrapping web.whatsapp.com gives the same UI
   and features without re-implementing anything.
2. The only visible differences are the native call window and Desktop-only
   settings such as "Start at login". We re-create the OS integrations natively.
3. WebView2 is the right engine: it is literally what the official app uses.
