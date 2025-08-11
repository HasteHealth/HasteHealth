CREATE TYPE user_role AS ENUM ('owner', 'admin', 'member');

CREATE TABLE
    users (
        email text NOT NULL,
        password text,
        email_verified boolean DEFAULT false,
        created_at timestamp time zone DEFAULT now () NOT NULL,
        updated_at timestamp time zone DEFAULT now () NOT NULL,
        tenant text NOT NULL,
        role user_role NOT NULL,
        method text DEFAULT 'email-password' NOT NULL,
        fhir_provider_id text,
        fhir_user_id text NOT NULL,
        CONSTRAINT unique_email UNIQUE NULLS NOT DISTINCT (tenant, email, method, fhir_provider_id),
        CONSTRAINT unique_fhir_user UNIQUE (tenant, fhir_user_id)
    );