import React from "react";

import { Identifier } from "@haste-health/fhir-types/r4/types";

export type FHIRIdentifierReadOnlyProps = {
  value: Identifier;
};

export const FHIRIdentifierReadOnly = ({
  value,
}: Readonly<FHIRIdentifierReadOnlyProps>) => {
  return (
    <span
      className="inline-flex items-center gap-1.5 whitespace-nowrap font-mono text-xs text-[var(--ci-ink)]"
      title={value?.system}
    >
      {value?.type?.text && (
        <span className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1.5 py-0.5 text-[10px] font-mono uppercase tracking-wider text-[var(--ci-muted)]">
          {value.type.text}
        </span>
      )}
      {value?.value && (
        <span className="font-mono text-[var(--ci-ink)]">{value.value}</span>
      )}
    </span>
  );
};
