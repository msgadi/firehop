# 0003: Never impersonate the official client

- Status: accepted (October 2026)

## Context

The official app loads `web.whatsapp.com/spin?subplatform=win_hybrid` and gives
the page about 30 native bridges (VoIP, SQLite, notifications, etc.). Imitating
them could unlock native-only behaviour.

## Decision

We load plain `https://web.whatsapp.com/` with WebView2's own user agent. We do
not set hybrid subplatforms, spoof bridges, inject scripts into WhatsApp's page
or modify its traffic.

## Why

- WhatsApp's Terms forbid it, and accounts using modified or unofficial clients get banned.
- Plain WhatsApp Web already provides chats, status, channels, communities and,
  since July 2026, voice and video calls in the browser.
- No injected scripts means nothing breaks when WhatsApp ships an update.
