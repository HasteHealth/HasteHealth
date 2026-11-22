-- `_history?_since=` filters on resources.created_at, but 20260807003321
-- renamed that column to db_insert_timestamp and the index followed the
-- rename; nothing reads db_insert_timestamp. A selective `_since` therefore
-- scanned the whole table (69 ms at 400,000 rows vs 0.1 ms indexed).
-- Re-point the index at created_at; the index count per write is unchanged.
DROP INDEX resources_created_at_idx;

CREATE INDEX resources_created_at_idx ON resources (tenant, project, created_at);
