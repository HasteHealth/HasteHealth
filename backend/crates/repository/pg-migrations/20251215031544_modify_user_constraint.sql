DROP INDEX owner_unique_idx;

CREATE UNIQUE INDEX owner_unique_idx ON users USING btree (email)
WHERE
    role = 'owner';