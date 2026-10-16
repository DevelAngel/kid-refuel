# 11. Risks and Technical Debt

- **Stale dependencies and toolchain compatibility:** restore a reproducible workspace build before relying on deeper test results.
- **Fragile error paths:** existing database and parsing code contains panic-prone paths that may turn bad input or unavailable services into process failures.
- **Startup coordination:** web delivery must remain available while the initial collection runs, while an initial failure still shuts down the whole application.
- **Source website changes:** scraping may fail when upstream pages change; later failures must be observable and retried.
- **Configuration gaps:** existing configuration may not expose all required operational settings.
- **Documentation drift:** update arc42 and ADRs when accepted architecture decisions change.
