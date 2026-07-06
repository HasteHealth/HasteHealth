-- USER MFA CREDENTIAL TABLE
CREATE TABLE
    user_mfa_credential (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
        tenant TEXT NOT NULL,
        user_id TEXT NOT NULL,
        -- factor identity
        credential_type TEXT NOT NULL, -- 'totp' only fornow.
        -- Secret information
        secret_ciphertext BYTEA NOT NULL,
        secret_nonce BYTEA NOT NULL,
        key_id TEXT NOT NULL, -- which KEK/DEK version encrypted this
        -- TOTP parameters (not secret; don't hardcode in app)
        totp_algorithm TEXT NOT NULL DEFAULT 'SHA1',
        totp_digits SMALLINT NOT NULL DEFAULT 6,
        totp_period SMALLINT NOT NULL DEFAULT 30,
        -- how many time-steps to allow for clock skew (1 = 1 step before and after)
        totp_skew SMALLINT NOT NULL DEFAULT 1,
        -- lifecycle
        created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        is_active boolean NOT NULL DEFAULT false,
        activated_at TIMESTAMPTZ, -- NULL = enrolled but unverified
        FOREIGN KEY (tenant, user_id) REFERENCES users (tenant, id) ON DELETE CASCADE
    );

CREATE INDEX idx_user_mfa_lookup ON user_mfa_credential (tenant, user_id);