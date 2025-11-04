ALTER TABLE memberships
DROP COLUMN IF EXISTS resource_id,
ADD COLUMN resource_id TEXT NOT NULL;