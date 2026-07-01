import React from "react";

import { CodeableConcept } from "@haste-health/fhir-types/r4/types";

import { FHIRCodingReadOnly } from "./CodingReadOnly";

export type FHIRCodeableConceptReadOnlyProps = {
  value: CodeableConcept;
};

export const FHIRCodeableConceptReadOnly = ({
  value,
}: Readonly<FHIRCodeableConceptReadOnlyProps>) => {
  const codings = value?.coding ?? [];
  const [primary, ...rest] = codings;

  return (
    <span className="inline-flex items-center gap-1.5 whitespace-nowrap">
      {primary ? (
        <FHIRCodingReadOnly value={primary} />
      ) : (
        value?.text && <span className="text-slate-700">{value.text}</span>
      )}
      {value?.text && primary && value.text !== primary.display && (
        <span className="text-[11px] text-slate-400">({value.text})</span>
      )}
      {rest.length > 0 && (
        <span
          className="rounded-full bg-slate-100 px-1.5 text-[10px] leading-4 text-slate-500"
          title={rest.map((c) => c.display ?? c.code).join(", ")}
        >
          +{rest.length}
        </span>
      )}
    </span>
  );
};
