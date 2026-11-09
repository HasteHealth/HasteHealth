-- Documents kept by the Postgres document storage backend. A document is one
-- row: it belongs to a tenant and project (its scope) and has a name within
-- them, and the three columns are the key, so a lookup is one primary key
-- probe and a project's documents are one range of the index. The body is a
-- bytea that Postgres keeps out of line (TOAST), so the row itself stays small
-- for a head or a listing.
--
-- The key columns use the "C" collation so they sort byte by byte, which is
-- the order listings page in.
CREATE TABLE document_storage (
    tenant TEXT COLLATE "C" NOT NULL,
    project TEXT COLLATE "C" NOT NULL,
    name TEXT COLLATE "C" NOT NULL,
    content_type TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    body BYTEA NOT NULL,
    PRIMARY KEY (tenant, project, name)
);

-- When the table shares the repository database, a project's documents go
-- with the project, as its resource rows do. In a database of its own there
-- is no projects table to point at, and the sweep removes them instead.
DO $$
BEGIN
    IF to_regclass('projects') IS NOT NULL THEN
        ALTER TABLE document_storage
            ADD CONSTRAINT document_storage_project_fkey
            FOREIGN KEY (tenant, project)
            REFERENCES projects (tenant, id) ON DELETE CASCADE;
    END IF;
END
$$;
