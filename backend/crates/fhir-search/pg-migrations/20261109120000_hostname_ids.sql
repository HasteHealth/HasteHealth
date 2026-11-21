-- Tenant and project ids are hostname labels (lowercase letters and digits
-- joined by single hyphens); the repository migration of the same name
-- rewrites ids created before that rule. The index rows of those tenants and
-- projects are rewritten with the same expression here, so they keep matching
-- the repository's rows. A Project resource is indexed under its id, which is
-- the project id, so it is rewritten too. Hand-written; a data change only.
UPDATE r4_resource_idx
SET tenant = trim(both '-' from regexp_replace(lower(tenant), '[^a-z0-9]+', '-', 'g')),
    project = trim(both '-' from regexp_replace(lower(project), '[^a-z0-9]+', '-', 'g')),
    resource_id = CASE
        WHEN resource_type = 'Project'
        THEN trim(both '-' from regexp_replace(lower(resource_id), '[^a-z0-9]+', '-', 'g'))
        ELSE resource_id
    END
WHERE tenant !~ '^[a-z0-9]+(-[a-z0-9]+)*$' OR length(tenant) > 63
   OR project !~ '^[a-z0-9]+(-[a-z0-9]+)*$' OR length(project) > 63
   OR (resource_type = 'Project'
       AND (resource_id !~ '^[a-z0-9]+(-[a-z0-9]+)*$' OR length(resource_id) > 63));
