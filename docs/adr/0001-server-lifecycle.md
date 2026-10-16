# ADR 0001: Manage Collection and Web Delivery in One Server Lifecycle

- Status: Accepted
- Date: 2026-10-16

## Context

The existing application has separate collection commands and a web application. Running the complete application should not require an operator to coordinate independent processes. Startup must also verify that the collection pipeline can currently download, parse, and persist fuel prices.

## Decision

The server process owns web delivery and accepts collection requests over a local Unix control socket. systemd owns the schedule and triggers requests through a short-lived client, following the control-socket pattern used by kid-agent.

The web server starts while the initial collection runs, so web delivery does not wait for that first run. The initial collection must succeed for startup to be considered successful. If it fails, the application shuts down the web server and exits with an error.

After the initial success, the daemon waits for collection requests from the systemd-triggered client. The daemon performs the actual collection and returns the result to the client; the timer does not run the collector itself. Later collection failures are reported without terminating the server.

The existing crates remain in place unless implementation demonstrates a concrete need to change their boundaries. SQLite and the existing web stack remain the default choices.

## Alternatives considered

- Run collection in the timer's one-shot process: rejected because collection should share the server's configured application and persistence lifecycle.
- Let the server own its own periodic timer: rejected because systemd is the scheduling authority and operational source of truth.
- Start the daemon's control socket only after the daemon starts: rejected in favor of systemd socket activation, which avoids a startup race and keeps socket ownership with systemd.

## Consequences

- The server needs a local control protocol and a lightweight trigger command that waits for and reports the collection result.
- The systemd socket unit owns the control socket; the daemon adopts the activated file descriptor and can support direct binding for local development.
- Tests must cover concurrent web availability, initial failure shutdown, successful and failed trigger requests, and later collection failures.
- MCP integration and AI-facing access remain out of scope for the initial milestone.
