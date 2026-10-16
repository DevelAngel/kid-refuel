UPDATE price_changes
SET station_id = (
    SELECT MIN(canonical.id)
    FROM stations AS duplicate
    JOIN stations AS canonical
      ON canonical.name = duplicate.name
     AND canonical.addr = duplicate.addr
    WHERE duplicate.id = price_changes.station_id
)
WHERE station_id IN (
    SELECT duplicate.id
    FROM stations AS duplicate
    JOIN stations AS canonical
      ON canonical.name = duplicate.name
     AND canonical.addr = duplicate.addr
     AND canonical.id < duplicate.id
);

DELETE FROM price_changes
WHERE id NOT IN (
    SELECT MIN(id)
    FROM price_changes
    GROUP BY station_id, updated
);

DELETE FROM stations
WHERE id NOT IN (
    SELECT MIN(id)
    FROM stations
    GROUP BY name, addr
);

CREATE UNIQUE INDEX stations_name_addr_unique
    ON stations (name, addr);

CREATE UNIQUE INDEX price_changes_station_updated_unique
    ON price_changes (station_id, updated);
