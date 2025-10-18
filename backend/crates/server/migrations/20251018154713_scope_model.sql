DROP TABLE authorization_scopes;

CREATE TABLE
    authorization_scopes (
        client TEXT NOT NULL,
        user_ TEXT NOT NULL,
        scope TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        tenant TEXT NOT NULL,
        project TEXT NOT NULL,
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        CONSTRAINT fk_project FOREIGN KEY (tenant, project) REFERENCES projects (tenant, id) ON DELETE CASCADE,
        CONSTRAINT fk_user FOREIGN KEY (tenant, user_) REFERENCES users (tenant, id) ON DELETE CASCADE
    )