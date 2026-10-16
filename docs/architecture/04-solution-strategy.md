# 4. Solution Strategy

- Keep the existing workspace crates and evolve their responsibilities incrementally.
- Let the server application own configuration, startup, task coordination, and shutdown.
- Run the web server and initial collection concurrently under the same lifecycle.
- Require the initial collection to succeed; if it fails, stop the complete application.
- Delegate scheduling to systemd; a one-shot trigger client asks the running server to perform each later collection over a Unix control socket.
- Keep persistence and shared domain operations behind the database/domain boundary.
- Verify lifecycle contracts with integration tests.
