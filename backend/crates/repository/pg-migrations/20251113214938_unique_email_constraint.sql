DROP CONSTRAINT IF EXISTS unique_email;

CREATE UNIQUE INDEX unique_email_idx ON users USING btree (tenant, email)
WHERE
    method = 'email-password';