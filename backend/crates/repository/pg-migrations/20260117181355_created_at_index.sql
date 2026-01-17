-- Add migration script here
CREATE INDEX resources_created_at_idx on resources (tenant, project, created_at);