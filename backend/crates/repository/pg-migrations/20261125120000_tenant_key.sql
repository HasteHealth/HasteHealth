-- The keys that encrypt a tenant's secrets. A tenant has its own key, in
-- versions: the active one encrypts, and a retired one still decrypts what it
-- encrypted. A key is stored wrapped, that is encrypted by a platform key
-- this database never holds; wrapped_by names that platform key.
CREATE TABLE
    tenant_key (
        tenant text NOT NULL,
        version integer NOT NULL,
        wrapped_key bytea NOT NULL,
        wrapped_by text NOT NULL,
        state text NOT NULL DEFAULT 'active',
        created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        retired_at TIMESTAMPTZ,
        CONSTRAINT tenant_key_pkey PRIMARY KEY (tenant, version),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        CONSTRAINT tenant_key_state CHECK (state IN ('active', 'retired')),
        CONSTRAINT tenant_key_retired_at CHECK ((state = 'retired') = (retired_at IS NOT NULL))
    );

-- One key encrypts for a tenant at a time.
CREATE UNIQUE INDEX tenant_key_active_idx ON tenant_key (tenant)
WHERE
    state = 'active';
