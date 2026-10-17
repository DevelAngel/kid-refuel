# 4. Solution Strategy

- Keep the existing workspace crates and evolve their responsibilities incrementally.
- Let the server application own configuration, startup, task coordination, and shutdown.
- Run the web server and initial collection concurrently under the same lifecycle.
- Require the initial collection to succeed; if it fails, stop the complete application.
- Schedule collections inside the server with a one-hour interval and randomized jitter of plus or minus ten minutes; continue the schedule after later failures.
- Keep persistence and shared domain operations behind the database/domain boundary.
- Verify lifecycle contracts with integration tests.
