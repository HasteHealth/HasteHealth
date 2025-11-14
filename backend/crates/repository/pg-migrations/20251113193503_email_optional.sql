ALTER TABLE users
ALTER COLUMN email
DROP NOT NULL,
ADD CONSTRAINT email_required_if_email_password CHECK (
    method != 'email-password'
    OR email IS NOT NULL
);