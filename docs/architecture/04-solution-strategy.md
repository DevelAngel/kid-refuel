# 4. Solution Strategy

- Keep the existing workspace crates and evolve their responsibilities incrementally.
- Let the server application own configuration, startup, task coordination, and shutdown.
- Run the web server and collector concurrently under the same lifecycle.
- Require the initial collection to succeed; if it fails, stop the complete application.
- Continue periodic collection after startup. Log later failures and retry at the next scheduled interval.
- Keep persistence and shared domain operations behind the database/domain boundary.
- Verify lifecycle contracts with integration tests.
