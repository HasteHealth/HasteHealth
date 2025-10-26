ALTER TABLE resources
DROP CONSTRAINT resources_pkey,
ADD CONSTRAINT resources_pkey PRIMARY KEY (tenant, project, version_id);

ALTER TABLE MEMBERSHIPS
DROP CONSTRAINT membership_pkey,
ADD CONSTRAINT membership_pkey PRIMARY KEY (tenant, project, user_id);