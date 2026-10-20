# 0001: Tauri 2 + WebView2 for the shell

- Status: accepted (October 2026)

## Context

The official WhatsApp Desktop renders its whole UI with WhatsApp Web inside
WebView2 (see `docs/research/whatsapp-desktop-teardown.md`). We want the same
UI, several accounts, a small footprint and smooth switching.

## Options

| Option | Verdict |
|---|---|
| **Tauri 2 (Rust) + WebView2** | Same engine as the official app, ~10 MB binary, Rust core, multi-webview windows, per-webview data directories. **Chosen.** |
| C# WinUI 3 + WebView2 | Same stack WhatsApp uses and very native, but Windows-only, with a heavier runtime and harder open-source contribution. Kept as a fallback if Tauri blocks a must-have feature. |
| Electron | Bundles its own Chromium (~150 MB+, more RAM per account). No advantage here. |
| Native Go/Rust UI (Fyne, Gio, Slint, egui, iced) | Cannot run WhatsApp Web. Would require re-implementing the protocol, which means feature lag, ban risk and no calls. Rejected. |

## Consequences

- Needs Tauri's `unstable` feature for `Window::add_child`.
- Each account uses its own WebView2 environment, so its own browser process
  (roughly 50–100 MB). The ten-account limit keeps this bounded.
- Raw WebView2 COM is needed for things Tauri lacks (memory target level). It is
  confined to `src-tauri/src/platform.rs`.
