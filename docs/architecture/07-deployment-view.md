# 7. Deployment View

The target deployment is one server process with access to a writable SQLite database and outbound network access to configured fuel-price sources. Users connect to the web server over its configured network interface.

The exact service manager, container setup, and hosting platform are deployment choices and are not prescribed by the initial milestone.

Operational configuration must make the listening address, database location, collection interval, and source-specific settings explicit and reproducible.
