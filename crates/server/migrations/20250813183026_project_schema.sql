CREATE TABLE
    project (
        tenant text NOT NULL,
        id uuid DEFAULT gen_random_uuid () NOT NULL PRIMARY KEY,
        fhir_version fhir_version NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
        CONSTRAINT fk_tenant FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE
    );