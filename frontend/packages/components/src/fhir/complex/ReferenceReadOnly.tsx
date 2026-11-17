import React from "react";

import { Reference } from "@haste-health/fhir-types/r4/types";

export type FHIRReferenceReadOnlyProps = {
  value: Reference;
};

export const FHIRReferenceReadOnly = ({
  value,
}: Readonly<FHIRReferenceReadOnlyProps>) => {
  const [resourceType, id] = (value?.reference ?? "").split("/");

  return (
    <span
      className="inline-flex items-center gap-1.5 whitespace-nowrap font-mono text-xs text-[var(--ci-ink)]"
      title={value?.reference}
    >
      {resourceType && (
        <span className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1.5 py-0.5 text-[10px] font-mono uppercase tracking-wider text-[var(--ci-signal-ink)]">
          {resourceType}
        </span>
      )}
      <span className="text-[var(--ci-ink)]">
        {value?.display ?? id ?? value?.reference}
      </span>
    </span>
  );
};
