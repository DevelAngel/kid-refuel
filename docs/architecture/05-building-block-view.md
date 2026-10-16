# 5. Building Block View

The current workspace contains these primary building blocks:

- **refuel-server** — application entry point, collection workflow, and Axum/Leptos server integration.
- **refuel-db** — SQLite connection setup, schema, persistence, and database queries.
- **refuel-app** — Leptos UI and server-side application functions.
- **refuel-front** — WebAssembly frontend hydration.
- **sim** — supporting simulation code; its production role must be confirmed during implementation.

Target responsibility boundaries:

- The server owns the process lifecycle and coordinates long-running tasks.
- Collection obtains, parses, validates, and persists price observations.
- Database operations own storage-specific behavior and expose shared domain operations.
- The web application presents current prices and station history without owning process lifecycle.
