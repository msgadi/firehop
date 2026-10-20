## What and why

<!-- What does this change, and why is it needed? Link the issue. -->

## Checklist

- [ ] `npm run lint`, `cargo fmt --all --check` and `cargo clippy --workspace -- -D warnings` pass
- [ ] Does not inject into, restyle or scrape WhatsApp's page
- [ ] Rules live in `crates/core`, platform glue in `src-tauri`
- [ ] Docs (`AGENTS.md`, `docs/`) updated if behaviour or architecture changed
