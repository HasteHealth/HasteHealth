-- Every reader of the write sequence scopes by tenant AND project: the
-- indexing worker's poll (`get_sequence`, one cursor per project since
-- 20260927174547_project_scoped_search_index_locks) and `_history` at the
-- system, type and instance level. The old index led with tenant only, so
-- a poll for a project that is caught up walked every sibling project's
-- entries above its cursor and discarded them on the heap (`project` was
-- not in the index). With one project 1M rows ahead of a sibling in the
-- same tenant, each poll of the sibling read ~805k buffers for zero rows,
-- ten times a second; with (tenant, project, sequence) it reads 6.
--
-- (tenant, project, sequence) serves every query (tenant, sequence) did,
-- so the old index is dropped rather than kept beside it.
CREATE INDEX resources_project_sequence_idx ON resources (tenant, project, sequence);

DROP INDEX sequence_asc_index;
