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
      className="inline-flex items-center gap-1.5 whitespace-nowrap"
      title={value?.system}
    >
      {value?.type?.text && (
        <span className="rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-slate-500">
          {value.type.text}
        </span>
      )}
      {value?.value && (
        <span className="font-mono text-slate-700">{value.value}</span>
      )}
    </span>
  );
};
