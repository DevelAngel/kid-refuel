# 2. Architecture Constraints

- The application is implemented in Rust and built as a Cargo workspace.
- The existing Leptos and Axum web stack is retained unless a concrete technical need justifies replacement.
- SQLite remains the persistence technology for this milestone.
- Architecture documentation is maintained as Markdown in this repository.
- systemd owns scheduled collection timing; the server exposes a local control socket for trigger requests.
- MCP integration is explicitly outside the initial milestone.
