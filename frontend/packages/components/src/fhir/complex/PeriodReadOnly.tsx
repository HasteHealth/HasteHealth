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
    <span className="inline-flex items-center gap-1 whitespace-nowrap font-mono text-[11px] text-[var(--ci-ink)]">
      <span>{value.start ?? "—"}</span>
      <span className="text-[var(--ci-muted)]">{"→"}</span>
      <span>{value.end ?? "present"}</span>
    </span>
  );
};
