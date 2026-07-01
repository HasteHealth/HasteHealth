import React from "react";

import { Period } from "@haste-health/fhir-types/r4/types";

export type FHIRPeriodReadOnlyProps = {
  value: Period;
};

export const FHIRPeriodReadOnly = ({
  value,
}: Readonly<FHIRPeriodReadOnlyProps>) => {
  if (!value?.start && !value?.end) return null;
  return (
    <span className="inline-flex items-center gap-1 whitespace-nowrap font-mono text-[11px] text-slate-600">
      <span>{value.start ?? "—"}</span>
      <span className="text-slate-300">{"→"}</span>
      <span>{value.end ?? "present"}</span>
    </span>
  );
};
