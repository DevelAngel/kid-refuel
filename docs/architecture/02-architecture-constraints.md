# 2. Architecture Constraints

- The application is implemented in Rust and built as a Cargo workspace.
- The existing Leptos and Axum web stack is retained unless a concrete technical need justifies replacement.
- SQLite remains the persistence technology for this milestone.
- Architecture documentation is maintained as Markdown in this repository.
- The server owns periodic collection scheduling; systemd supervises the server process.
- MCP integration is explicitly outside the initial milestone.
