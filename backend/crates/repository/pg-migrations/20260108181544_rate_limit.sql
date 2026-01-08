-- https://neon.com/guides/rate-limiting
-- Rate Limit setup on postgres.
CREATE TABLE
    rate_limits (
        key TEXT PRIMARY KEY,
        points INTEGER NOT NULL DEFAULT 0,
        window_start TIMESTAMPTZ NOT NULL
    );


CREATE OR REPLACE FUNCTION check_rate_limit(rate_key TEXT, max_points INTEGER, next_points INTEGER, window_seconds INTEGER)
RETURNS INTEGER AS $$
DECLARE
  now TIMESTAMPTZ := clock_timestamp();
  window_length INTERVAL := make_interval(secs => window_seconds);
  current_points INTEGER;
BEGIN
  PERFORM pg_advisory_xact_lock(hashtext(rate_key));

  INSERT INTO rate_limits (key, points, window_start)
  VALUES (rate_key, next_points, now)
  ON CONFLICT (key) DO UPDATE
  SET points = CASE
                WHEN rate_limits.window_start + window_length <= now
                  THEN next_points
                  ELSE rate_limits.points + next_points
              END,
      window_start = CASE
                       WHEN rate_limits.window_start + window_length <= now
                         THEN now
                         ELSE rate_limits.window_start
                     END;
  -- Note if it fails than the transaction will be aborted so should not have side effects.
  SELECT points INTO current_points FROM rate_limits WHERE key = rate_key;
  IF current_points > max_points THEN
    RAISE EXCEPTION 'Rate limit exceeded for %', rate_key;
  END IF;

  RETURN current_points;
END;
$$ LANGUAGE plpgsql;