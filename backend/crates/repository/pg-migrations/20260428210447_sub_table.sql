CREATE TABLE subscriptions (
    id TEXT,
    version_id TEXT NOT NULL,
    tenant TEXT NOT NULL,
    project TEXT NOT NULL,
    status TEXT NOT NULL, -- 'requested' | 'active' | 'error' | 'off'
    reason TEXT NOT NULL,
    criteria TEXT NOT NULL,
    
    -- Where to send the notifications
    channel_type TEXT NOT NULL, -- 'rest-hook' | 'websocket' | 'email' | 'sms'
    channel_endpoint TEXT,
    channel_payload TEXT, -- MIME type
    channel_headers JSONB, -- Array of headers FHIRString of headers.

    -- Tracking fields
    last_event_sequence BIGINT NOT NULL DEFAULT 0,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT subscription_pkey PRIMARY KEY (tenant, project, id),
    CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
    CONSTRAINT fk_project FOREIGN KEY (tenant, project) REFERENCES projects (tenant, id) ON DELETE CASCADE
);


CREATE INDEX subscriptions_tenant_project_idx ON subscriptions(tenant, project);
CREATE INDEX subscriptions_status_idx ON subscriptions(tenant, project, status);