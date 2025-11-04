ALTER TABLE users
RENAME COLUMN fhir_user_id TO id;

ALTER TABLE users
RENAME COLUMN fhir_provider_id TO provider_id;