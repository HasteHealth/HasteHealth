import React from "react";

import { Coding } from "@haste-health/fhir-types/r4/types";

export type FHIRCodingReadOnlyProps = {
  value: Coding;
};

export const FHIRCodingReadOnly = ({
  value,
}: Readonly<FHIRCodingReadOnlyProps>) => {
  if (!value) return null;
  return (
    <span
      className="inline-flex items-center gap-1.5 whitespace-nowrap font-mono text-xs text-[var(--ci-ink)]"
      title={[value.system, value.code].filter(Boolean).join(" | ")}
    >
      {value.code && (
        <span className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1.5 py-0.5 font-mono text-[10px] leading-none text-[var(--ci-signal-ink)]">
          {value.code}
        </span>
      )}
      {value.display && <span className="text-[var(--ci-ink)]">{value.display}</span>}
    </span>
  );
};
