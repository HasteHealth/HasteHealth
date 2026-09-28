import { useAtomValue, useSetAtom } from "jotai";
import React, { useCallback, useMemo } from "react";
import { useNavigate } from "react-router";

import {
  FHIRResultsTable,
  OutcomePanel,
  toOutcome,
  useSearch,
} from "@haste-health/components";
import {
  AllResourceTypes,
  R4,
  Resource,
} from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { Command, describeRequest } from "../../query/model";
import { columnsAtom, setColumnsAtom } from "../../query/atoms";

const PAGE_SIZE = 20;

/**
 * Columns shown for a type the user has not chosen columns for yet.
 *
 * Content first, metadata last: a table that opens on ids and timestamps
 * reads as machine output, where names and dates read as the records
 * themselves. Only the codes a type actually has become columns, so this is a
 * preference order rather than a fixed set.
 */
const PREFERRED_COLUMNS = [
  "name",
  "title",
  "code",
  "status",
  "identifier",
  "subject",
  "patient",
  "address",
  "birthdate",
  "date",
];

/** Shown after the preferred ones, when nothing better is available. */
const FALLBACK_COLUMNS = ["_id", "_lastUpdated"];

/** How many columns to show before the user picks their own. */
const DEFAULT_COLUMN_COUNT = 6;

/**
 * The columns to open with for `resourceType`, given the parameters it
 * actually supports.
 */
function defaultColumns(available: string[]): string[] {
  const has = new Set(available);
  const preferred = PREFERRED_COLUMNS.filter((code) => has.has(code));
  // Anything left over, so a type with none of the preferred codes still
  // shows something about itself rather than only its id.
  const rest = available.filter(
    (code) =>
      !preferred.includes(code) &&
      !FALLBACK_COLUMNS.includes(code) &&
      !code.startsWith("_"),
  );
  const chosen = [...preferred, ...rest].slice(0, DEFAULT_COLUMN_COUNT);
  return chosen.length > 0 ? chosen : FALLBACK_COLUMNS;
}

export interface ResultsPanelProps {
  command: Command;
  resourceType: string;
  /** Search text after the `?`, without paging. */
  search: string;
  offset: number;
  onOffsetChange: (offset: number) => void;
  onSortChange: (sort: string | undefined) => void;
  sort?: string;
}

/**
 * A search's results: the auto generated table, or the server's complaint
 * about the query in place of it.
 */
export function ResultsPanel({
  command,
  resourceType,
  search,
  offset,
  onOffsetChange,
  onSortChange,
  sort,
}: Readonly<ResultsPanelProps>) {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();

  const {
    resources,
    total,
    loading,
    elapsedMs,
    error,
    errorCause,
    searchParameters,
  } = useSearch({
    client,
    fhirVersion: R4,
    resourceType,
    query: search,
    pageSize: PAGE_SIZE,
    offset,
  });

  // Column choices persist per resource type through the storage atom.
  const allColumns = useAtomValue(columnsAtom);
  const setColumns = useSetAtom(setColumnsAtom);
  // Recomputed when the parameters arrive, since which columns exist depends
  // on the type. A saved choice always wins.
  const columns = useMemo(
    () =>
      allColumns[resourceType] ??
      defaultColumns(searchParameters.map((p) => p.code as string)),
    [allColumns, resourceType, searchParameters],
  );

  const onColumnsChange = useCallback(
    (codes: string[]) => setColumns(resourceType, codes),
    [setColumns, resourceType],
  );

  const request = useMemo(() => describeRequest(command), [command]);

  if (error) {
    // Prefer the thrown error, which still carries the OperationOutcome, and
    // fall back to the flattened message for a failure that had none.
    return (
      <OutcomePanel
        outcome={errorCause ? toOutcome(errorCause) : { message: error }}
        request={request}
      />
    );
  }

  return (
    <FHIRResultsTable
      searchParameters={searchParameters}
      visibleColumns={columns}
      onVisibleColumnsChange={onColumnsChange}
      resources={resources}
      total={total}
      loading={loading}
      elapsedMs={elapsedMs}
      pageSize={PAGE_SIZE}
      offset={offset}
      onOffsetChange={onOffsetChange}
      sort={sort}
      onSortChange={onSortChange}
      onRowClick={(row) => {
        const id = (row as Resource<R4, AllResourceTypes>).id;
        if (id) navigate(`/r/${resourceType}/${id}`);
      }}
    />
  );
}
