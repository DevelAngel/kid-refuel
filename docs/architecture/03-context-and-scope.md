# 3. Context and Scope

## System responsibility

kid-refuel collects prices from configured fuel-price sources, persists valid observations, and serves a web interface for current prices and station history.

## External dependencies

- Fuel-price source websites provide the input data.
- Operators provide configuration and choose where the server runs.
- Users access the web application through a browser.
- The server uses a local SQLite database for durable data.

## Out of scope

AI/MCP access, a new hosting platform, unrelated product features, and a database migration are not part of the initial server transformation.
