CREATE TYPE user_role AS ENUM ('owner', 'admin', 'member');

CREATE TYPE auth_method as ENUM ('email-password', 'oidc-provider');

CREATE TABLE
    users (
        tenant text NOT NULL,
        fhir_user_id text NOT NULL,
        fhir_provider_id text,
        email text NOT NULL,
        password text,
        role user_role NOT NULL,
        method auth_method DEFAULT 'email-password' NOT NULL,
        email_verified boolean DEFAULT false,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        CONSTRAINT users_pkey PRIMARY KEY (tenant, fhir_user_id),
        CONSTRAINT unique_email UNIQUE NULLS NOT DISTINCT (tenant, email, method, fhir_provider_id),
        CONSTRAINT unique_fhir_user UNIQUE (tenant, fhir_user_id),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE
    );

CREATE UNIQUE INDEX owner_unique_idx ON users USING btree (email)
WHERE
    role = 'owner'
    AND email_verified = true;