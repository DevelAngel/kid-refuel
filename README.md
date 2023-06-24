# refuel-rs
An example application which retrieves fuel prices and generates metrics

## useful SQLite Database commands

```sh
sqlite3 test.db .tables
sqlite3 test.db .schema
sqlite3 test.db "SELECT * FROM stations;"
sqlite3 test.db "SELECT * FROM price_changes;"
```

Show all entries from both tables:

```sh
sqlite3 test.db "SELECT s.id, p.id, s.name, s.addr, p.price, p.updated FROM stations s INNER JOIN price_changes p ON p.station_id = s.id;"
```

All price changes of one specific station:

```sh
sqlite3 test.db "SELECT * FROM price_changes WHERE station_id == 1;"
```

All price changes of statios with name MyName:

```sh
sqlite3 test.db "SELECT p.* FROM price_changes p LEFT JOIN stations s ON p.station_id = s.id WHERE s.name == 'MyName';"
```

Show all prices changes ordered by datetime (newest first):

```sh
sqlite3 test.db "SELECT * FROM price_changes ORDER BY updated DESC;"
```

Show all latest price changes for each station:

```sh
sqlite3 test.db "SELECT id,station_id,MAX(updated),price FROM price_changes GROUP BY station_id ORDER BY updated DESC;"
```

Show all latest price changes for each station including station name and addr (the second one does not show the ids):

```sh
sqlite3 test.db "SELECT id, station_id, updated, price FROM price_changes p1 WHERE updated = (SELECT MAX(updated) FROM price_changes p2 WHERE p1.station_id = p2.station_id) ORDER BY station_id;"
sqlite3 test.db "SELECT s.id, p.id, p.updated, p.price, s.name, s.addr FROM stations s INNER JOIN price_changes p ON s.id = p.station_id WHERE updated = (SELECT MAX(updated) FROM price_changes p2 WHERE p.station_id = p2.station_id) ORDER BY station_id;"
```
Note: For postgresql, it would be possible to use the "distinct on" clause to shorten the query.
