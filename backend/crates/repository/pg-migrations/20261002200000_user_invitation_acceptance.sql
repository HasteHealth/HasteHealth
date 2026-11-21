-- `email_verified` now records that a user accepted their account, and
-- email-password login requires it. Existing users who could log in (a
-- password is set, or they own the tenant) are marked accepted. The rest had
-- no password and will accept on their next login.
UPDATE users
SET
    email_verified = true
WHERE
    method = 'email-password'
    AND (
        password IS NOT NULL
        OR role = 'owner'
    );

UPDATE users
SET
    email_verified = false
WHERE
    email_verified IS NULL;

ALTER TABLE users
ALTER COLUMN email_verified
SET DEFAULT false,
ALTER COLUMN email_verified
SET NOT NULL;

-- Invitation link codes.
ALTER TYPE code_kind
ADD VALUE IF NOT EXISTS 'invitation';
