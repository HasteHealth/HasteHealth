import {
  Bars3BottomLeftIcon,
  ChevronUpDownIcon,
  MagnifyingGlassIcon,
} from "@heroicons/react/24/outline";
import React, { useEffect, useMemo, useRef, useState } from "react";

import { Pagination } from "../../base/pagination";
import { Table, TableProps } from "../../base/table";
import { DataDisplay } from "../generative/searchTable";
import { SearchParameterResource } from "./metadata";

export interface FHIRResultsTableProps {
  /** Search parameters for the searched type; the columns come from these. */
  searchParameters: SearchParameterResource[];
  /** Codes to show as columns, in order. The rest stay in the picker. */
  visibleColumns: string[];
  onVisibleColumnsChange: (codes: string[]) => void;
  resources: unknown[];
  total?: number;
  loading?: boolean;
  /** Request time, shown beside the row count. */
  elapsedMs?: number;
  /** Rows per page, and the offset shown. */
  pageSize: number;
  offset: number;
  onOffsetChange: (offset: number) => void;
  /** Sort code, `-` prefixed for descending, as `_sort` carries it. */
  sort?: string;
  onSortChange: (sort: string | undefined) => void;
  onRowClick: TableProps["onRowClick"];
  /** Columns appended after the generated ones. */
  columns?: TableProps["columns"];
}

/** The row count line, e.g. `Showing 1 to 20 of 137`. */
function resultSummary(
  count: number,
  offset: number,
  total: number | undefined,
) {
  if (count === 0) return "No results";
  const range = `Showing ${offset + 1} to ${offset + count}`;
  return total === undefined ? range : `${range} of ${total}`;
}

/** Cycles ascending, descending, unsorted. */
function nextSort(code: string, current: string | undefined) {
  if (current === code) return `-${code}`;
  if (current === `-${code}`) return undefined;
  return code;
}

/** Arrow for the current sort. */
function sortLabel(code: string, current: string | undefined) {
  if (current === code) return "▲";
  if (current === `-${code}`) return "▼";
  return "";
}

/**
 * A table whose columns are generated from the searched type's search
 * parameters, each read with that parameter's own FHIRPath expression. Any
 * type the server can search displays with no per resource configuration.
 *
 * Which columns are shown is the caller's to persist.
 */
export function FHIRResultsTable({
  searchParameters,
  visibleColumns,
  onVisibleColumnsChange,
  resources,
  total,
  loading,
  elapsedMs,
  pageSize,
  offset,
  onOffsetChange,
  sort,
  onSortChange,
  onRowClick,
  columns,
}: Readonly<FHIRResultsTableProps>) {
  // Only an expression can be read out of a resource.
  const displayable = useMemo(
    () =>
      searchParameters.filter(
        (parameter) =>
          parameter.expression &&
          parameter.type !== "composite" &&
          parameter.type !== "special",
      ),
    [searchParameters],
  );

  const generated = useMemo(
    () =>
      visibleColumns
        .map((code) => displayable.find((p) => p.code === code))
        .filter((p): p is SearchParameterResource => p !== undefined)
        .map((parameter) => {
          const code = parameter.code as string;
          return {
            id: code,
            content: (
              <button
                className="flex w-full items-center gap-1 text-left hover:text-brand-600"
                onClick={() => onSortChange(nextSort(code, sort))}
                type="button"
                title={`Sort by ${code}`}
              >
                <span>{code}</span>
                <span className="text-xs text-slate-400">
                  {sortLabel(code, sort)}
                </span>
              </button>
            ),
            selector: parameter.expression as string,
            selectorType: "fhirpath" as const,
            renderer: (data: unknown[]) =>
              DataDisplay(parameter.type as string, data.slice(0, 1)),
          };
        }),
    [displayable, visibleColumns, sort, onSortChange],
  );

  const totalPages = Math.max(1, Math.ceil((total ?? 0) / pageSize));
  const currentPage = Math.floor(offset / pageSize) + 1;

  return (
    <div className="flex h-full w-full flex-col">
      <div className="mb-2 flex items-center justify-between gap-2">
        <span className="flex items-center gap-2 text-xs text-slate-500">
          <span>
            {resultSummary(resources.length, offset, total)}
          </span>
          {elapsedMs !== undefined && (
            <span
              className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[11px] text-slate-600"
              title="Round trip time as the browser measured it"
            >
              {elapsedMs} ms
            </span>
          )}
        </span>
        <ColumnPicker
          parameters={displayable}
          visible={visibleColumns}
          onChange={onVisibleColumnsChange}
        />
      </div>

      {/* An empty table renders as nothing, which reads as a broken page. */}
      {!loading && resources.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center gap-2 rounded-md border border-dashed border-slate-200 py-12 text-center">
          <MagnifyingGlassIcon className="h-8 w-8 text-slate-300" />
          <p className="text-sm font-medium text-slate-600">No matches</p>
          <p className="max-w-sm text-xs text-slate-400">
            {offset > 0
              ? "There is nothing on this page. Go back to the first page to see earlier results."
              : "The search ran and returned no resources. Try removing a parameter, or loosening one with a modifier such as :contains."}
          </p>
        </div>
      ) : (
        <Table
          isLoading={loading}
          data={resources}
          onRowClick={onRowClick}
          columns={[...generated, ...(columns ?? [])]}
        />
      )}

      {/* Paging is meaningless on an empty first page. */}
      {(resources.length > 0 || offset > 0) && (
        <div className="mt-2 flex items-center justify-end">
          <Pagination
            currentPage={currentPage}
            totalPages={totalPages}
            onPagination={(page) => onOffsetChange((page - 1) * pageSize)}
          />
        </div>
      )}
    </div>
  );
}

/**
 * Chooses which parameters appear as columns. A popover rather than a menu,
 * which would close on each pick; several in a row is the normal case.
 */
function ColumnPicker({
  parameters,
  visible,
  onChange,
}: Readonly<{
  parameters: SearchParameterResource[];
  visible: string[];
  onChange: (codes: string[]) => void;
}>) {
  const [open, setOpen] = useState(false);
  const container = useRef<HTMLDivElement | null>(null);

  // Close on a click outside.
  useEffect(() => {
    if (!open) return undefined;
    const onDocumentClick = (event: MouseEvent) => {
      if (!container.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener("mousedown", onDocumentClick);
    return () => document.removeEventListener("mousedown", onDocumentClick);
  }, [open]);

  const toggle = (code: string) => {
    // Keep parameter order, rather than moving a re-enabled column last.
    onChange(
      visible.includes(code)
        ? visible.filter((c) => c !== code)
        : parameters
            .map((p) => p.code as string)
            .filter((c) => c === code || visible.includes(c)),
    );
  };

  return (
    <div className="relative" ref={container}>
      <button
        className="inline-flex items-center gap-1 rounded-md border border-slate-200 px-2 py-1 text-xs text-slate-600 hover:bg-slate-50"
        onClick={() => setOpen((isOpen) => !isOpen)}
        type="button"
      >
        <Bars3BottomLeftIcon className="h-4 w-4" />
        Columns
        <ChevronUpDownIcon className="h-4 w-4" />
      </button>
      {open && (
        <div className="absolute right-0 z-20 mt-1 max-h-80 w-64 overflow-auto rounded-md border border-slate-200 bg-white py-1 shadow-lg">
          {parameters.map((parameter) => {
            const code = parameter.code as string;
            return (
              <label
                key={code}
                className="flex cursor-pointer items-center gap-2 px-3 py-1.5 text-sm hover:bg-slate-100"
              >
                <input
                  type="checkbox"
                  checked={visible.includes(code)}
                  onChange={() => toggle(code)}
                />
                <span className="flex-1 truncate">{code}</span>
                <span className="text-xs text-slate-400">
                  {parameter.type as string}
                </span>
              </label>
            );
          })}
        </div>
      )}
    </div>
  );
}
