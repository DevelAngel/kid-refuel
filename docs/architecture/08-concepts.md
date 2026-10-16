# 8. Cross-cutting Concepts

## Configuration

Configuration is loaded and validated at startup. Invalid required settings should produce actionable errors rather than panics.

## Error handling

Expected operational failures are represented and propagated as errors. Startup collection failure is fatal; later trigger failures are returned to the trigger client and logged without terminating the server. A failed individual web request must not crash the server.

## Persistence

SQLite stores station information and price observations. Database access should remain centralized in the persistence layer.

## Observability

Operational events and failures should be reported through structured logging. Logs should distinguish startup, successful collection, collection failure, and shutdown.

## Testing

Tests should verify observable contracts, especially startup behavior, concurrent web availability, successful and failed control-socket requests, persistence, and web queries.
