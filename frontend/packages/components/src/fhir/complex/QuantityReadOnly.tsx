import React from "react";

import { Quantity } from "@haste-health/fhir-types/r4/types";

export type FHIRQuantityReadOnlyProps = {
  value: Quantity;
};

export const FHIRQuantityReadOnly = ({
  value,
}: Readonly<FHIRQuantityReadOnlyProps>) => {
  return (
    <span className="inline-flex items-baseline gap-1 whitespace-nowrap text-[var(--ci-ink)]">
      {value?.value !== undefined && (
        <span className="font-mono tabular-nums">
          {value.value}
        </span>
      )}
      {value?.unit && (
        <span className="text-[11px] text-[var(--ci-muted)]">{value.unit}</span>
      )}
    </span>
  );
};
