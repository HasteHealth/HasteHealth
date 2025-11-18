CREATE OR REPLACE FUNCTION filter_fhir_extensions (url TEXT, value JSONB) RETURNS JSONB AS $$
     declare variables JSONB;
     declare res JSONB;
     BEGIN
          variables := jsonb_set('{}'::jsonb, '{url}', to_jsonb(url::text));
      -- Filters all extensions that don't have url or equal to the url passed in.
	    res := (SELECT jsonb_agg(jsonb_path_query) 
            FROM jsonb_path_query(value, '$ ? (@.url != $url)', variables));

        IF res is NULL THEN
            res:= '[]'::jsonb;
        END IF;

        RETURN res;
	  
     END;
$$  LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION haste_fhir_extensions (version_id TEXT, author_type Text, author_id Text, extensions JSONB) RETURNS JSONB AS $$
    declare ext_author_url TEXT;
    declare new_ext_author JSONB;
    declare author_reference JSONB;

     BEGIN
        ext_author_url := 'https://haste.health/author';

        author_reference := jsonb_set('{}'::jsonb, '{reference}', to_jsonb(concat(author_type, '/', author_id)::text));
        new_ext_author := jsonb_set('{}'::jsonb, '{url}', to_jsonb(ext_author_url::text));
        new_ext_author := jsonb_set(new_ext_author, '{valueReference}', author_reference);
        
        extensions := 
          filter_fhir_extensions(ext_author_url, extensions)  ||
          new_ext_author;

        return extensions;
     END;
$$  LANGUAGE plpgsql;


--
-- Name: proc_update_resource_meta(); Auto updating with extension and meta last updated.
--
CREATE OR REPLACE FUNCTION proc_update_resource_meta() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
    BEGIN
        IF (NEW.resource -> 'meta') IS NULL THEN
    	   NEW.resource := jsonb_set(NEW.resource, '{meta}', '{}');
        END IF;

        IF (NEW.resource -> 'meta' -> 'extension') IS NULL THEN
           NEW.resource := jsonb_set(NEW.resource, '{meta, extension}', '[]');
        END IF;

        NEW.resource := jsonb_set(NEW.resource, '{meta,extension}',   haste_fhir_extensions(NEW.version_id, NEW.author_type, NEW.author_id, NEW.resource -> 'meta' -> 'extension'));
        NEW.resource := jsonb_set(NEW.resource, '{meta,lastUpdated}', to_jsonb(generate_fhir_instant_string(NEW.created_at)));

        RETURN NEW;
    END;
$$;