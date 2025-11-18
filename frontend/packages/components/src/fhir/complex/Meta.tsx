import React from "react";

import { Meta } from "@haste-health/fhir-types/r4/types";

import { InputContainer } from "../../base";
import { FHIRStringEditable } from "../primitives/string";
import { ClientProps } from "../types";
import { FHIRReferenceEditable } from "./Reference";

export interface FHIRMetaReadonlyProps extends ClientProps {
  value?: Meta;
}

export const FHIRMetaReadOnly = ({
  value,
  client,
  fhirVersion,
}: FHIRMetaReadonlyProps) => {
  return (
    <InputContainer>
      <div className="flex flex-1 space-x-1">
        <FHIRStringEditable
          value={value?.versionId}
          disabled={true}
          label="Version ID"
        />
        <FHIRStringEditable
          value={value?.lastUpdated}
          disabled={true}
          label="Last Updated"
        />
        <FHIRReferenceEditable
          fhirVersion={fhirVersion}
          client={client}
          value={
            value?.extension?.find(
              (e) => e.url === "https://haste.health/author"
            )?.valueReference
          }
          disabled={true}
          label="Author"
        />
      </div>
    </InputContainer>
  );
};
