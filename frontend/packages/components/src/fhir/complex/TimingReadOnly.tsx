import React from "react";

import { Timing } from "@oxidized-health/fhir-types/r4/types";

import { FHIRPeriodReadOnly } from "./PeriodReadOnly";
import { FHIRQuantityReadOnly } from "./QuantityReadOnly";
import { FHIRRangeReadOnly } from "./RangeReadOnly";

export type FHIRTimingReadOnlyProps = {
  value: Timing;
};

export const FHIRTimingReadOnly = ({
  value,
}: Readonly<FHIRTimingReadOnlyProps>) => {
  if (value.repeat?.boundsDuration) {
    return <FHIRQuantityReadOnly value={value.repeat?.boundsDuration} />;
  }
  if (value.repeat?.boundsRange) {
    return <FHIRRangeReadOnly value={value.repeat?.boundsRange} />;
  }
  if (value.repeat?.boundsPeriod) {
    return <FHIRPeriodReadOnly value={value.repeat?.boundsPeriod} />;
  }
  return undefined;
};
