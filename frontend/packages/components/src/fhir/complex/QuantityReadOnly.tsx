import React from "react";

import { Quantity } from "@haste-health/fhir-types/r4/types";

export type FHIRQuantityReadOnlyProps = {
  value: Quantity;
};

export const FHIRQuantityReadOnly = ({
  value,
}: Readonly<FHIRQuantityReadOnlyProps>) => {
  return (
    <span className="inline-flex items-baseline gap-1 whitespace-nowrap">
      {value?.value !== undefined && (
        <span className="font-mono tabular-nums text-slate-800">
          {value.value}
        </span>
      )}
      {value?.unit && (
        <span className="text-[11px] text-slate-400">{value.unit}</span>
      )}
    </span>
  );
};
