# 8. Cross-cutting Concepts

## Configuration

Configuration is loaded and validated at startup. Invalid required settings should produce actionable errors rather than panics.

## Error handling

Expected operational failures are represented and propagated as errors. Startup collection failure is fatal; later collection failures are logged and retried. A failed individual web request must not crash the server.

## Persistence

SQLite stores station information and price observations. Database access should remain centralized in the persistence layer.

## Observability

Operational events and failures should be reported through structured logging. Logs should distinguish startup, successful collection, collection failure, and shutdown.

## Testing

Tests should verify observable contracts, especially startup behavior, concurrent web availability, periodic retry behavior, persistence, and web queries.
