import React from "react";

import { Address } from "@oxidized-health/fhir-types/r4/types";

export type FHIRAddressReadonlyProps = {
  value: Address;
};

export const FHIRAddressReadOnly = ({
  value,
}: Readonly<FHIRAddressReadonlyProps>) => {
  return (
    <div className="flex flex-1 space-x-1">
      {value?.line && <div>{value?.line?.join(" ")}</div>}
      {value?.city && <div>{value?.city}</div>}
      {value?.state && <div>{value?.state}</div>}
      {value?.postalCode && <div>{value?.postalCode}</div>}
      {value?.country && <div>{value?.country}</div>}
    </div>
  );
};
