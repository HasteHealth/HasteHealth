CREATE TYPE membership_role AS ENUM ('admin', 'member');

CREATE TABLE
    memberships (
        tenant text NOT NULL,
        user text NOT NULL,
        role membership_role NOT NULL,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        CONSTRAINT membership_pkey PRIMARY KEY (tenant, user),
        CONSTRAINT fk_user FOREIGN KEY (user) REFERENCES users (fhir_user_id) ON DELETE CASCADE,
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE
    );