create or replace function register_sequence_transaction(sequence_name text) 
returns bigint as $$
declare
    seq_id oid;
    next_val bigint;
begin
    -- Get the OID for the given sequence
    select oid into seq_id
    from pg_class
    where relname = sequence_name and relkind = 'S';

    if seq_id is null then
        raise exception 'Sequence % does not exist', sequence_name;
    end if;


    perform pg_advisory_lock(seq_id::int);
    -- Get the last value of the sequence
    select last_value into next_val
    from pg_sequences
    where sequencename = sequence_name;

	-- Acquire a lock on this sequence
    perform pg_try_advisory_xact_lock_shared(seq_id::int, 0);
    -- Acquire a lock with the last value used
    perform pg_advisory_xact_lock_shared(next_val);

    perform pg_advisory_unlock(seq_id::int);

    return next_val;
end;
$$ language plpgsql;

create or replace function max_safe_seq(sequence_name text) 
returns bigint as $$
declare
    seq_id oid;
    max_seq bigint;
begin
    -- Get the OID for the given sequence
    select oid into seq_id
    from pg_class
    where relname = sequence_name and relkind = 'S';
    
    if seq_id is null then
        raise exception 'Sequence % does not exist', sequence_name;
    end if;

    perform pg_advisory_lock(seq_id::int);
    -- Find the minimum seq across all running transactions
    select min(l1.objid) into max_seq
    from pg_locks l1
    inner join pg_locks l2 on l1.pid = l2.pid
    where l2.classid = seq_id::int
      and l1.classid = 0
      and l1.locktype = 'advisory';

    -- If no locks are found, return the maximum possible bigint value
    if max_seq is null then
        -- return 9223372036854775807;
        select last_value into max_seq
        from pg_sequences
        where sequencename = sequence_name;

        -- Negated so I could view in health checks.
        -- select max_seq * -1 into max_seq;
        -- return -1;
    end if;

    perform pg_advisory_unlock(seq_id::int);

    return max_seq;
end;
$$ language plpgsql;
