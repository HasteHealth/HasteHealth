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
    extract(row, column.selector, column.selectorType).then(setValue);
  }, [column, row]);

  const render = useMemo(() => {
    return column.renderer ? column.renderer(value) : value.join(" ");
  }, [column, value]);
  return (
    <td
      key={column.id}
      className="overflow-auto whitespace-nowrap px-4 py-2 font-medium"
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
      <table className="text-left text-xs text-slate-600 w-full">
        <thead className="sticky top-0 z-10 bg-white border-b font-medium">
          <tr>
            {columns.map((column, i) => (
              <th
                onClick={(_e) => column.onClick?.call(undefined, column)}
                key={i}
                className="px-4 py-2 whitespace-nowrap "
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
                className="border cursor-pointer hover:bg-slate-100 focus:outline-none focus-visible:bg-brand-50 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-brand-500"
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
