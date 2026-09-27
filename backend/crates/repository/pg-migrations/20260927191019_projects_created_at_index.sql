-- The indexing worker now polls `projects` (via get_projects) instead of
-- `tenants` to drive per-project locking, using
-- `WHERE created_at > $1 ORDER BY created_at.
CREATE INDEX projects_created_at_idx ON projects (created_at);
