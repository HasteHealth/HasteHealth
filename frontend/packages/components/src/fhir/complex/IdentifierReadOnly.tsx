import React from "react";

import { Identifier } from "@haste-health/fhir-types/r4/types";

export type FHIRIdentifierReadOnlyProps = {
  value: Identifier;
};

export const FHIRIdentifierReadOnly = ({
  value,
}: Readonly<FHIRIdentifierReadOnlyProps>) => {
  return (
    <div className="flex flex-1 space-x-1">
      {value?.system && <div>{value?.system}</div>}
      {value?.value && <div>{value?.value}</div>}
      {value?.type && <div>{value?.type?.text}</div>}
    </div>
  );
};
