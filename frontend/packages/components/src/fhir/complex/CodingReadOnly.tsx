import React from "react";

import { Coding } from "@haste-health/fhir-types/r4/types";

export type FHIRCodingReadOnlyProps = {
  value: Coding;
};

function FHIRCodingDetailDisplay({ value }: Readonly<FHIRCodingReadOnlyProps>) {
  return (
    <div>
      <span>
        [{value.system ? `${value.system}:` : ""} {value.code}]
      </span>
    </div>
  );
}

export const FHIRCodingReadOnly = ({
  value,
}: Readonly<FHIRCodingReadOnlyProps>) => {
  return (
    <>
      {value.display ? (
        value.display
      ) : (
        <FHIRCodingDetailDisplay value={value} />
      )}
    </>
  );
};
