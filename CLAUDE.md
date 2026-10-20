# CLAUDE.md

@AGENTS.md

All project instructions live in `AGENTS.md` (imported above), so every agent
works from one source of truth.

Claude-specific reminders:
- When the owner gives a new instruction, append it to "Owner instructions" in
  `AGENTS.md` in the same turn.
- Run npm through Node 24: `eval "$(fnm env --shell bash)" && fnm use 24` (Bash)
  or `fnm exec --using=24 -- <cmd>` (PowerShell).
- Never write tests (owner instruction 7).
- Propose and discuss new features before building them (owner instruction 5).
