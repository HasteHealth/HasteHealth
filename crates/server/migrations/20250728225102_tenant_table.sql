CREATE TABLE
    subscription_tier (
        id TEXT NOT NULL PRIMARY KEY,
        name TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
    );

INSERT INTO
    subscription_tier (id, name)
VALUES
    ('free', 'Free');

INSERT INTO
    subscription_tier (id, name)
VALUES
    ('professional', 'Professional');

INSERT INTO
    subscription_tier (id, name)
VALUES
    ('team', 'Team');

INSERT INTO
    subscription_tier (id, name)
VALUES
    ('unlimited', 'Unlimited');

CREATE TABLE
    tenants (
        id TEXT NOT NULL PRIMARY KEY,
        deleted BOOLEAN NOT NULL DEFAULT false,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        subscription_tier TEXT NOT NULL DEFAULT 'free',
        CONSTRAINT fk_subscription_tier FOREIGN KEY (subscription_tier) REFERENCES subscription_tier (id)
    );