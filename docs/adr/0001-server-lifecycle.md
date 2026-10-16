# ADR 0001: Manage Collection and Web Delivery in One Server Lifecycle

- Status: Accepted
- Date: 2026-10-16

## Context

The existing application has separate collection commands and a web application. Running the complete application should not require an operator to coordinate independent processes. Startup must also verify that the collection pipeline can currently download, parse, and persist fuel prices.

## Decision

The server process manages the web application and periodic collection concurrently.

The web server is started while the initial collection is running, so web delivery does not wait for the first collection to finish. The first collection must succeed for startup to be considered successful. If it fails, the application shuts down the web server and exits with an error.

After a successful initial collection, collection repeats at a configured interval. Later collection failures are reported but do not terminate the server; the next scheduled attempt can recover.

The existing crates remain in place unless implementation demonstrates a concrete need to change their boundaries. SQLite and the existing web stack remain the default choices.

## Alternatives considered

- Run collection before starting the web server: rejected because the web application would be unavailable during initial collection.
- Treat initial collection failure like any later failure: rejected because startup would not validate the current collection pipeline.
- Split collection and web delivery into separately operated services: rejected because the target is one manageable server lifecycle.

## Consequences

- Startup orchestration must coordinate the initial collection result and server shutdown.
- Tests must verify concurrent web availability, initial failure shutdown, and recovery after later collection failures.
- Collection interval and other operational settings must be configurable.
- MCP integration and AI-facing access remain out of scope for the initial milestone.
