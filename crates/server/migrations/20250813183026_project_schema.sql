CREATE TABLE
    project (
        tenant text NOT NULL,
        id text NOT NULL,
        fhir_version fhir_version NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        CONSTRAINT project_pkey PRIMARY KEY (tenant, id)
    );

ALTER TABLE resources ADD CONSTRAINT fk_project FOREIGN KEY (tenant, project) REFERENCES project (tenant, id) ON DELETE CASCADE;

ALTER TABLE authorization_code ADD CONSTRAINT fk_project FOREIGN KEY (tenant, project) REFERENCES project (tenant, id) ON DELETE CASCADE;