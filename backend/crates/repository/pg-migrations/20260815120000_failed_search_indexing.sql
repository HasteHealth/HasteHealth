-- Reference-only record of resources that failed search indexing. Stores just
-- enough identity to look the resource back up and retry/inspect it later -
-- never the resource body itself.
CREATE TABLE
    failed_search_indexing (
        tenant TEXT NOT NULL,
        project TEXT NOT NULL,
        version_id TEXT NOT NULL,
        resource_type TEXT NOT NULL,
        fhir_method fhir_method NOT NULL,
        error_message TEXT NOT NULL,
        attempt_count INT NOT NULL DEFAULT 1,
        first_failed_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        last_failed_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        resolved_at TIMESTAMPTZ,
        PRIMARY KEY (tenant, project, version_id),
        FOREIGN KEY (tenant) REFERENCES tenants (id) ON DELETE CASCADE,
        FOREIGN KEY (tenant, project) REFERENCES projects (tenant, id) ON DELETE CASCADE,
        FOREIGN KEY (tenant, project, version_id) REFERENCES resources (tenant, project, version_id) ON DELETE CASCADE
    );

CREATE INDEX failed_search_indexing_unresolved_idx ON failed_search_indexing (tenant, project)
WHERE
    resolved_at IS NULL;