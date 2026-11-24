# Contributor guidance

- Use American English for code, documentation, and UI.
- Keep source files under 600 lines.
- Preserve original source images: editing and export operations are non-destructive.
- Deterministic processing: the same source bytes and settings must produce identical outputs.
- Avoid AI services and external asset uploads; user images remain local.
- Add regression tests for changes to atlas geometry, trimming, and exported metadata.
- Keep the core library independent of UI and engine integrations.
- Check `AGENTS.md` and `CLAUDE.md` before changing this repository.
