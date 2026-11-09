-- Tenant and project ids are hostname labels: lowercase letters and digits
-- joined by single hyphens, at most 63 characters (haste_jwt::HOSTNAME_ID_PATTERN).
-- TenantId and ProjectId enforce that rule when they are built, so a row that
-- holds anything else can no longer be read. Ids created before the rule are
-- rewritten to it here, wherever they appear in this database: lowercased,
-- every run of other characters replaced by one hyphen, hyphens at either end
-- dropped. The search index database rewrites its rows with the same
-- expression in its own migration. A document store on a filesystem or in a
-- bucket is not touched: it ships in the same release as this migration, so
-- it holds nothing under an old id yet.
--
-- Nothing happens in a database whose ids all conform. Otherwise the rewrite is
-- one transaction, and it fails, leaving everything as it was, when a rewritten
-- id collides with an existing one or comes out empty; the operator renames
-- that tenant or project by hand and migrates again. Users of a renamed tenant
-- sign in again: their sessions and tokens name the old id, and so does any
-- URL they had, since the id is part of the hostname and the path.
DO $$
DECLARE
    pattern CONSTANT text := '^[a-z0-9]+(-[a-z0-9]+)*$';
    restore text[];
    statement text;
    rename RECORD;
    tbl text;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM tenants WHERE id !~ pattern OR length(id) > 63)
       AND NOT EXISTS (SELECT 1 FROM projects WHERE id !~ pattern OR length(id) > 63) THEN
        RETURN;
    END IF;

    -- Every foreign key is checked at the end of this block instead of after
    -- each statement, so an id can change on both sides of a key (tenants and
    -- their referrers, projects, resources, document_storage and its chunks).
    -- A table with a tenant column that the loops below missed would fail
    -- that check. The ones that were not deferrable are put back afterwards.
    SELECT array_agg(format('ALTER TABLE %s ALTER CONSTRAINT %I NOT DEFERRABLE', conrelid::regclass, conname))
    INTO restore
    FROM pg_constraint
    WHERE contype = 'f' AND NOT condeferrable AND connamespace = current_schema()::regnamespace;
    FOREACH statement IN ARRAY coalesce(restore, '{}') LOOP
        EXECUTE replace(statement, ' NOT DEFERRABLE', ' DEFERRABLE INITIALLY DEFERRED');
    END LOOP;
    SET CONSTRAINTS ALL DEFERRED;

    FOR rename IN
        SELECT id AS old_id,
               trim(both '-' from regexp_replace(lower(id), '[^a-z0-9]+', '-', 'g')) AS new_id
        FROM tenants
        WHERE id !~ pattern OR length(id) > 63
    LOOP
        IF rename.new_id = '' OR length(rename.new_id) > 63 THEN
            RAISE EXCEPTION 'Tenant id % has no hostname form; rename it by hand before migrating.', rename.old_id;
        END IF;
        IF EXISTS (SELECT 1 FROM tenants WHERE id = rename.new_id) THEN
            RAISE EXCEPTION 'Tenant id % would become %, which is taken; rename it by hand before migrating.', rename.old_id, rename.new_id;
        END IF;
        FOR tbl IN
            SELECT c.relname
            FROM pg_attribute a
            JOIN pg_class c ON c.oid = a.attrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = current_schema() AND c.relkind IN ('r', 'p')
              AND a.attname = 'tenant' AND NOT a.attisdropped
        LOOP
            EXECUTE format('UPDATE %I SET tenant = $1 WHERE tenant = $2', tbl)
                USING rename.new_id, rename.old_id;
        END LOOP;
        UPDATE tenants SET id = rename.new_id WHERE id = rename.old_id;
        RAISE NOTICE 'Renamed tenant % to %.', rename.old_id, rename.new_id;
    END LOOP;

    -- After the tenants, so a project's tenant column already holds the new id.
    FOR rename IN
        SELECT tenant, id AS old_id,
               trim(both '-' from regexp_replace(lower(id), '[^a-z0-9]+', '-', 'g')) AS new_id
        FROM projects
        WHERE id !~ pattern OR length(id) > 63
    LOOP
        IF rename.new_id = '' OR length(rename.new_id) > 63 THEN
            RAISE EXCEPTION 'Project id % in tenant % has no hostname form; rename it by hand before migrating.', rename.old_id, rename.tenant;
        END IF;
        IF EXISTS (SELECT 1 FROM projects WHERE tenant = rename.tenant AND id = rename.new_id) THEN
            RAISE EXCEPTION 'Project id % in tenant % would become %, which is taken; rename it by hand before migrating.', rename.old_id, rename.tenant, rename.new_id;
        END IF;
        FOR tbl IN
            SELECT c.relname
            FROM pg_attribute a
            JOIN pg_class c ON c.oid = a.attrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = current_schema() AND c.relkind IN ('r', 'p')
              AND a.attname = 'project' AND NOT a.attisdropped
              AND EXISTS (SELECT 1 FROM pg_attribute t
                          WHERE t.attrelid = c.oid AND t.attname = 'tenant' AND NOT t.attisdropped)
        LOOP
            EXECUTE format('UPDATE %I SET project = $1 WHERE tenant = $2 AND project = $3', tbl)
                USING rename.new_id, rename.tenant, rename.old_id;
        END LOOP;
        -- The Project resource itself, every version of it: its id column is
        -- generated from the JSON.
        UPDATE resources
        SET resource = jsonb_set(resource, '{id}', to_jsonb(rename.new_id))
        WHERE tenant = rename.tenant AND resource_type = 'Project' AND id = rename.old_id;
        UPDATE projects SET id = rename.new_id WHERE tenant = rename.tenant AND id = rename.old_id;
        RAISE NOTICE 'Renamed project % in tenant % to %.', rename.old_id, rename.tenant, rename.new_id;
    END LOOP;

    -- Sessions name their tenant inside serialized data this block cannot
    -- read, so every session goes; the table exists once a server has run.
    IF to_regclass('tower_sessions.session') IS NOT NULL THEN
        DELETE FROM tower_sessions.session;
    END IF;

    -- Runs the deferred checks now, then puts the constraints back as they were.
    SET CONSTRAINTS ALL IMMEDIATE;
    FOREACH statement IN ARRAY coalesce(restore, '{}') LOOP
        EXECUTE statement;
    END LOOP;
END $$;
