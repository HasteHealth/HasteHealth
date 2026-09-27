-- Replaces the single tenant-wide `tenants.index_sequence_position_v2`
-- indexing cursor with a dedicated lock table, scoped per project rather
-- than per tenant, and split per search backend.
--
-- Two problems with the old column:
-- 1. One cursor per tenant meant every project in a tenant shared the same
--    indexing progress marker, even though `get_sequence` already filters
--    by tenant only (not project) - fine while there was one lock, but it
--    blocks giving projects independent indexing progress/backpressure.
-- 2. This allows seperate search backends to be supported concurrently, without
--    them interfering with each other's progress markers.
--
-- `search_index_locks` addresses both: one row per (tenant, project,
-- backend). Workers keep using `SELECT ... FOR NO KEY UPDATE SKIP LOCKED`,
-- now against this table instead of `tenants`.
CREATE TYPE search_index_backend AS ENUM ('elasticsearch', 'postgres');

CREATE TABLE
    search_index_locks (
        tenant TEXT NOT NULL,
        project TEXT NOT NULL,
        backend search_index_backend NOT NULL,
        index_sequence_position BIGINT NOT NULL DEFAULT 0,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        PRIMARY KEY (tenant, project, backend),
        FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        FOREIGN KEY (tenant, project) REFERENCES projects (tenant, id) ON DELETE CASCADE
    );

-- Use Previously created update_modified_column() function to keep the
-- updated_at column current on changes to search_index_locks. 
CREATE TRIGGER update_modified_time BEFORE
UPDATE ON search_index_locks FOR EACH ROW EXECUTE PROCEDURE update_modified_column ();

-- Backfill: every project in production today has been indexing into
-- Elasticsearch under the tenant-wide cursor, so each project inherits its
-- tenant's current position as a starting point rather than 0. Postgres
-- Not indexed yet so fine to start at zero.
INSERT INTO
    search_index_locks (tenant, project, backend, index_sequence_position)
SELECT
    p.tenant,
    p.id,
    'elasticsearch',
    t.index_sequence_position_v2
FROM
    projects p
    JOIN tenants t ON t.id = p.tenant;

INSERT INTO
    search_index_locks (tenant, project, backend, index_sequence_position)
SELECT
    p.tenant,
    p.id,
    'postgres',
    0
FROM
    projects p;

-- `tenants.index_sequence_position_v2` is superseded as of this migration -
-- nothing reads or writes it anymore - but dropping it is left as a
-- separate, deliberate migration rather than bundled in here.

-- Keeps future projects covered without relying on every call site that
-- creates a project to remember to seed its locks.
CREATE FUNCTION create_search_index_locks_for_project () RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO search_index_locks (tenant, project, backend, index_sequence_position)
    VALUES
        (NEW.tenant, NEW.id, 'elasticsearch', 0),
        (NEW.tenant, NEW.id, 'postgres', 0);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_search_index_locks
AFTER INSERT ON projects FOR EACH ROW
EXECUTE PROCEDURE create_search_index_locks_for_project ();

