-- Add migration script here
DROP INDEX resources_id_idx;

DROP INDEX resources_type_fitler;

CREATE INDEX resources_id_idx ON resources (tenant, project, resource_type, id, sequence DESC);

CREATE INDEX resources_type_filter ON resources (tenant, project, resource_type);