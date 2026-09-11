-- register_sequence_transaction: identical contract, no catalog scans.
create or replace function register_sequence_transaction(sequence_name text)
returns bigint as $$
declare
    seq_id   oid;
    next_val bigint;
begin
    -- regclass resolves through the relation syscache (in-memory hash),
    -- not a pg_class scan. Raises undefined_table itself if missing, so
    -- the explicit `if seq_id is null` check is no longer reachable.
    seq_id := sequence_name::regclass;

    -- Direct sequence-page read. Returns NULL if the sequence has never
    -- been called -- the old code passed that NULL straight into
    -- pg_advisory_xact_lock_shared, which takes no lock and returns NULL,
    -- leaving that transaction invisible to max_safe_seq. Narrow window
    -- (first write ever) but it's a real skipped-row bug; coalesce closes it.
    next_val := coalesce(pg_sequence_last_value(seq_id), 0);

    -- Acquire a lock on this sequence
    perform pg_try_advisory_xact_lock_shared(seq_id::int, 0);
     -- Acquire a lock with the last value used
    perform pg_advisory_xact_lock_shared(next_val);

    return next_val;
end;
$$ language plpgsql;


create or replace function max_safe_seq(sequence_name text)
returns bigint as $$
declare
    seq_id       oid;
    sequence_max bigint;
    max_seq      bigint;
begin
    seq_id := sequence_name::regclass;

    -- MUST be read BEFORE scanning pg_locks. A writer registers (reads
    -- last_value, takes its lock) and only then does nextval(). Scanning locks
    -- first and reading last_value after loses any writer that registers in
    -- the gap, and returns a watermark sitting above its uncommitted row
    -- the worker advances past it and that row is never indexed.
    sequence_max := coalesce(pg_sequence_last_value(seq_id), 0);

    select min((l1.classid::bigint << 32) | l1.objid::bigint)
      into max_seq
      from pg_locks l1
      join pg_locks l2 on l1.pid = l2.pid
     -- l2: the writer marker, two-int key (seq oid, 0) -> objsubid 2.
     -- Compared oid to oid; ::int is a binary coercion that silently wraps
     -- negative above 2^31 (it round-trips, but there's no reason to do it).
     where l2.locktype = 'advisory' and l2.objsubid = 2 and l2.classid = seq_id
     -- l1: the registered sequence value, single-bigint key -> objsubid 1.
     -- Reconstruct the full 64-bit key: pg_locks splits it as
     -- classid = key>>32, objid = key & 0xffffffff. The old `classid = 0`
     -- filter therefore matches nothing once the sequence passes 2^32, and
     -- max_safe_seq silently falls through to last_value.
       and l1.locktype = 'advisory' and l1.objsubid = 1
     -- mode is what separates us from the rate limiter, which takes an
     -- EXCLUSIVE bigint advisory lock (pg_advisory_xact_lock(hashtext(key)))
     -- in this same transaction, with an otherwise identical pg_locks shape.
     -- Registration is SHARED. Holds while nothing else takes a shared
     -- bigint advisory lock.
       and l1.mode = 'ShareLock';

    return coalesce(max_seq, sequence_max);
end;
$$ language plpgsql;