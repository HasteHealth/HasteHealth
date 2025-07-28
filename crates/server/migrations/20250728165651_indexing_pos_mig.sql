CREATE TYPE lock_kind AS ENUM ('system');

CREATE TABLE
    locks (
        tenant text,
        kind lock_kind NOT NULL,
        id text NOT NULL,
        position bigint NOT NULL
    );