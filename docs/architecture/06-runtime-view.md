# 6. Runtime View

## Startup

1. Load and validate configuration and initialize required resources.
2. Start the web server task.
3. Run the first collection while the web server is available.
4. If the first collection fails, initiate shutdown, stop the web server, and exit with an error.
5. If it succeeds, schedule collections internally, waiting 50 to 70 minutes after each completed run.

## Scheduled collection

1. After a collection completes, choose a fresh randomized wait between 50 and 70 minutes.
2. Run the next collection after the wait; do not overlap collection runs.
3. Log collection failures and continue with the next randomized wait.
4. On shutdown, stop the scheduler and cancel and await any in-progress collection.



## Shutdown

On shutdown signal or fatal startup failure, stop accepting new web requests, cancel or finish managed tasks according to their contracts, and release resources cleanly.
