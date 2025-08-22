import React from "react";

import { Range } from "@oxidized-health/fhir-types/r4/types";

import { FHIRQuantityReadOnly } from "./QuantityReadOnly";

export type FHIRRangeReadOnlyProps = {
  value: Range;
};

export const FHIRRangeReadOnly = ({
  value,
}: Readonly<FHIRRangeReadOnlyProps>) => {
  return (
    <div className="flex flex-1 space-x-1">
      {value?.low && (
        <div>{value.low && <FHIRQuantityReadOnly value={value.low} />}</div>
      )}
      <div>{"->"}</div>
      {value.high && (
        <div>{value.high && <FHIRQuantityReadOnly value={value.high} />}</div>
      )}
    </div>
  );
};
