-- Document storage keeps a large Binary version's content out of its row and
-- leaves an extension on `_data` naming the stored document. The orphan sweep
-- asks, per stored document, whether any version still carries that name; this
-- index answers it with one containment probe instead of a scan of the
-- project's Binary rows (1000 keys against 200k Binary rows: 18 ms versus
-- 690 ms, and flat as rows grow). It covers Binary rows only, so other writes
-- do not maintain it.
--
-- fastupdate is off: with it on, entries wait in a pending list until vacuum
-- merges them, and every probe scans that whole list (up to 4 MB), which made
-- a sweep right after a burst of Binary writes slower than the scan it
-- replaces (4 s for the same 1000 keys). A direct insert into the tree costs a
-- Binary write a few extra pages, next to the document upload it already does.
CREATE INDEX resources_binary_data_idx
    ON resources USING GIN ((resource -> '_data') jsonb_path_ops)
    WITH (fastupdate = off)
    WHERE resource_type = 'Binary';
