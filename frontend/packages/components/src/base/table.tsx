import React, { useEffect, useMemo, useState } from "react";

import * as fhirpath from "@oxidized-health/fhirpath";

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
    e: React.MouseEvent<HTMLTableRowElement, MouseEvent>
  ) => void;
}

async function extract(
  data: unknown,
  selector: string,
  selectorType: SelectorType
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
    <div className="w-full">
      <div className="overflow-x-auto overflow-y-auto">
        <table className="text-left text-xs text-slate-600 w-full">
          <thead className="border-b font-medium">
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
                  className="border cursor-pointer hover:bg-slate-100"
                  onClick={(e) => {
                    onRowClick(row, e);
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
    </div>
  );
}
