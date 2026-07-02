import React, { useMemo, useState } from "react";

import { ViewDefinitionRun } from "@haste-health/generated-ops/r4";
import {
  code,
  instant,
  Resource,
  ViewDefinition,
} from "@haste-health/fhir-types/r4/types";

import { Button } from "../base/button";
import { CodeMirror } from "../base/codemirror";
import { Loading } from "../base/loading";
import { Pagination } from "../base/pagination";
import { Tab, Tabs } from "../base/tabs";
import { ClientProps } from "../fhir/types";
import { basicSetup } from "codemirror";
import { ResponseError } from "@haste-health/client/lib/http";

type ParsedResults = {
  headers: string[];
  rows: string[][];
};

type ExportFormat = "csv" | "json" | "ndjson";

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

function parseCsv(csv: string): ParsedResults {
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
    return { headers: [], rows: [] };
  }

  const [headers, ...dataRows] = cleanedRows;
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

function rowsToParsedResults(rows: RawResultRow[]): ParsedResults {
  if (rows.length === 0) {
    return { headers: [], rows: [] };
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

function parseJson(json: string): ParsedResults {
  if (!json.trim()) {
    return { headers: [], rows: [] };
  }

  const parsed = JSON.parse(json);
  if (!Array.isArray(parsed)) {
    throw new TypeError("Expected JSON output to be an array of rows");
  }

  return rowsToParsedResults(parsed as RawResultRow[]);
}

function parseNdjson(ndjson: string): ParsedResults {
  const rows = ndjson
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .map((line) => JSON.parse(line) as RawResultRow);

  return rowsToParsedResults(rows);
}

function parseResults(format: ExportFormat, raw: string): ParsedResults {
  switch (format) {
    case "json":
      return parseJson(raw);
    case "ndjson":
      return parseNdjson(raw);
    case "csv":
      return parseCsv(raw);
  }
}

function EditorPane({
  extensions = [basicSetup],
  viewDefinition,
  setViewDefinition,
  exportFormat,
  setExportFormat,
  onRun,
  onReset,
  isRunning,
}: {
  extensions?: any[];
  viewDefinition: ViewDefinition;
  setViewDefinition: (value: ViewDefinition) => void;
  exportFormat: ExportFormat;
  setExportFormat: (format: ExportFormat) => void;
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
              key={`row-${rowIndex}`}
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

export type ViewDefinitionSqlRunnerProps = ClientProps & {
  editorExtensions?: any[];
  since?: instant;
  resources?: Resource[];
  viewDefinition: ViewDefinition;
  setViewDefinition: (value: ViewDefinition) => void;
  defaultPageSize?: number;
  pageSizeOptions?: number[];
};

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
}: ViewDefinitionSqlRunnerProps) {
  const [activeTab, setActiveTab] = useState(0);
  const [results, setResults] = useState<ParsedResults>({
    headers: [],
    rows: [],
  });
  const [rawOutput, setRawOutput] = useState<string>();
  const [exportFormat, setExportFormat] = useState<ExportFormat>("csv");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string>();
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(defaultPageSize);

  async function runViewDefinition() {
    setIsLoading(true);
    setError(undefined);

    try {
      const binary = await client.invoke_system(
        ViewDefinitionRun.Op,
        {},
        fhirVersion,
        {
          _format: exportFormat as code,
          header: true,
          viewResource: viewDefinition,
          resource: resources,
          _since: since,
        },
      );

      const rawData = decodeBase64ToString(binary.data);
      const parsedResults = parseResults(exportFormat, rawData);
      setResults(parsedResults);
      setRawOutput(rawData);
      setCurrentPage(1);
      setActiveTab(1);
    } catch (e) {
      if (e instanceof ResponseError) {
        const errorMessage = e.response?.body?.issue?.[0]?.diagnostics;
        setError(errorMessage ?? "Failed to run ViewDefinition: " + e.message);
      } else {
        const message =
          e instanceof Error ? e.message : "Failed to run ViewDefinition";
        setError(message);
      }

      setResults({ headers: [], rows: [] });
      setActiveTab(1);
    } finally {
      setIsLoading(false);
    }
  }

  function resetEditor() {
    setViewDefinition(viewDefinition);
    setError(undefined);
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
          setExportFormat={setExportFormat}
          onRun={runViewDefinition}
          onReset={resetEditor}
          isRunning={isLoading}
        />
      ),
    },
    {
      id: 1,
      title: "SQL Runner Results",
      content: (
        <ResultsPane
          results={results}
          isRunning={isLoading}
          error={error}
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          pageSize={pageSize}
          setPageSize={setPageSize}
          pageSizeOptions={pageSizeOptions}
        />
      ),
    },
    {
      id: 2,
      title: "Raw Output",
      content: (
        <textarea
          className="h-full w-full rounded border border-slate-200 bg-white p-3 text-xs text-slate-700"
          value={rawOutput}
          readOnly
        />
      ),
    },
  ];

  return (
    <Tabs
      tabs={tabs}
      selectedTab={activeTab}
      onTab={(tab) => setActiveTab(Number(tab.id))}
    />
  );
}
