import React from "react";

import { CodeableConcept } from "@oxidized-health/fhir-types/r4/types";

import { FHIRCodingReadOnly } from "./CodingReadOnly";

export type FHIRCodeableConceptReadOnlyProps = {
  value: CodeableConcept;
};

export const FHIRCodeableConceptReadOnly = ({
  value,
}: Readonly<FHIRCodeableConceptReadOnlyProps>) => {
  return (
    <div className="flex flex-1 space-x-1">
      {value.text && <div>{value?.text}</div>}
      {value?.coding?.map((coding, index) => (
        <div key={index} className="mr-1">
          <FHIRCodingReadOnly key={index} value={coding} />
        </div>
      ))}
    </div>
  );
};
