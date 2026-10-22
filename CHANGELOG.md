# Changelog

All notable changes to Firehop are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Firehop stays running in the notification area (tray) when you close the window, so messages and calls keep arriving. Click the tray icon to open it again; right-click → Quit Firehop to exit. The tooltip shows the unread total.

## [0.2.0] - 2026-10-21

### Added
- Total unread chats on the taskbar button (chats, as WhatsApp counts them): a Windows overlay badge in your system accent colour, like WhatsApp Desktop's own badge. Hidden at 0, "99+" past 99.
- Reorder accounts in the strip: drag and drop, Alt+↑/↓, or right-click → Move up / Move down. The order is saved, and Ctrl+1…9 follow it.

## [0.1.0] - 2026-10-20

### Added
- One window, up to 10 WhatsApp accounts, each in an isolated WebView2 profile.
- Account strip with unread badges, a total in the window title, and smooth switching.
- Account editor: photo, name, full name and colour per account.
- Voice and video calls via WhatsApp Web's browser calling (mic/camera permission handled).
- Security policy: no IPC for WhatsApp pages, navigation lock, safe external links.
- Low-memory mode for hidden accounts.
