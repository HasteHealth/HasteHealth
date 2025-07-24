--
-- Name: code_type; Type: TYPE; Schema: public; Owner: postgres
--
CREATE TYPE code_type AS ENUM (
    'password_reset',
    'oauth2_code_grant',
    'refresh_token'
);
ALTER TYPE code_type OWNER TO postgres;

--
-- Name: fhir_method; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE fhir_method AS ENUM (
    'update',
    'patch',
    'delete',
    'create'
);


ALTER TYPE fhir_method OWNER TO postgres;

--
-- Name: fhir_version; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE fhir_version AS ENUM (
    'r4',
    'r4b',
    'r5'
);

--
-- Name: proc_update_resource_meta(); Type: FUNCTION; Schema: public; Owner: postgres
--
CREATE FUNCTION proc_update_resource_meta() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
    BEGIN
        NEW.resource := jsonb_set(NEW.resource, '{meta,lastUpdated}', to_jsonb(generate_fhir_instant_string(NEW.created_at)));

        RETURN NEW;
    END;
$$;

--
-- Name: generate_fhir_instant_string(timestamp with time zone); Type: FUNCTION; Schema: public; Owner: postgres
--
CREATE FUNCTION public.generate_fhir_instant_string(tstamp timestamp with time zone) RETURNS text
    LANGUAGE plpgsql
    AS $$
     declare utc_time TIMESTAMPTZ;
     BEGIN
          utc_time := tstamp AT TIME ZONE 'UTC';
	  RETURN to_char(utc_time, 'YYYY-MM-DD') ||
	         'T' ||
         	 to_char(utc_time, 'HH24:MI:SS.MS+00:00');
	  
     END;
$$;

ALTER FUNCTION proc_update_resource_meta() OWNER TO postgres;

CREATE TABLE resources (
    id text GENERATED ALWAYS AS ((resource ->> 'id'::text)) STORED NOT NULL,
    tenant text NOT NULL,
    project text NOT NULL,
    resource_type text GENERATED ALWAYS AS ((resource ->> 'resourceType'::text)) STORED NOT NULL,
    author_id text NOT NULL,
    resource jsonb NOT NULL,
    deleted boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    request_method character varying(7) DEFAULT 'PUT'::character varying,
    fhir_version fhir_version NOT NULL,
    author_type text NOT NULL,
    version_id text GENERATED ALWAYS AS (((resource -> 'meta'::text) ->> 'versionId'::text)) STORED NOT NULL,
    fhir_method fhir_method NOT NULL,
    sequence bigint NOT NULL
);

ALTER TABLE resources OWNER TO postgres;

CREATE SEQUENCE resources_sequence_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE resources_sequence_seq OWNER TO postgres;
ALTER SEQUENCE resources_sequence_seq OWNED BY resources.sequence;    
ALTER TABLE ONLY resources ALTER COLUMN sequence SET DEFAULT nextval('resources_sequence_seq'::regclass);
ALTER TABLE ONLY resources
    ADD CONSTRAINT resources_pkey PRIMARY KEY (version_id);
CREATE INDEX resources_id_idx ON resources USING btree (tenant, id);
CREATE INDEX resources_type_fitler ON resources USING btree (tenant, fhir_version, resource_type);
CREATE TRIGGER update_resource_meta_trigger BEFORE INSERT OR UPDATE ON resources FOR EACH ROW EXECUTE FUNCTION proc_update_resource_meta();


