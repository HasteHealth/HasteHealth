import React from "react";

import { Range } from "@haste-health/fhir-types/r4/types";

import { FHIRQuantityReadOnly } from "./QuantityReadOnly";

export type FHIRRangeReadOnlyProps = {
  value: Range;
};

export const FHIRRangeReadOnly = ({
  value,
}: Readonly<FHIRRangeReadOnlyProps>) => {
  if (!value?.low && !value?.high) return null;
  return (
    <span className="inline-flex items-center gap-1 whitespace-nowrap text-[11px] text-[var(--ci-ink)]">
      {value.low ? <FHIRQuantityReadOnly value={value.low} /> : <span>—</span>}
      <span className="text-[var(--ci-muted)]">{"–"}</span>
      {value.high ? (
        <FHIRQuantityReadOnly value={value.high} />
      ) : (
        <span>—</span>
      )}
    </span>
  );
};
