/** Runs the query typed into {@link FHIRSearchInput} against a server. */
import createHTTPClient from "@haste-health/client/http";
import {
  AllResourceTypes,
  FHIR_VERSION,
  Resource,
  ResourceType,
} from "@haste-health/fhir-types/versions";
import { useEffect, useMemo, useState } from "react";

import { SearchMetadata, SearchParameterResource } from "./metadata";

type HTTPClient = ReturnType<typeof createHTTPClient>;

export interface UseSearchOptions {
  client: HTTPClient;
  fhirVersion: FHIR_VERSION;
  /** Type to search; the query holds only parameters. */
  resourceType?: string;
  /** Query after the `?`, e.g. `name=Smith&_count=20`. */
  query: string;
  /** Rows per page, appended as `_count`. */
  pageSize: number;
  offset: number;
}

export interface UseSearchResult {
  resources: Resource<FHIR_VERSION, AllResourceTypes>[];
  total?: number;
  loading: boolean;
  /** Round trip time in ms as the browser saw it, not server processing. */
  elapsedMs?: number;
  /** Message from a rejected search. */
  error?: string;
  /**
   * Whatever the client threw, for a caller that renders an OperationOutcome
   * rather than the flattened message.
   */
  errorCause?: unknown;
  /** Search parameters for the searched type, for table columns. */
  searchParameters: SearchParameterResource[];
  /** Re-runs the current query. */
  refresh: () => void;
}

/** Reads a `diagnostics` message out of what the client threw. */
function errorMessage(error: unknown): string {
  const response = (error as { response?: { body?: unknown } })?.response;
  const body = response?.body as
    | { issue?: { diagnostics?: string }[] }
    | undefined;
  return (
    body?.issue?.[0]?.diagnostics ??
    (error instanceof Error ? error.message : "Search failed.")
  );
}

/**
 * Runs `query` and returns that page of results, with the searched type's
 * parameters so a table can build its columns.
 */
export function useSearch({
  client,
  fhirVersion,
  resourceType,
  query,
  pageSize,
  offset,
}: UseSearchOptions): UseSearchResult {
  const [resources, setResources] = useState<
    Resource<FHIR_VERSION, AllResourceTypes>[]
  >([]);
  const [total, setTotal] = useState<number>();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();
  const [errorCause, setErrorCause] = useState<unknown>();
  const [elapsedMs, setElapsedMs] = useState<number>();
  const [searchParameters, setSearchParameters] = useState<
    SearchParameterResource[]
  >([]);
  const [nonce, setNonce] = useState(0);

  const metadata = useMemo(
    () => new SearchMetadata(client, fhirVersion),
    [client, fhirVersion],
  );

  useEffect(() => {
    let cancelled = false;
    metadata.getParameters(resourceType).then((parameters) => {
      if (!cancelled) setSearchParameters(parameters);
    });
    return () => {
      cancelled = true;
    };
  }, [metadata, resourceType]);

  useEffect(() => {
    if (!resourceType) {
      setResources([]);
      setTotal(undefined);
      return undefined;
    }

    // A later query wins even if an earlier one resolves after it.
    let cancelled = false;
    setLoading(true);
    setError(undefined);
    setErrorCause(undefined);
    setElapsedMs(undefined);
    const startedAt = performance.now();

    // Ours go first so a typed `_count` overrides them.
    const parameters = [
      `_count=${pageSize}`,
      `_offset=${offset}`,
      "_total=estimate",
      query,
    ]
      .filter((part) => part.length > 0)
      .join("&");

    client
      .search_type(
        {},
        fhirVersion,
        resourceType as ResourceType<FHIR_VERSION>,
        parameters,
      )
      .then((bundle) => {
        if (cancelled) return;
        setResources(
          bundle.resources as Resource<FHIR_VERSION, AllResourceTypes>[],
        );
        setTotal(bundle.total);
        setElapsedMs(Math.round(performance.now() - startedAt));
        setLoading(false);
      })
      .catch((error_) => {
        if (cancelled) return;
        setError(errorMessage(error_));
        setErrorCause(error_);
        // How long a failure took is part of reading it.
        setElapsedMs(Math.round(performance.now() - startedAt));
        setResources([]);
        setTotal(undefined);
        setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [client, fhirVersion, resourceType, query, pageSize, offset, nonce]);

  return {
    resources,
    total,
    loading,
    elapsedMs,
    error,
    errorCause,
    searchParameters,
    refresh: () => setNonce((n) => n + 1),
  };
}
