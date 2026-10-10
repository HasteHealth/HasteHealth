-- SMART EHR launch codes: the opaque launch value an app is started with, and
-- the patient (and encounter) it stands for.
ALTER TYPE code_kind
ADD VALUE IF NOT EXISTS 'smart_launch';
