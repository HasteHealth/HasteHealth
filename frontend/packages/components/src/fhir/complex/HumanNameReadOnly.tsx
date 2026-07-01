import React from "react";

import { HumanName } from "@haste-health/fhir-types/r4/types";

export type FHIRHumanNameReadOnlyProps = {
  value: HumanName;
};

export const FHIRHumanNameReadOnly = ({
  value,
}: Readonly<FHIRHumanNameReadOnlyProps>) => {
  if (value?.text) {
    return <span className="whitespace-nowrap text-slate-700">{value.text}</span>;
  }

  const lead = [value?.prefix?.join(" "), value?.given?.join(" ")]
    .filter(Boolean)
    .join(" ");

  return (
    <span className="inline-flex items-baseline gap-1 whitespace-nowrap">
      {lead && <span className="text-slate-700">{lead}</span>}
      {value?.family && (
        <span className="font-semibold text-slate-800">{value.family}</span>
      )}
      {value?.suffix && (
        <span className="text-slate-500">{value.suffix.join(" ")}</span>
      )}
    </span>
  );
};
