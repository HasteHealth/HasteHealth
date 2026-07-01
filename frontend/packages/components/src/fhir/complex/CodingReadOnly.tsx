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
      className="inline-flex items-center gap-1.5 whitespace-nowrap"
      title={[value.system, value.code].filter(Boolean).join(" | ")}
    >
      {value.code && (
        <span className="rounded bg-indigo-50 px-1.5 py-0.5 font-mono text-[11px] leading-none text-indigo-700">
          {value.code}
        </span>
      )}
      {value.display && (
        <span className="text-slate-700">{value.display}</span>
      )}
    </span>
  );
};
