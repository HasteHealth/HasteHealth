-- Replay protection for private_key_jwt client assertions (RFC 7523 section 3,
-- item 7): each (tenant, project, client, jti) is accepted once and kept until
-- the assertion that carried it has expired, after which the row is purged.
CREATE TABLE
    client_assertion_jti (
        tenant text NOT NULL,
        project text NOT NULL,
        client_id text NOT NULL,
        jti text NOT NULL,
        expires_at TIMESTAMPTZ NOT NULL,
        CONSTRAINT client_assertion_jti_pkey PRIMARY KEY (tenant, project, client_id, jti),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        CONSTRAINT fk_project FOREIGN KEY (tenant, project) REFERENCES projects (tenant, id) ON DELETE CASCADE
    );

CREATE INDEX client_assertion_jti_expires_at_idx ON client_assertion_jti (expires_at);
