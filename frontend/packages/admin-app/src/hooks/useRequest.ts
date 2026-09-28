/**
 * Running one FHIR request and tracking what happened to it.
 *
 * Every panel in the console does the same four things around a request: show
 * a spinner, time it, keep the failure as an outcome it can render, and
 * ignore a response that arrived after the user moved on. Writing that by
 * hand in each panel is where the bugs live, so it is written once here.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import {
  Outcome,
  toOutcome,
} from "@haste-health/components";


export interface RequestState<T> {
  data?: T;
  loading: boolean;
  /** The failure, shaped for `OutcomePanel`. */
  outcome?: Outcome;
  /** Milliseconds the round trip took, including a failed one. */
  elapsedMs?: number;
  /** Runs the request again. */
  refresh: () => void;
}

/**
 * Runs `send` whenever `deps` change, and on `refresh()`.
 *
 * A response that arrives after the deps changed is discarded, so a slow
 * request can never overwrite the results of the one that replaced it.
 */
export function useRequest<T>(
  send: () => Promise<T>,
  deps: React.DependencyList,
): RequestState<T> {
  const [data, setData] = useState<T>();
  const [loading, setLoading] = useState(true);
  const [outcome, setOutcome] = useState<Outcome>();
  const [elapsedMs, setElapsedMs] = useState<number>();
  const [nonce, setNonce] = useState(0);

  // `send` is read through a ref so that an inline closure - which every
  // caller passes - does not re-run the request on every render.
  const latest = useRef(send);
  latest.current = send;

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setOutcome(undefined);
    const startedAt = performance.now();

    latest
      .current()
      .then((result) => {
        if (cancelled) return;
        setData(result);
      })
      .catch((error: unknown) => {
        if (cancelled) return;
        setOutcome(toOutcome(error));
        setData(undefined);
      })
      .finally(() => {
        if (cancelled) return;
        setElapsedMs(Math.round(performance.now() - startedAt));
        setLoading(false);
      });

    return () => {
      cancelled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, nonce]);

  const refresh = useCallback(() => setNonce((n) => n + 1), []);

  return { data, loading, outcome, elapsedMs, refresh };
}
