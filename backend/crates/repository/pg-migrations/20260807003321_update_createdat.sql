ALTER TABLE resources RENAME COLUMN created_at TO db_insert_timestamp;

-- text -> timestamptz casts are only STABLE in Postgres (parsing can depend on the
-- session's TimeZone setting), so they are rejected in a GENERATED column expression.
-- meta.lastUpdated is always written as UTC with an explicit offset
-- (see repository::utilities::set_resource_meta), so this wrapper is safe to mark IMMUTABLE.
CREATE FUNCTION immutable_timestamptz(text) RETURNS timestamp with time zone AS $$
    SELECT $1::timestamp with time zone
$$ LANGUAGE sql IMMUTABLE STRICT PARALLEL SAFE;

ALTER TABLE resources
ADD COLUMN created_at timestamp with time zone GENERATED ALWAYS AS (immutable_timestamptz((resource -> 'meta'::text) ->> 'lastUpdated'::text)) STORED NOT NULL;