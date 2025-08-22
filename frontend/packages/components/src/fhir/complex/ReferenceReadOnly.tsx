import React from "react";

import { Reference } from "@oxidized-health/fhir-types/r4/types";

export type FHIRReferenceReadOnlyProps = {
  value: Reference;
};

export const FHIRReferenceReadOnly = ({
  value,
}: Readonly<FHIRReferenceReadOnlyProps>) => {
  return <div>{value.display ?? value.reference}</div>;
};
