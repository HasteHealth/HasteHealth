import React from "react";

import { HumanName } from "@oxidized-health/fhir-types/r4/types";

export type FHIRHumanNameReadOnlyProps = {
  value: HumanName;
};

export const FHIRHumanNameReadOnly = ({
  value,
}: Readonly<FHIRHumanNameReadOnlyProps>) => {
  return (
    <div className="flex flex-1 space-x-1">
      {value?.text ? (
        value.text
      ) : (
        <>
          {value?.prefix && <div>{value?.prefix?.join(" ")}</div>}
          {value?.text && <div>{value?.text}</div>}
          {value?.given && <div>{value?.given?.join(" ")}</div>}
          {value?.family && <div>{value?.family}</div>}
          {value?.suffix && <div>{value?.suffix?.join(" ")}</div>}
        </>
      )}
    </div>
  );
};
