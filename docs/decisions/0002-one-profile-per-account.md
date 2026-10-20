# 0002: One WebView2 profile directory per account

- Status: accepted (October 2026)

## Context

WhatsApp Web keeps its login in IndexedDB, local storage and service workers.
Two accounts in the same browser profile would overwrite each other.

## Decision

Every account webview gets `data_directory(<app-data>/profiles/<account-id>)`.
The account id is a UUID that never changes, so the directory survives renames.
Removing an account closes its webview and deletes its directory (retrying
while the browser process releases its file locks). That logs the linked device out.

## Alternatives

- WebView2 *profiles* inside one user-data folder (`ProfileName`): fewer browser
  processes, but Tauri does not expose it yet. We can revisit this to save memory later.
- Incognito webviews: the login would be lost on every restart. Rejected.
