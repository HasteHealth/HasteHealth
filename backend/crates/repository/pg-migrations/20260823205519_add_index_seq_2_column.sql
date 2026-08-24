-- Tracks indexing progress into r4_search_index_v2 separately from the
-- original index_sequence_position column, which tracked r4_search_index
-- (v1) and is left untouched/frozen so its position is retained while the
-- worker switches over to indexing v2 from scratch.
ALTER TABLE tenants
ADD COLUMN index_sequence_position_v2 BIGINT NOT NULL DEFAULT 0;
