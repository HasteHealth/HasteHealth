-- Documents kept by the Postgres document storage backend. A document belongs
-- to a tenant and project (its scope) and has a name within them; the three
-- columns are the key, so a lookup is one primary key probe and a project's
-- documents are one range of the index. A body is split into chunks of at
-- most 1 MiB. Every put takes a new generation and its chunks are stored under
-- it, so a reader that started on one body never mixes in chunks of the body
-- that replaced it.
--
-- The key columns use the "C" collation so they sort byte by byte, which is
-- the order listings page in.
CREATE SEQUENCE document_storage_generations;

CREATE TABLE document_storage (
    tenant TEXT COLLATE "C" NOT NULL,
    project TEXT COLLATE "C" NOT NULL,
    name TEXT COLLATE "C" NOT NULL,
    generation BIGINT NOT NULL,
    size BIGINT NOT NULL,
    chunks INTEGER NOT NULL,
    content_type TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant, project, name)
);

CREATE TABLE document_storage_chunks (
    tenant TEXT COLLATE "C" NOT NULL,
    project TEXT COLLATE "C" NOT NULL,
    name TEXT COLLATE "C" NOT NULL,
    generation BIGINT NOT NULL,
    chunk INTEGER NOT NULL,
    data BYTEA NOT NULL,
    PRIMARY KEY (tenant, project, name, generation, chunk),
    CONSTRAINT document_storage_chunks_document_fkey
        FOREIGN KEY (tenant, project, name)
        REFERENCES document_storage (tenant, project, name) ON DELETE CASCADE
);

-- When the tables share the repository database, a project's documents go
-- with the project, as its resource rows do. In a database of their own there
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
