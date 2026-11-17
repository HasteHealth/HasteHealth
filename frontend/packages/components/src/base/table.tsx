import React, { useEffect, useMemo, useState } from "react";

import * as fhirpath from "@haste-health/fhirpath";

import { Loading } from "./loading";

export type SelectorType = "fhirpath";

export interface Columns {
  id: string;
  content: React.ReactNode;
  selectorType: SelectorType;
  selector: string;
  onClick?: (column: Columns) => void;
  renderer?: (data: unknown[]) => React.ReactNode;
}

export interface TableProps {
  isLoading?: boolean;
  columns: Columns[];
  data: unknown[];
  onRowClick?: (
    row: unknown,
    e: React.MouseEvent<HTMLTableRowElement, MouseEvent>,
  ) => void;
}

async function extract(
  data: unknown,
  selector: string,
  selectorType: SelectorType,
): Promise<unknown[]> {
  switch (selectorType) {
    case "fhirpath": {
      return fhirpath.evaluate(selector, data);
    }
    default:
      throw new Error(`Unknown selector type: ${selectorType}`);
  }
}

function RenderCell({
  column,
  row,
}: Readonly<{ column: Columns; row: unknown }>) {
  const [value, setValue] = useState<unknown[]>([]);
  useEffect(() => {
    extract(row, column.selector, column.selectorType).then(setValue, (e) => {
      console.error(`Failed to extract column '${column.id}'`, e);
      setValue([]);
    });
  }, [column, row]);

  const render = useMemo(() => {
    return column.renderer ? column.renderer(value) : value.join(" ");
  }, [column, value]);
  return (
    <td
      key={column.id}
      className="overflow-auto whitespace-nowrap px-4 py-2.5 font-mono text-xs text-[var(--ci-ink)]"
    >
      {render}
    </td>
  );
}

export function Table({
  columns,
  data,
  onRowClick = () => {},
  isLoading = false,
}: TableProps) {
  return (
    <div className="overflow-x-auto overflow-y-auto">
      <table className="text-left text-xs text-[var(--ci-ink)] w-full border-collapse">
        <thead className="sticky top-0 z-10 bg-[var(--ci-surface-raised)] border-b border-[var(--ci-line)] font-mono text-[11px] uppercase tracking-wider text-[var(--ci-muted)]">
          <tr>
            {columns.map((column, i) => (
              <th
                onClick={(_e) => column.onClick?.call(undefined, column)}
                key={i}
                className="px-4 py-2.5 whitespace-nowrap font-medium"
              >
                {column.content}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {!isLoading &&
            data.map((row, index) => (
              <tr
                key={index}
                // Keyboard reachable, so results can be worked without a
                // mouse. The row keeps its `row` role: overriding it with
                // `button` would hide the cells from a screen reader.
                tabIndex={0}
                data-table-row
                className="border-b border-[var(--ci-line)] cursor-pointer hover:bg-[var(--ci-surface-raised)] transition-colors focus:outline-none focus-visible:bg-[var(--ci-surface-raised)] focus-visible:ring-1 focus-visible:ring-[var(--ci-signal)]"
                onClick={(e) => {
                  onRowClick(row, e);
                }}
                onKeyDown={(e) => {
                  if (e.key !== "Enter" && e.key !== " ") return;
                  e.preventDefault();
                  onRowClick(
                    row,
                    e as unknown as React.MouseEvent<HTMLTableRowElement>,
                  );
                }}
              >
                {columns.map((column) => (
                  <RenderCell key={column.id} row={row} column={column} />
                ))}
              </tr>
            ))}
        </tbody>
      </table>
      {isLoading && (
        <div className="w-full mt-4 flex justify-center items-center flex-col">
          <Loading className="w-6 h-6" />
        </div>
      )}
    </div>
  );
}
