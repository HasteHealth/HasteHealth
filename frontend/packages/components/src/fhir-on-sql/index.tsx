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
import { ClientProps } from "../fhir/types";
import { basicSetup } from "codemirror";
import { ResponseError } from "@haste-health/client/lib/http";

const DEFAULT_VIEW_DEFINITION: ViewDefinition = {
  resourceType: "ViewDefinition",
  status: "draft",
  resource: "Patient",
  select: [
    {
      column: [
        {
          name: "id",
          path: "id",
          type: "id",
        },
        {
          name: "date of birth",
          path: "$this.birthDate",
          type: "date",
        },
      ],
    },
    {
      forEach: "$this.name",
      column: [
        {
          name: "name",
          path: "$this.given",
          type: "string",
        },
        {
          name: "family",
          path: "$this.family",
          type: "string",
        },
      ],
    },
  ],
} as ViewDefinition;

type ParsedResults = {
  headers: string[];
  rows: string[][];
};

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

function EditorPane({
  extensions = [basicSetup],
  viewDefinition,
  setViewDefinition,
  onRun,
  onReset,
  isRunning,
}: {
  extensions?: any[];
  viewDefinition: string;
  setViewDefinition: (value: string) => void;
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
        <div className="flex items-center gap-2">
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
            value={viewDefinition}
            onChange={(value) => setViewDefinition(value)}
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
  className?: string;
  editorExtensions?: any[];
  since?: instant;
  resources?: Resource[];
  initialViewDefinition?: ViewDefinition;
  defaultPageSize?: number;
  pageSizeOptions?: number[];
};

export function ViewDefinitionSqlRunner({
  client,
  editorExtensions = [basicSetup],
  fhirVersion,
  className,
  resources,
  since,
  initialViewDefinition,
  defaultPageSize = 20,
  pageSizeOptions = [10, 20, 50, 100],
}: ViewDefinitionSqlRunnerProps) {
  const [viewDefinition, setViewDefinition] = useState(() =>
    JSON.stringify(initialViewDefinition ?? DEFAULT_VIEW_DEFINITION, null, 2),
  );
  const [results, setResults] = useState<ParsedResults>({
    headers: [],
    rows: [],
  });
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string>();
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(defaultPageSize);

  async function runViewDefinition() {
    setIsLoading(true);
    setError(undefined);

    try {
      const parsedViewDefinition = JSON.parse(viewDefinition) as ViewDefinition;
      const binary = await client.invoke_system(
        ViewDefinitionRun.Op,
        {},
        fhirVersion,
        {
          _format: "csv" as code,
          header: true,
          viewResource: parsedViewDefinition,
          resource: resources,
          _since: since,
        },
      );

      console.log("ViewDefinitionRun output:", binary);

      const csv = decodeBase64ToString(binary.data);
      const parsedResults = parseCsv(csv);
      setResults(parsedResults);
      setCurrentPage(1);
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
    } finally {
      setIsLoading(false);
    }
  }

  function resetEditor() {
    setViewDefinition(
      JSON.stringify(initialViewDefinition ?? DEFAULT_VIEW_DEFINITION, null, 2),
    );
    setError(undefined);
  }

  return (
    <div className={className}>
      <div className="flex flex-col gap-4 lg:flex-row">
        <EditorPane
          extensions={editorExtensions}
          viewDefinition={viewDefinition}
          setViewDefinition={setViewDefinition}
          onRun={runViewDefinition}
          onReset={resetEditor}
          isRunning={isLoading}
        />
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
      </div>
    </div>
  );
}
