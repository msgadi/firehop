# Security policy

Firehop hosts other people's private conversations, so we take security seriously.

## Reporting a vulnerability

**Please do not open a public issue.** Report privately through
[GitHub Security Advisories](https://github.com/msgadi/firehop/security/advisories/new).
Include steps to reproduce and the impact you expect. You will get a reply as
soon as possible, and credit in the fix unless you prefer otherwise.

## What counts

Anything that breaks the security model in [docs/architecture.md](docs/architecture.md#security-model), for example:

- WhatsApp's page (or any remote content) reaching Firehop's IPC commands
- One account's session or data leaking into another account
- A link or message that makes Firehop open local files or programs
- Permission grants beyond microphone, camera, notifications and clipboard read

## Out of scope

- WhatsApp's own web client and servers: report those to
  [Meta's bug bounty](https://bugbounty.meta.com/).
- Account restrictions applied by WhatsApp (see the README disclaimer).
