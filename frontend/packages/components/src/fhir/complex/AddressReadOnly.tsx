import React from "react";

import { Address } from "@haste-health/fhir-types/r4/types";

export type FHIRAddressReadonlyProps = {
  value: Address;
};

export const FHIRAddressReadOnly = ({
  value,
}: Readonly<FHIRAddressReadonlyProps>) => {
  const parts = [
    value?.line?.join(" "),
    value?.city,
    value?.state,
    value?.postalCode,
    value?.country,
  ].filter(Boolean);

  return (
    <span className="whitespace-nowrap text-slate-700" title={parts.join(", ")}>
      {parts.join(", ")}
    </span>
  );
};
