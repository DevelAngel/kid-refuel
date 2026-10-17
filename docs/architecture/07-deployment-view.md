# 7. Deployment View

The target deployment consists of one long-running server process, a writable SQLite database, outbound network access to configured fuel-price sources, and systemd supervision for the server process. Users connect to the web server over its configured network interface.

The server schedules fuel-price collection internally, using a one-hour nominal interval with randomized jitter of plus or minus ten minutes. No systemd timer, trigger client, or collection control socket is required.

Operational configuration must make the listening address, database location, process supervision policy, and source-specific settings explicit and reproducible.
