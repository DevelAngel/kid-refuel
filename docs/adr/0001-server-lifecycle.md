# ADR 0001: Manage Collection and Web Delivery in One Server Lifecycle

- Status: Accepted
- Date: 2026-10-16

## Context

The existing application has separate collection commands and a web application. Running the complete application should not require an operator to coordinate independent processes. Startup must also verify that the collection pipeline can currently download, parse, and persist fuel prices.

## Decision


The server process owns both web delivery and periodic fuel-price collection. It starts the web server task before running the initial collection, so web delivery remains available during startup. The initial collection must succeed for startup to be considered successful; if it fails, the application shuts down the web server and exits with an error.

After the initial success, the server schedules collections internally. Each subsequent wait begins after the previous collection finishes. The nominal interval is one hour with a newly randomized jitter of plus or minus ten minutes, producing a wait between 50 and 70 minutes. A failed scheduled collection is logged but does not terminate the server; the next attempt occurs after the next normal randomized wait. Collections never overlap.

A shutdown signal stops the scheduler and gracefully shuts down the web server. If a collection is in progress, shutdown cancels that task and awaits its termination.

The existing crates remain in place unless implementation demonstrates a concrete need to change their boundaries. SQLite and the existing web stack remain the default choices.

## Alternatives considered

- Run collection in a timer's one-shot process: rejected because collection should share the server's configured application and persistence lifecycle.
- Let systemd own each collection time: rejected because this application does not need immediate event-driven retries, and an internal schedule avoids an extra trigger client and control socket.
- Retry a failed collection immediately: rejected because the next regular attempt is sufficient.

## Consequences

- The server owns collection timing and randomizes each wait to avoid a rigid polling cadence.
- systemd remains responsible for process supervision and restart policy, not collection scheduling.
- Tests cover jitter bounds, continued scheduling after failures, non-overlapping runs, and shutdown.
- MCP integration and AI-facing access remain out of scope for the initial milestone.
