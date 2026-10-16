# 7. Deployment View

The target deployment consists of one long-running server process, a writable SQLite database, outbound network access to configured fuel-price sources, and systemd units for service and timer management. Users connect to the web server over its configured network interface.

A systemd socket unit owns the Unix control socket and passes it to the server through socket activation. A timer activates a one-shot service that runs the trigger client; the client sends a request and waits for the server's collection result.

Operational configuration must make the listening address, database location, control-socket path, systemd schedule, and source-specific settings explicit and reproducible.
