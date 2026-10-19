-- Queue and status store for asynchronous jobs: work a request kicks off and
-- a worker finishes later, which the client follows through the FHIR
-- Asynchronous Request Pattern (202 Accepted + Content-Location, then polling).
-- The first consumer is an operation invoked with `Prefer: respond-async`;
-- Bulk Data `$export` is the next.
CREATE TYPE async_job_status AS ENUM (
    -- Queued, not yet picked up.
    'accepted',
    -- Claimed by a worker and running (or abandoned, until its lease lapses).
    'in-progress',
    -- Finished with a result in `output`.
    'completed',
    -- Finished with an OperationOutcome in `error`.
    'failed',
    -- Cancelled by the client (DELETE on the status URL), or expired.
    'cancelled'
);

CREATE TABLE
    async_jobs (
        -- Opaque id; it is the last segment of the status URL the client polls.
        id TEXT NOT NULL PRIMARY KEY,
        tenant TEXT NOT NULL,
        project TEXT NOT NULL,
        -- Which handler runs the job (`operation`, later `export`). Text rather
        -- than an enum so a new kind is a code change only.
        kind TEXT NOT NULL,
        status async_job_status NOT NULL DEFAULT 'accepted',
        -- The kick-off request, as the client sent it (method, URL and
        -- the parameters that mattered). Echoed in a Bulk Data manifest.
        request TEXT NOT NULL,
        -- Handler-specific input, as the kick-off recorded it.
        parameters JSONB NOT NULL DEFAULT '{}'::jsonb,
        -- The verified token claims of the submitter. The worker rebuilds the
        -- request context from these, so the job runs with exactly the
        -- scopes and access policies the kick-off had. Never the token.
        claims JSONB NOT NULL,
        -- `claims.sub`, kept as a column so ownership checks and listings
        -- do not have to open the JSON.
        submitted_by TEXT NOT NULL,
        -- Free text from the handler, returned in `X-Progress` while running.
        progress TEXT,
        -- The result once completed, and how to serve it (a FHIR resource as
        -- `application/fhir+json`, a Bulk Data manifest as `application/json`).
        output JSONB,
        output_content_type TEXT,
        -- An OperationOutcome once failed.
        error JSONB,
        -- How many times a worker has claimed the job. A claim past
        -- `max_attempts` is refused and the job is failed instead, so a job
        -- that crashes its worker every time does not run forever.
        attempts INTEGER NOT NULL DEFAULT 0,
        max_attempts INTEGER NOT NULL DEFAULT 3,
        -- The lease: who holds the job and until when. Both NULL unless
        -- in-progress.
        locked_by TEXT,
        lease_expires_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        started_at TIMESTAMPTZ,
        finished_at TIMESTAMPTZ,
        -- When the row (and whatever the handler produced for it) is removed.
        -- Set at kick-off and again at completion, so a job that never ran is
        -- cleared too.
        expires_at TIMESTAMPTZ NOT NULL,
        FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        FOREIGN KEY (tenant, project) REFERENCES projects (tenant, id) ON DELETE CASCADE
    );

-- The claim scan: the oldest queued job, or the oldest in-progress job whose
-- lease has lapsed. Finished rows are the bulk of the table over time and are
-- never claimed, so they are kept out of the index.
CREATE INDEX async_jobs_claim_idx ON async_jobs (created_at)
WHERE
    status IN ('accepted', 'in-progress');

-- Ownership checks, listings and the per-project cap on live jobs.
CREATE INDEX async_jobs_project_idx ON async_jobs (tenant, project, created_at DESC);

-- The expiry sweep.
CREATE INDEX async_jobs_expires_at_idx ON async_jobs (expires_at);

-- Reuse the trigger function from `subscription_topic_locks` to keep
-- `updated_at` current.
CREATE TRIGGER update_modified_time BEFORE
UPDATE ON async_jobs FOR EACH ROW EXECUTE PROCEDURE update_modified_column ();
