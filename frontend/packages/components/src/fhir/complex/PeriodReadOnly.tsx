import React from "react";

import { Period } from "@oxidized-health/fhir-types/r4/types";

export type FHIRPeriodReadOnlyProps = {
  value: Period;
};

export const FHIRPeriodReadOnly = ({
  value,
}: Readonly<FHIRPeriodReadOnlyProps>) => {
  return (
    <div className="flex flex-1 space-x-1">
      {value?.start && (
        <div>
          {value.start ?? "any time"} <span>{"->"}</span>{" "}
        </div>
      )}
      {value?.end && <div>{value.end ?? "any time"}</div>}
    </div>
  );
};
