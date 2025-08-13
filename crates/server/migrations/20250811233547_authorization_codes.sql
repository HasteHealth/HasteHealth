CREATE TYPE code_kind AS ENUM (
    'password_reset',
    'oauth2_code_grant',
    'refresh_token'
);

CREATE TYPE pkce_method AS ENUM ('S256', 'plain');

CREATE TABLE
    authorization_code (
        tenant text NOT NULL,
        id uuid DEFAULT gen_random_uuid () NOT NULL PRIMARY KEY,
        client_id text,
        kind code_kind NOT NULL,
        code text NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        expires_in interval NOT NULL,
        user_id text NOT NULL,
        pkce_code_challenge text,
        pkce_code_challenge_method pkce_method,
        redirect_uri character varying(255),
        meta jsonb,
        CONSTRAINT authorization_code_code_key UNIQUE (code),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        CONSTRAINT fk_user FOREIGN KEY (tenant, user_id) REFERENCES users (tenant, fhir_user_id) ON DELETE CASCADE
    );

CREATE TABLE
    authorization_scopes (
        tenant text NOT NULL,
        client_id text NOT NULL,
        user_id text NOT NULL,
        scope text NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        CONSTRAINT authorization_scopes_pkey PRIMARY KEY (tenant, client_id, user_id),
        CONSTRAINT authorization_scopes_tenant_fkey FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        CONSTRAINT fk_user FOREIGN KEY (tenant, user_id) REFERENCES users (tenant, fhir_user_id) ON DELETE CASCADE
    );