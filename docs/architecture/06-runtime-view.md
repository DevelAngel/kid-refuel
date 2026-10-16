# 6. Runtime View

## Startup

1. Load and validate configuration and initialize required resources.
2. Start the web server task.
3. Run the first collection while the web server is available.
4. If the first collection fails, initiate shutdown, stop the web server, and exit with an error.
5. If it succeeds, keep the server running and wait for collection requests from the control socket.

## Triggered collection

1. systemd activates the timer's one-shot service.
2. The service runs a lightweight trigger client that connects to the server's Unix control socket.
3. The server performs the collection and sends a success or error response.
4. The client exits successfully only when the server reports a successful collection; failures are visible to systemd and logs.

The socket unit owns the socket and passes it to the server through socket activation, avoiding a race between server startup and timer execution.


## Shutdown

On shutdown signal or fatal startup failure, stop accepting new web requests, cancel or finish managed tasks according to their contracts, and release resources cleanly.
