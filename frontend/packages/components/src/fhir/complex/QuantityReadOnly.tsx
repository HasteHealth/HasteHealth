import React from "react";

import { Quantity } from "@oxidized-health/fhir-types/r4/types";

export type FHIRQuantityReadOnlyProps = {
  value: Quantity;
};

export const FHIRQuantityReadOnly = ({
  value,
}: Readonly<FHIRQuantityReadOnlyProps>) => {
  return (
    <div className="flex flex-1 space-x-1">
      {value?.value && <div>{value.value}</div>}
      {value?.unit && <div>{value.unit}</div>}
    </div>
  );
};
