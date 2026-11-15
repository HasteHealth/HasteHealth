-- Like check_rate_limit, but reports when the window resets so a refused
-- caller can be told when to retry, and refuses by returning exceeded = true
-- rather than raising, so a refusal spends nothing and leaves the caller's
-- transaction usable.
--
-- check_rate_limit stays for servers still running the previous release
-- during a rolling deploy; nothing in this release calls it.
CREATE OR REPLACE FUNCTION check_rate_limit_window(rate_key TEXT, max_points INTEGER, next_points INTEGER, window_seconds INTEGER)
RETURNS TABLE (spent INTEGER, resets_in_ms BIGINT, exceeded BOOLEAN) AS $$
DECLARE
  now TIMESTAMPTZ := clock_timestamp();
  window_length INTERVAL := make_interval(secs => window_seconds);
  current_points INTEGER;
  current_start TIMESTAMPTZ;
BEGIN
  PERFORM pg_advisory_xact_lock(hashtext(rate_key));

  SELECT rate_limits.points, rate_limits.window_start
    INTO current_points, current_start
    FROM rate_limits
   WHERE rate_limits.key = rate_key;

  -- No row yet, or its window has passed: a new window starts now.
  IF current_start IS NULL OR current_start + window_length <= now THEN
    current_points := 0;
    current_start := now;
  END IF;

  exceeded := current_points + next_points > max_points;

  IF NOT exceeded THEN
    current_points := current_points + next_points;

    INSERT INTO rate_limits (key, points, window_start)
    VALUES (rate_key, current_points, current_start)
    ON CONFLICT (key) DO UPDATE
    SET points = EXCLUDED.points,
        window_start = EXCLUDED.window_start;
  END IF;

  spent := current_points;
  resets_in_ms := ceil(extract(epoch FROM (current_start + window_length - now)) * 1000)::BIGINT;
  RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

-- Tenant request budgets were counted against the old one-day window. Clear
-- them so no tenant stays refused under a window it no longer has. They are
-- keyed by the bare tenant id, which is a hostname label and never contains
-- ':'; every other limit (email codes, package installs) is keyed
-- '<purpose>:...' and keeps its count.
DELETE FROM rate_limits WHERE strpos(key, ':') = 0;
