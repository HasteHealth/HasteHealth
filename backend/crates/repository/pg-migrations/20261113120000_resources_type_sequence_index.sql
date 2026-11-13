-- Type-level `_history` (`history`, `GET [base]/<Type>/_history`) filters on
-- (tenant, project, resource_type) and orders by sequence DESC. No index
-- covered both: resources_type_filter stopped at resource_type and
-- resources_id_idx puts id before sequence, so the planner walked
-- resources_project_sequence_idx backwards and discarded every other type's
-- rows on the heap until it had `_count` matches. For a type whose newest
-- versions are old (an Organization or ValueSet load followed by many writes
-- of other types) that cost grows with the project: 4,000 Organization
-- versions under 396,000 newer rows took 147.6 ms (26,340 buffers), 322 ms at
-- 800,000 rows. With (tenant, project, resource_type, sequence DESC) it is
-- 0.14 ms and 10 buffers at either size.
--
-- (tenant, project, resource_type) is a prefix of the new index, so it serves
-- every query resources_type_filter did, and that index is dropped rather than
-- kept beside it: every write still maintains the same number of indexes.
CREATE INDEX resources_type_sequence_idx ON resources (tenant, project, resource_type, sequence DESC);

DROP INDEX resources_type_filter;
