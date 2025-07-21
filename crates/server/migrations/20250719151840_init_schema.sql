--
-- Name: code_type; Type: TYPE; Schema: public; Owner: postgres
--
CREATE TYPE public.code_type AS ENUM (
    'password_reset',
    'oauth2_code_grant',
    'refresh_token'
);
ALTER TYPE public.code_type OWNER TO postgres;

--
-- Name: fhir_method; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.fhir_method AS ENUM (
    'update',
    'patch',
    'delete',
    'create'
);


ALTER TYPE public.fhir_method OWNER TO postgres;

--
-- Name: fhir_version; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.fhir_version AS ENUM (
    'r4',
    'r4b',
    'r5'
);

--
-- Name: proc_update_resource_meta(); Type: FUNCTION; Schema: public; Owner: postgres
--
CREATE FUNCTION public.proc_update_resource_meta() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
    BEGIN
        NEW.resource := jsonb_set(NEW.resource, '{meta,lastUpdated}', to_jsonb(generate_fhir_instant_string(NEW.created_at)));

        RETURN NEW;
    END;
$$;

ALTER FUNCTION public.proc_update_resource_meta() OWNER TO postgres;

CREATE TABLE public.resources (
    id text GENERATED ALWAYS AS ((resource ->> 'id'::text)) STORED NOT NULL,
    tenant text NOT NULL,
    project text NOT NULL,
    resource_type text GENERATED ALWAYS AS ((resource ->> 'resourceType'::text)) STORED NOT NULL,
    author_id text NOT NULL,
    resource jsonb NOT NULL,
    deleted boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    request_method character varying(7) DEFAULT 'PUT'::character varying,
    fhir_version public.fhir_version NOT NULL,
    author_type text NOT NULL,
    version_id text GENERATED ALWAYS AS (((resource -> 'meta'::text) ->> 'versionId'::text)) STORED NOT NULL,
    fhir_method public.fhir_method NOT NULL,
    sequence bigint NOT NULL
);

ALTER TABLE public.resources OWNER TO postgres;

CREATE SEQUENCE public.resources_sequence_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.resources_sequence_seq OWNER TO postgres;
ALTER SEQUENCE public.resources_sequence_seq OWNED BY public.resources.sequence;    
ALTER TABLE ONLY public.resources ALTER COLUMN sequence SET DEFAULT nextval('public.resources_sequence_seq'::regclass);
ALTER TABLE ONLY public.resources
    ADD CONSTRAINT resources_pkey PRIMARY KEY (version_id);
CREATE INDEX resources_id_idx ON public.resources USING btree (tenant, id);
CREATE INDEX resources_type_fitler ON public.resources USING btree (tenant, fhir_version, resource_type);
CREATE TRIGGER update_resource_meta_trigger BEFORE INSERT OR UPDATE ON public.resources FOR EACH ROW EXECUTE FUNCTION public.proc_update_resource_meta();

