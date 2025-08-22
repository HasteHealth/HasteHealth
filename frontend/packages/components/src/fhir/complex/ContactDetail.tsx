import React from "react";

import { ContactDetail, uri } from "@oxidized-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRCodeEditable, FHIRStringEditable } from "../primitives";
import { ClientProps, EditableProps } from "../types";

export type FHIRContactDetailEditableProps = EditableProps<ContactDetail> &
  ClientProps;

export const FHIRContactDetailEditable = ({
  fhirVersion,
  value,
  onChange,
  client,
  issue,
  label,
}: FHIRContactDetailEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRStringEditable
          label="Name"
          value={value?.name}
          onChange={(name) => {
            if (name !== undefined) {
              onChange?.call(this, { ...value, name });
            }
          }}
        />
        <FHIRCodeEditable
          fhirVersion={fhirVersion}
          label="System"
          client={client}
          open={true}
          system={"http://hl7.org/fhir/ValueSet/contact-point-system" as uri}
          value={value?.telecom?.[0]?.system}
          onChange={(telecomSystem) => {
            onChange?.call(this, {
              ...value,
              telecom: [{ ...value?.telecom?.[0], system: telecomSystem }],
            });
          }}
        />
        <FHIRStringEditable
          label="Value"
          value={value?.telecom?.[0]?.value}
          onChange={(telecomValue) => {
            if (telecomValue !== undefined) {
              onChange?.call(this, {
                ...value,
                telecom: [{ ...value?.telecom?.[0], value: telecomValue }],
              });
            }
          }}
        />
      </div>
    </InputContainer>
  );
};
