-- Documents kept by the Postgres document storage backend. A body is split
-- into chunks of at most 1 MiB. Every put takes a new generation and its
-- chunks are stored under it, so a reader that started on one body never
-- mixes in chunks of the body that replaced it.
--
-- Keys use the "C" collation so they sort byte by byte, which is the order
-- listings page in.
CREATE SEQUENCE document_storage_generations;

CREATE TABLE document_storage (
    key TEXT COLLATE "C" PRIMARY KEY,
    generation BIGINT NOT NULL,
    size BIGINT NOT NULL,
    chunks INTEGER NOT NULL,
    content_type TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE document_storage_chunks (
    key TEXT COLLATE "C" NOT NULL,
    generation BIGINT NOT NULL,
    chunk INTEGER NOT NULL,
    data BYTEA NOT NULL,
    PRIMARY KEY (key, generation, chunk)
);
