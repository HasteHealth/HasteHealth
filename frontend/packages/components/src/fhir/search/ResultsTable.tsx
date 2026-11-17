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
                className="flex w-full items-center gap-1 text-left font-mono text-xs uppercase tracking-wider text-[var(--ci-muted)] hover:text-[var(--ci-signal)] transition-colors"
                onClick={() => onSortChange(nextSort(code, sort))}
                type="button"
                title={`Sort by ${code}`}
              >
                <span>{code}</span>
                <span className="text-xs text-[var(--ci-signal)]">
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
        <span className="flex items-center gap-2 text-xs font-mono text-[var(--ci-muted)]">
          <span>
            {resultSummary(resources.length, offset, total)}
          </span>
          {elapsedMs !== undefined && (
            <span
              className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--ci-ink)]"
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
        <div className="flex flex-1 flex-col items-center justify-center gap-2 rounded-none border border-dashed border-[var(--ci-line)] bg-[var(--ci-surface)] py-12 text-center">
          <MagnifyingGlassIcon className="h-8 w-8 text-[var(--ci-muted)] opacity-50" />
          <p className="text-sm font-mono font-medium text-[var(--ci-ink)]">No matches</p>
          <p className="max-w-sm text-xs font-mono text-[var(--ci-muted)]">
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
        className="inline-flex items-center gap-1.5 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] px-2.5 py-1 text-xs font-mono uppercase tracking-wider text-[var(--ci-ink)] hover:border-[var(--ci-signal)] transition-colors"
        onClick={() => setOpen((isOpen) => !isOpen)}
        type="button"
      >
        <Bars3BottomLeftIcon className="h-3.5 w-3.5 text-[var(--ci-signal)]" />
        <span>Columns</span>
        <ChevronUpDownIcon className="h-3.5 w-3.5 text-[var(--ci-muted)]" />
      </button>
      {open && (
        <div className="absolute right-0 z-20 mt-1 max-h-80 w-64 overflow-auto rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] py-1 shadow-2xl">
          {parameters.map((parameter) => {
            const code = parameter.code as string;
            return (
              <label
                key={code}
                className="flex cursor-pointer items-center gap-2 px-3 py-1.5 text-xs font-mono text-[var(--ci-ink)] hover:bg-[var(--ci-surface-raised)] transition-colors"
              >
                <input
                  type="checkbox"
                  checked={visible.includes(code)}
                  onChange={() => toggle(code)}
                  className="rounded-none border-[var(--ci-line)] text-[var(--ci-signal)] focus:ring-[var(--ci-signal)]"
                />
                <span className="flex-1 truncate">{code}</span>
                <span className="text-[11px] font-mono text-[var(--ci-muted)]">
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
