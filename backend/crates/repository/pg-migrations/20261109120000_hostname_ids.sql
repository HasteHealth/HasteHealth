-- Tenant and project ids are hostname labels: lowercase letters and digits
-- joined by single hyphens, at most 63 characters (haste_jwt::HOSTNAME_ID_PATTERN).
-- TenantId and ProjectId enforce that rule when they are built, so a row that
-- holds anything else can no longer be read. Ids created before the rule are
-- rewritten to it here: lowercased, every run of other characters replaced by
-- one hyphen, hyphens at either end dropped. The rows that reference an id
-- follow it through their foreign keys, and the CHECK constraints added at the
-- end keep the rule in the schema from now on. The search index database
-- rewrites its rows with the same expression in its own migration. A document
-- store on a filesystem or in a bucket is not touched: it ships in the same
-- release as this migration, so it holds nothing under an old id yet.
--
-- Nothing happens in a database whose ids all conform. Otherwise the rewrite is
-- one transaction, and it fails, leaving everything as it was, when a rewritten
-- id collides with an existing one (duplicate key) or comes out empty (CHECK);
-- the operator renames that tenant or project by hand and migrates again.
-- Users of a renamed tenant sign in again: their sessions and tokens name the
-- old id, and so does any URL they had, since the id is part of the hostname
-- and the path.
DO $$
DECLARE
    pattern CONSTANT text := '^[a-z0-9]+(-[a-z0-9]+)*$';
    fk RECORD;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM tenants WHERE id !~ pattern OR length(id) > 63)
       AND NOT EXISTS (SELECT 1 FROM projects WHERE id !~ pattern OR length(id) > 63) THEN
        RETURN;
    END IF;

    -- A renamed id has to reach every row that references it. The foreign keys
    -- onto tenants, projects and the other tables keyed by tenant were declared
    -- with ON DELETE CASCADE only, so they are recreated with ON UPDATE CASCADE
    -- as well and stay that way: it costs nothing at rest and makes a later
    -- rename a plain UPDATE.
    FOR fk IN
        SELECT conrelid::regclass AS tbl, conname, pg_get_constraintdef(oid) AS def
        FROM pg_constraint
        WHERE contype = 'f'
          AND confupdtype = 'a'
          AND connamespace = current_schema()::regnamespace
          AND (confrelid = 'tenants'::regclass
               OR confrelid IN (SELECT attrelid FROM pg_attribute
                                WHERE attname = 'tenant' AND NOT attisdropped))
    LOOP
        EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I, ADD CONSTRAINT %I %s ON UPDATE CASCADE',
                       fk.tbl, fk.conname, fk.conname, fk.def);
    END LOOP;

    UPDATE tenants
    SET id = trim(both '-' from regexp_replace(lower(id), '[^a-z0-9]+', '-', 'g'))
    WHERE id !~ pattern OR length(id) > 63;

    UPDATE projects
    SET id = trim(both '-' from regexp_replace(lower(id), '[^a-z0-9]+', '-', 'g'))
    WHERE id !~ pattern OR length(id) > 63;

    -- The Project resource itself, every version of it: its id column is
    -- generated from the JSON, which no key reaches.
    UPDATE resources
    SET resource = jsonb_set(resource, '{id}',
                             to_jsonb(trim(both '-' from regexp_replace(lower(id), '[^a-z0-9]+', '-', 'g'))))
    WHERE resource_type = 'Project' AND (id !~ pattern OR length(id) > 63);

    -- The one tenant column without a foreign key.
    UPDATE locks
    SET tenant = trim(both '-' from regexp_replace(lower(tenant), '[^a-z0-9]+', '-', 'g'))
    WHERE tenant !~ pattern OR length(tenant) > 63;

    -- Sessions name their tenant inside serialized data this block cannot
    -- read, so every session goes; the table exists once a server has run.
    IF to_regclass('tower_sessions.session') IS NOT NULL THEN
        DELETE FROM tower_sessions.session;
    END IF;
END $$;

ALTER TABLE tenants
    ADD CONSTRAINT tenants_id_hostname_check
    CHECK (id ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND length(id) <= 63);

ALTER TABLE projects
    ADD CONSTRAINT projects_id_hostname_check
    CHECK (id ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND length(id) <= 63);
