CREATE TABLE price_changes (
    name VARCHAR NOT NULL,
    addr VARCHAR NOT NULL,
    updated TIMESTAMP NOT NULL,
    price INTEGER NOT NULL,
    PRIMARY KEY (name, addr, updated)
)
