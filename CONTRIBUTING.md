# Contributing to Firehop

Thanks for helping. Firehop has one promise: **the real WhatsApp Web, untouched,
for many accounts, buttery smooth.** Every change should keep that promise.

## Before you start

1. Read [AGENTS.md](AGENTS.md). It is the project rulebook (product rules,
   architecture, conventions) for humans and AI agents alike.
2. For anything bigger than a bug fix, **open an issue first** so we can agree
   on the approach before you write code.

## Ground rules

- **Never** inject scripts into, restyle or scrape WhatsApp's page, and never
  imitate the official client. That keeps users safe from bans and the app safe
  from WhatsApp updates. See
  [ADR 0003](docs/decisions/0003-no-official-client-impersonation.md).
- No WhatsApp or Meta assets (logos, fonts, emoji) in the repo.
- Business rules belong in `crates/core` (no Tauri there). Platform glue belongs
  in `src-tauri`. UI belongs in `ui/`.

## Setup

```sh
npm install
npm run dev
```

Requires Rust stable and Node 24 (`.node-version`; `fnm use` or `nvm use` picks it up).

## Before you open a pull request

```sh
npm run lint                          # Biome + TypeScript
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

CI runs the same checks. Keep pull requests focused, and describe the *why* in the description.

## Commit messages

Imperative and specific: `Show account photo in strip tooltip`, not `fix stuff`.
