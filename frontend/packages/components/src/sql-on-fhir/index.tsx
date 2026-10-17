import React, { useMemo, useState } from "react";

import { ViewDefinitionRun } from "@haste-health/generated-ops/r4";
import {
  code,
  instant,
  integer,
  Reference,
  Resource,
  ViewDefinition,
  ViewDefinitionSelect,
} from "@haste-health/fhir-types/r4/types";

import { Button } from "../base/button";
import { CodeMirror } from "../base/codemirror";
import { contentKeys } from "../base/keys";
import { Loading } from "../base/loading";
import { Pagination } from "../base/pagination";
import { Tab, Tabs } from "../base/tabs";
import { ClientProps } from "../fhir/types";
import { basicSetup } from "codemirror";
import { ResponseError } from "@haste-health/client/lib/http";

/** A run's rows as the results table shows them, every cell a string. */
export type ParsedResults = {
  headers: string[];
  rows: string[][];
};

const EMPTY_RESULTS: ParsedResults = { headers: [], rows: [] };

/**
 * The `_format` codes this runner can read, from the specification's output
 * format codes (which also name parquet and fhir).
 */
export type ExportFormat = "csv" | "json" | "ndjson";

/** The media type the specification returns each format as. */
const FORMAT_OF_CONTENT_TYPE = new Map<string, ExportFormat>([
  ["text/csv", "csv"],
  ["application/json", "json"],
  ["application/x-ndjson", "ndjson"],
  ["application/ndjson", "ndjson"],
  ["application/fhir+ndjson", "ndjson"],
]);

/**
 * The format a Binary's contentType says it holds, when it is one this runner
 * reads. Parameters such as `;charset=utf-8` are ignored.
 */
function formatOfContentType(
  contentType: string | undefined,
): ExportFormat | undefined {
  if (!contentType) {
    return undefined;
  }
  return FORMAT_OF_CONTENT_TYPE.get(
    contentType.split(";")[0].trim().toLowerCase(),
  );
}

/**
 * The names of the columns `view` produces, in output order: a select's own
 * columns, then those of its nested selects, then those of its unionAll (all
 * of whose branches share one set). The specification only requires the names
 * to be unique; this is the order the server writes them in.
 */
export function viewDefinitionColumnNames(view: ViewDefinition): string[] {
  return (view.select ?? []).flatMap(selectColumnNames);
}

function selectColumnNames(select: ViewDefinitionSelect): string[] {
  return [
    ...(select.column ?? []).flatMap((column) =>
      column.name ? [column.name] : [],
    ),
    ...(select.select ?? []).flatMap(selectColumnNames),
    ...(select.unionAll?.[0] ? selectColumnNames(select.unionAll[0]) : []),
  ];
}

/** `names` as headers for rows `width` cells wide, numbering any extra cells. */
function headersFor(names: string[], width: number): string[] {
  const headers = [...names];
  for (let i = headers.length; i < width; i += 1) {
    headers.push(`column ${i + 1}`);
  }
  return headers;
}

const EXPORT_FORMAT_OPTIONS: { value: ExportFormat; label: string }[] = [
  { value: "csv", label: "CSV" },
  { value: "json", label: "JSON" },
  { value: "ndjson", label: "NDJSON" },
];

function decodeBase64ToString(value?: string): string {
  if (!value) {
    return "";
  }

  if (typeof atob !== "function") {
    throw new Error("Base64 decoding is unavailable in this environment");
  }

  const binary = atob(value);
  const bytes = Uint8Array.from(binary, (ch) => ch.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

/**
 * Parses csv output. With `columnNames` the run asked for `header=false`, so
 * the csv has no header row and every row is data; otherwise the first row
 * names the columns.
 */
function parseCsv(csv: string, columnNames?: string[]): ParsedResults {
  const rows: string[][] = [];
  let row: string[] = [];
  let current = "";
  let inQuotes = false;

  for (let i = 0; i < csv.length; i += 1) {
    const char = csv[i];
    const next = csv[i + 1];

    if (char === '"' && inQuotes && next === '"') {
      current += '"';
      i += 1;
      continue;
    }

    if (char === '"') {
      inQuotes = !inQuotes;
      continue;
    }

    if (!inQuotes && char === ",") {
      row.push(current);
      current = "";
      continue;
    }

    if (!inQuotes && (char === "\n" || char === "\r")) {
      if (char === "\r" && next === "\n") {
        i += 1;
      }
      row.push(current);
      rows.push(row);
      row = [];
      current = "";
      continue;
    }

    current += char;
  }

  if (current.length > 0 || row.length > 0) {
    row.push(current);
    rows.push(row);
  }

  const cleanedRows = rows.filter((r) =>
    r.some((value) => value.trim().length > 0),
  );
  if (cleanedRows.length === 0) {
    return { headers: columnNames ?? [], rows: [] };
  }

  const [headers, dataRows] = columnNames
    ? [
        headersFor(
          columnNames,
          Math.max(...cleanedRows.map((dataRow) => dataRow.length)),
        ),
        cleanedRows,
      ]
    : [cleanedRows[0], cleanedRows.slice(1)];
  return {
    headers,
    rows: dataRows.map((dataRow) => {
      if (dataRow.length < headers.length) {
        return [...dataRow, ...Array(headers.length - dataRow.length).fill("")];
      }
      return dataRow.slice(0, headers.length);
    }),
  };
}

type RawResultRow = Record<
  string,
  Array<string | number | boolean | null> | number | boolean | null | undefined
>;

function primitiveToString(
  value: string | number | boolean | null | undefined,
): string {
  if (value === null || value === undefined) {
    return "";
  }
  return String(value);
}

function rowsToParsedResults(
  rows: RawResultRow[],
  columnNames: string[],
): ParsedResults {
  if (rows.length === 0) {
    return { headers: columnNames, rows: [] };
  }

  const headers = Object.keys(rows[0]);
  return {
    headers,
    rows: rows.map((row) =>
      headers.map((header) => {
        let value = row[header] ?? [];
        if (!Array.isArray(value)) {
          value = [value];
        }

        return value
          .filter((value) => value !== null && value !== undefined)
          .map(primitiveToString)
          .join(";");
      }),
    ),
  };
}

/** The specification's json output is an array of row objects. */
function parseJson(json: string, columnNames: string[]): ParsedResults {
  if (!json.trim()) {
    return { headers: columnNames, rows: [] };
  }

  const parsed = JSON.parse(json);
  if (!Array.isArray(parsed)) {
    throw new TypeError("Expected JSON output to be an array of rows");
  }

  return rowsToParsedResults(parsed as RawResultRow[], columnNames);
}

/** One row object per line. */
function parseNdjson(ndjson: string, columnNames: string[]): ParsedResults {
  const rows = ndjson
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line) as RawResultRow);

  return rowsToParsedResults(rows, columnNames);
}

interface ParseOptions {
  /** Whether csv output carries its header row (the run's `header`). */
  csvHeader: boolean;
  /** The view's columns, for output that does not name them itself. */
  columnNames: string[];
}

function parseResults(
  format: ExportFormat,
  raw: string,
  options: ParseOptions,
): ParsedResults {
  switch (format) {
    case "json":
      return parseJson(raw, options.columnNames);
    case "ndjson":
      return parseNdjson(raw, options.columnNames);
    case "csv":
      return parseCsv(raw, options.csvHeader ? undefined : options.columnNames);
  }
}

function ScopeControls({
  patientReference,
  setPatientReference,
  groupReferences,
  setGroupReferences,
  limit,
  setLimit,
  includeHeader,
  setIncludeHeader,
  isRunning,
}: Readonly<{
  patientReference: string;
  setPatientReference: (value: string) => void;
  groupReferences: string;
  setGroupReferences: (value: string) => void;
  limit: string;
  setLimit: (value: string) => void;
  includeHeader: boolean;
  setIncludeHeader: (value: boolean) => void;
  isRunning: boolean;
}>) {
  return (
    <details className="border-b border-slate-200 px-4 py-2 text-xs text-slate-600">
      <summary className="cursor-pointer select-none font-medium text-slate-700">
        Scope &amp; output options
      </summary>
      <div className="mt-3 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <label className="flex flex-col gap-1">
          <span>Patient reference</span>
          <input
            type="text"
            className="rounded border border-slate-300 bg-white px-2 py-1 text-xs"
            placeholder="Patient/123"
            value={patientReference}
            onChange={(event) => setPatientReference(event.target.value)}
            disabled={isRunning}
          />
        </label>
        <label className="flex flex-col gap-1">
          <span>Group reference(s)</span>
          <input
            type="text"
            className="rounded border border-slate-300 bg-white px-2 py-1 text-xs"
            placeholder="Group/1, Group/2"
            value={groupReferences}
            onChange={(event) => setGroupReferences(event.target.value)}
            disabled={isRunning}
          />
        </label>
        <label className="flex flex-col gap-1">
          <span>Row limit</span>
          <input
            type="number"
            min={0}
            className="rounded border border-slate-300 bg-white px-2 py-1 text-xs"
            placeholder="No limit"
            value={limit}
            onChange={(event) => setLimit(event.target.value)}
            disabled={isRunning}
          />
        </label>
        <label className="flex items-end gap-2 pb-1">
          <input
            type="checkbox"
            checked={includeHeader}
            onChange={(event) => setIncludeHeader(event.target.checked)}
            disabled={isRunning}
          />
          <span>Include CSV header</span>
        </label>
      </div>
      <p className="mt-2 text-[11px] text-slate-500">
        Restricting to a patient or group only applies when the ViewDefinition's
        resource type is part of the Patient compartment (e.g. Observation,
        Condition) or is Patient itself.
      </p>
    </details>
  );
}

function EditorPane({
  extensions = [basicSetup],
  viewDefinition,
  setViewDefinition,
  exportFormat,
  setExportFormat,
  patientReference,
  setPatientReference,
  groupReferences,
  setGroupReferences,
  limit,
  setLimit,
  includeHeader,
  setIncludeHeader,
  onRun,
  onReset,
  isRunning,
}: {
  extensions?: any[];
  viewDefinition: ViewDefinition;
  setViewDefinition: (value: ViewDefinition) => void;
  exportFormat: ExportFormat;
  setExportFormat: (format: ExportFormat) => void;
  patientReference: string;
  setPatientReference: (value: string) => void;
  groupReferences: string;
  setGroupReferences: (value: string) => void;
  limit: string;
  setLimit: (value: string) => void;
  includeHeader: boolean;
  setIncludeHeader: (value: boolean) => void;
  onRun: () => void;
  onReset: () => void;
  isRunning: boolean;
}) {
  return (
    <section className="flex min-h-[24rem] flex-1 flex-col rounded border border-slate-200 bg-white">
      <header className="flex items-center justify-between border-b border-slate-200 px-4 py-3">
        <h3 className="text-sm font-semibold text-slate-800">
          ViewDefinition Editor
        </h3>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2 text-xs text-slate-600">
            <span>Format</span>
            <select
              className="rounded border border-slate-300 bg-white px-2 py-1 text-xs"
              value={exportFormat}
              onChange={(event) =>
                setExportFormat(event.target.value as ExportFormat)
              }
              disabled={isRunning}
            >
              {EXPORT_FORMAT_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
          <Button
            buttonType="secondary"
            buttonSize="small"
            type="button"
            onClick={onReset}
            disabled={isRunning}
          >
            Reset
          </Button>
          <Button
            buttonType="primary"
            buttonSize="small"
            type="button"
            onClick={onRun}
            disabled={isRunning}
          >
            {isRunning ? "Running..." : "Run"}
          </Button>
        </div>
      </header>
      <ScopeControls
        patientReference={patientReference}
        setPatientReference={setPatientReference}
        groupReferences={groupReferences}
        setGroupReferences={setGroupReferences}
        limit={limit}
        setLimit={setLimit}
        includeHeader={includeHeader}
        setIncludeHeader={setIncludeHeader}
        isRunning={isRunning}
      />
      <div className="flex flex-1 overflow-hidden p-3">
        <div className="h-full w-full overflow-hidden rounded border border-slate-200">
          <CodeMirror
            extensions={extensions}
            value={JSON.stringify(viewDefinition, null, 2)}
            onChange={(v) => {
              try {
                const resource = JSON.parse(v);
                setViewDefinition(resource);
              } catch (e) {
                console.error(e);
                return;
              }
            }}
            theme={{
              "&": {
                height: "100%",
                width: "100%",
                fontSize: "12px",
              },
            }}
          />
        </div>
      </div>
    </section>
  );
}

function ResultsTable({
  headers,
  rows,
}: {
  headers: string[];
  rows: string[][];
}) {
  const rowKeys = contentKeys(rows);
  return (
    <div className="flex-1 overflow-auto">
      <table className="w-full min-w-[42rem] text-left text-xs text-slate-700">
        <thead className="sticky top-0 bg-slate-50">
          <tr>
            {headers.map((header) => (
              <th
                key={header}
                className="border-b border-slate-200 px-3 py-2 font-semibold text-slate-900"
              >
                {header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, rowIndex) => (
            <tr
              key={rowKeys[rowIndex]}
              className="odd:bg-white even:bg-slate-50/40"
            >
              {row.map((value, colIndex) => (
                <td
                  key={`cell-${rowIndex}-${colIndex}`}
                  className="border-b border-slate-100 px-3 py-2 align-top text-slate-700"
                >
                  {value || "-"}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function ResultsPane({
  results,
  isRunning,
  error,
  currentPage,
  setCurrentPage,
  pageSize,
  setPageSize,
  pageSizeOptions,
}: {
  results: ParsedResults;
  isRunning: boolean;
  error?: string;
  currentPage: number;
  setCurrentPage: (page: number) => void;
  pageSize: number;
  setPageSize: (size: number) => void;
  pageSizeOptions: number[];
}) {
  const totalRows = results.rows.length;
  const totalPages = Math.max(1, Math.ceil(totalRows / pageSize));

  const pagedRows = useMemo(() => {
    const start = (currentPage - 1) * pageSize;
    return results.rows.slice(start, start + pageSize);
  }, [currentPage, pageSize, results.rows]);

  return (
    <section className="flex min-h-[24rem] flex-1 flex-col rounded border border-slate-200 bg-white">
      <header className="flex items-center justify-between border-b border-slate-200 px-4 py-3">
        <h3 className="text-sm font-semibold text-slate-800">Results</h3>
        <div className="text-xs text-slate-500">{totalRows} rows</div>
      </header>

      {isRunning && (
        <div className="flex flex-1 items-center justify-center">
          <Loading className="h-7 w-7" />
        </div>
      )}

      {!isRunning && error && (
        <div className="m-4 rounded border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">
          {error}
        </div>
      )}

      {!isRunning && !error && results.headers.length === 0 && (
        <div className="flex flex-1 items-center justify-center px-4 text-sm text-slate-500">
          Run a ViewDefinition to preview tabular output.
        </div>
      )}

      {!isRunning && !error && results.headers.length > 0 && (
        <>
          <ResultsTable headers={results.headers} rows={pagedRows} />
          <footer className="flex items-center justify-between border-t border-slate-200 px-4 py-3">
            <div className="flex items-center gap-2 text-xs text-slate-600">
              <span>Rows per page</span>
              <select
                className="rounded border border-slate-300 bg-white px-2 py-1 text-xs"
                value={pageSize}
                onChange={(event) => {
                  setPageSize(Number(event.target.value));
                  setCurrentPage(1);
                }}
              >
                {pageSizeOptions.map((option) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </select>
            </div>
            <Pagination
              currentPage={currentPage}
              onPagination={setCurrentPage}
              totalPages={totalPages}
            />
          </footer>
        </>
      )}
    </section>
  );
}

/**
 * Everything the runner remembers between renders: what to run with, what
 * the last run produced, and how the results are paged. A host can hold it
 * instead of the runner; see `onStateChange` on the props.
 */
export interface ViewDefinitionRunnerState {
  exportFormat: ExportFormat;
  /** Sent as the run's `header`; only csv output has one. */
  includeHeader: boolean;
  patientReference: string;
  /** Group references, one per line or comma separated. */
  groupReferences: string;
  /** The row limit as typed; blank for none. */
  limit: string;
  isRunning: boolean;
  error?: string;
  results: ParsedResults;
  rawOutput?: string;
  currentPage: number;
  /** Rows per page once chosen; the runner's `defaultPageSize` until then. */
  pageSize?: number;
}

/** The runner's state before anything has been typed or run. */
export const INITIAL_VIEW_DEFINITION_RUNNER_STATE: ViewDefinitionRunnerState = {
  exportFormat: "csv",
  includeHeader: true,
  patientReference: "",
  groupReferences: "",
  limit: "",
  isRunning: false,
  results: EMPTY_RESULTS,
  currentPage: 1,
};

export type ViewDefinitionSqlRunnerProps = ClientProps & {
  editorExtensions?: any[];
  since?: instant;
  resources?: Resource[];
  viewDefinition: ViewDefinition;
  setViewDefinition: (value: ViewDefinition) => void;
  defaultPageSize?: number;
  pageSizeOptions?: number[];
  /**
   * Shows just one pane and hides the runner's own tab strip, for a host that
   * lays the panes out as tabs of its own. Without it the runner renders all
   * three behind its own tabs, as it always has.
   *
   * {@link VIEW_DEFINITION_PANES} names the panes a host can pick from. Such a
   * host mounts a runner per pane, so it must also hold the state through
   * `onStateChange`, or a run's results are lost with the pane that ran it.
   */
  activePane?: number;
  /** Told when the runner wants a different pane shown, e.g. after a run. */
  onActivePaneChange?: (pane: number) => void;
  /**
   * The runner's state as the host holds it; see `onStateChange`. Undefined
   * until the host has been given a change.
   */
  state?: ViewDefinitionRunnerState;
  /**
   * Makes the host hold the state. Every change arrives as an update to
   * apply to the host's current value (undefined before the first one), and
   * the host passes the result back as `state`. Without it the runner keeps
   * its state itself.
   */
  onStateChange?: (
    update: (
      current: ViewDefinitionRunnerState | undefined,
    ) => ViewDefinitionRunnerState,
  ) => void;
};

/** The runner's panes, in order, for a host driving `activePane`. */
export const VIEW_DEFINITION_PANES = [
  { id: 0, title: "ViewDefinition" },
  { id: 1, title: "Results" },
  { id: 2, title: "Raw Output" },
] as const;

export function ViewDefinitionSqlRunner({
  client,
  editorExtensions = [basicSetup],
  fhirVersion,
  resources,
  since,
  viewDefinition,
  setViewDefinition,
  defaultPageSize = 100,
  pageSizeOptions = [10, 20, 50, 100],
  activePane,
  onActivePaneChange,
  state: heldState,
  onStateChange,
}: ViewDefinitionSqlRunnerProps) {
  const [ownActiveTab, setOwnActiveTab] = useState(0);
  // Controlled when the host passes a pane, uncontrolled otherwise.
  const activeTab = activePane ?? ownActiveTab;
  const setActiveTab = (pane: number) => {
    setOwnActiveTab(pane);
    onActivePaneChange?.(pane);
  };

  const [ownState, setOwnState] = useState(
    INITIAL_VIEW_DEFINITION_RUNNER_STATE,
  );
  // Likewise held by the host when it takes the changes.
  const state = onStateChange
    ? (heldState ?? INITIAL_VIEW_DEFINITION_RUNNER_STATE)
    : ownState;
  const update = (change: Partial<ViewDefinitionRunnerState>) => {
    const apply = (current: ViewDefinitionRunnerState | undefined) => ({
      ...(current ?? INITIAL_VIEW_DEFINITION_RUNNER_STATE),
      ...change,
    });
    if (onStateChange) {
      onStateChange(apply);
    } else {
      setOwnState(apply);
    }
  };
  const {
    exportFormat,
    includeHeader,
    patientReference,
    groupReferences,
    limit,
    isRunning,
    error,
    results,
    rawOutput,
    currentPage,
  } = state;
  const pageSize = state.pageSize ?? defaultPageSize;

  async function runViewDefinition() {
    update({ isRunning: true, error: undefined });

    try {
      const trimmedPatientReference = patientReference.trim();
      const groupReferenceList = groupReferences
        .split(/[\n,]+/)
        .map((reference) => reference.trim())
        .filter((reference) => reference.length > 0);
      const parsedLimit = limit.trim() ? Number(limit) : undefined;

      const binary = await client.invoke_system(
        ViewDefinitionRun.Op,
        {},
        fhirVersion,
        {
          _format: exportFormat as code,
          // Only csv output has a header row.
          header: exportFormat === "csv" ? includeHeader : undefined,
          viewResource: viewDefinition,
          resource: resources,
          _since: since,
          patient: trimmedPatientReference
            ? ({ reference: trimmedPatientReference } as Reference)
            : undefined,
          group:
            groupReferenceList.length > 0
              ? groupReferenceList.map(
                  (reference) => ({ reference }) as Reference,
                )
              : undefined,
          _limit:
            parsedLimit !== undefined && !Number.isNaN(parsedLimit)
              ? (parsedLimit as integer)
              : undefined,
        },
      );

      // The Binary says what it holds; it is what was asked for unless the
      // server could not oblige.
      const format = formatOfContentType(binary.contentType) ?? exportFormat;
      const rawData = decodeBase64ToString(binary.data);
      const parsedResults = parseResults(format, rawData, {
        csvHeader: includeHeader,
        columnNames: viewDefinitionColumnNames(viewDefinition),
      });
      update({ results: parsedResults, rawOutput: rawData, currentPage: 1 });
      setActiveTab(1);
    } catch (e) {
      if (e instanceof ResponseError) {
        const errorMessage = e.response?.body?.issue?.[0]?.diagnostics;
        update({
          error: errorMessage ?? "Failed to run ViewDefinition: " + e.message,
          results: EMPTY_RESULTS,
        });
      } else {
        const message =
          e instanceof Error ? e.message : "Failed to run ViewDefinition";
        update({ error: message, results: EMPTY_RESULTS });
      }

      setActiveTab(1);
    } finally {
      update({ isRunning: false });
    }
  }

  function resetEditor() {
    setViewDefinition(viewDefinition);
    update({ error: undefined });
  }

  const tabs: Tab[] = [
    {
      id: 0,
      title: "ViewDefinition",
      content: (
        <EditorPane
          extensions={editorExtensions}
          viewDefinition={viewDefinition}
          setViewDefinition={setViewDefinition}
          exportFormat={exportFormat}
          setExportFormat={(exportFormat) => update({ exportFormat })}
          patientReference={patientReference}
          setPatientReference={(patientReference) =>
            update({ patientReference })
          }
          groupReferences={groupReferences}
          setGroupReferences={(groupReferences) => update({ groupReferences })}
          limit={limit}
          setLimit={(limit) => update({ limit })}
          includeHeader={includeHeader}
          setIncludeHeader={(includeHeader) => update({ includeHeader })}
          onRun={runViewDefinition}
          onReset={resetEditor}
          isRunning={isRunning}
        />
      ),
    },
    {
      id: 1,
      title: "SQL Runner Results",
      content: (
        <ResultsPane
          results={results}
          isRunning={isRunning}
          error={error}
          currentPage={currentPage}
          setCurrentPage={(currentPage) => update({ currentPage })}
          pageSize={pageSize}
          setPageSize={(pageSize) => update({ pageSize })}
          pageSizeOptions={pageSizeOptions}
        />
      ),
    },
    {
      id: 2,
      title: "Raw Output",
      content: (
        <textarea
          className="min-h-[24rem] w-full flex-1 rounded border border-slate-200 bg-white p-3 text-xs text-slate-700"
          value={rawOutput}
          readOnly
        />
      ),
    },
  ];

  // A host with its own tabs shows one pane and supplies the strip itself.
  if (activePane !== undefined) {
    return <>{tabs.find((tab) => tab.id === activePane)?.content ?? null}</>;
  }

  return (
    <Tabs
      tabs={tabs}
      selectedTab={activeTab}
      onTab={(tab) => setActiveTab(Number(tab.id))}
    />
  );
}
