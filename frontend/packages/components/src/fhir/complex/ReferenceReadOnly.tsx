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
      className="inline-flex items-center gap-1.5 whitespace-nowrap"
      title={value?.reference}
    >
      {resourceType && (
        <span className="rounded bg-brand-50 px-1.5 py-0.5 text-[10px] font-medium leading-none text-brand-700">
          {resourceType}
        </span>
      )}
      <span className="text-slate-700">
        {value?.display ?? id ?? value?.reference}
      </span>
    </span>
  );
};
