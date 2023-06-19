PRAGMA foreign_keys = ON; -- SQlite 3.6.19 or higher

CREATE TABLE price_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    station_id INTEGER NOT NULL,
    updated TIMESTAMP NOT NULL,
    price INTEGER NOT NULL,
    FOREIGN KEY (station_id)
       REFERENCES stations (id)
       ON DELETE RESTRICT
       ON UPDATE CASCADE
);
