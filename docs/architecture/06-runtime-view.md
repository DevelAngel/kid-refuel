# 6. Runtime View

## Startup

1. Load and validate configuration and initialize required resources.
2. Start the web server task.
3. Run the first collection while the web server is available.
4. If the first collection fails, initiate shutdown, stop the web server, and exit with an error.
5. If it succeeds, continue with periodic collection.

## Periodic collection

1. Wait for the configured interval.
2. Download and parse current fuel prices.
3. Persist valid observations.
4. Report failures without terminating the server, then attempt collection again at the next interval.

## Shutdown

On shutdown signal or fatal startup failure, stop accepting new web requests, cancel or finish managed tasks according to their contracts, and release resources cleanly.
