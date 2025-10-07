CREATE TYPE membership_role AS ENUM ('admin', 'member');

CREATE TABLE
    memberships (
        tenant TEXT NOT NULL,
        project TEXT NOT NULL,
        user_id TEXT NOT NULL,
        role membership_role NOT NULL,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        CONSTRAINT membership_pkey PRIMARY KEY (tenant, user_id),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        CONSTRAINT fk_project FOREIGN KEY (tenant, project) REFERENCES project (tenant, id) ON DELETE CASCADE,
        CONSTRAINT fk_user FOREIGN KEY (tenant, user_id) REFERENCES users (tenant, id) ON DELETE CASCADE
    );