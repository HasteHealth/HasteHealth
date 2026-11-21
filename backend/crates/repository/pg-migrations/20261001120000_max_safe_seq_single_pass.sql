-- Same contract. The marker lock now blocks instead of using the try_ form,
-- which on failure silently dropped the writer out of max_safe_seq.
create or replace function register_sequence_transaction(sequence_name text)
returns bigint as $$
declare
    seq_id   oid;
    next_val bigint;
begin
    -- Raises undefined_table if the sequence is missing.
    seq_id := sequence_name::regclass;

    -- NULL before the first nextval().
    next_val := coalesce(pg_sequence_last_value(seq_id), 0);

    -- Marker: this backend is a registrant of this sequence.
    perform pg_advisory_xact_lock_shared(seq_id::int, 0);
    -- Registered value: every row this transaction writes sits above it.
    perform pg_advisory_xact_lock_shared(next_val);

    return next_val;
end;
$$ language plpgsql;


-- Same contract, one pass over pg_locks.
--
-- The previous self-join on pid was planned as a nested loop (pg_locks has no
-- statistics) that rescanned the lock table once per registrant: 20-170 ms
-- per call at ~2000 lock entries. Grouping by pid gives the same answer in
-- 0.8 ms under the same load.
create or replace function max_safe_seq(sequence_name text)
returns bigint as $$
declare
    seq_id       oid;
    sequence_max bigint;
    max_seq      bigint;
begin
    seq_id := sequence_name::regclass;

    -- MUST be read BEFORE scanning pg_locks. Otherwise a writer registering
    -- in between is missed and the watermark passes its uncommitted row.
    sequence_max := coalesce(pg_sequence_last_value(seq_id), 0);

    select min(s.value)
      into max_seq
      from (
          select l.pid,
                 -- Marker: two-int key (seq oid, 0), objsubid 2.
                 bool_or(l.objsubid = 2 and l.classid = seq_id) as is_registrant,
                 -- Registered value: bigint key, objsubid 1, split across
                 -- classid (high 32 bits) and objid (low). ShareLock excludes
                 -- the rate limiter's exclusive bigint lock.
                 min(case when l.objsubid = 1 and l.mode = 'ShareLock'
                          then (l.classid::bigint << 32) | l.objid::bigint
                     end) as value
            from pg_locks l
           where l.locktype = 'advisory'
           group by l.pid
      ) s
     where s.is_registrant;

    return coalesce(max_seq, sequence_max);
end;
$$ language plpgsql;


-- CACHE > 1 would let a session hand out a preallocated value below a later
-- registration, which max_safe_seq would report as safe before it commits.
comment on sequence resources_sequence_seq is
    'Write order of resources. CACHE 1 is required: register_sequence_transaction and max_safe_seq assume every nextval() after a registration returns a value above the registered last_value.';
