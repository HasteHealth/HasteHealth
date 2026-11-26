-- The secrets of a tenant that the server has to read back, such as the
-- shared secret of a TOTP credential. A row holds one secret, encrypted under
-- one version of the tenant's key (see tenant_key). The encryption binds the
-- secret to its row: it decrypts only for the tenant, kind and owner it was
-- stored for.
CREATE TABLE
    tenant_secret (
        tenant text NOT NULL,
        -- What the secret is.
        kind text NOT NULL,
        -- What the secret belongs to, such as the id of the TOTP credential.
        owner text NOT NULL,
        ciphertext bytea NOT NULL,
        nonce bytea NOT NULL,
        -- The version of the tenant's key that decrypts the secret.
        key_version integer NOT NULL,
        -- When the secret was last stored. Re-encrypting it under another key
        -- version does not change this.
        set_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        CONSTRAINT tenant_secret_pkey PRIMARY KEY (tenant, kind, owner),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        -- A key version cannot be removed while a secret is encrypted under
        -- it. Deleting the tenant removes its secrets and its keys together.
        CONSTRAINT fk_tenant_key FOREIGN KEY (tenant, key_version) REFERENCES tenant_key (tenant, version),
        CONSTRAINT tenant_secret_kind CHECK (kind IN ('mfa_totp', 'idp_client_secret'))
    );

-- The secrets under one key version: what a key rotation has left to
-- re-encrypt, and what the foreign key looks up when a key is removed.
CREATE INDEX tenant_secret_key_version_idx ON tenant_secret (tenant, key_version);
