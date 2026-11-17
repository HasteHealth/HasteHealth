import React from "react";

import { HumanName } from "@haste-health/fhir-types/r4/types";

export type FHIRHumanNameReadOnlyProps = {
  value: HumanName;
};

export const FHIRHumanNameReadOnly = ({
  value,
}: Readonly<FHIRHumanNameReadOnlyProps>) => {
  if (value?.text) {
    return (
      <span className="whitespace-nowrap text-[var(--ci-ink)]">{value.text}</span>
    );
  }

  const lead = [value?.prefix?.join(" "), value?.given?.join(" ")]
    .filter(Boolean)
    .join(" ");

  return (
    <span className="inline-flex items-baseline gap-1 whitespace-nowrap text-[var(--ci-ink)]">
      {lead && <span>{lead}</span>}
      {value?.family && (
        <span className="font-semibold">{value.family}</span>
      )}
      {value?.suffix && (
        <span className="text-[var(--ci-muted)]">{value.suffix.join(" ")}</span>
      )}
    </span>
  );
};
