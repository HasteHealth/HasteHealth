import React from "react";

import { ContactPoint, uri } from "@haste-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRCodeEditable } from "../primitives/code";
import { FHIRStringEditable } from "../primitives/string";
import { ClientProps, EditableProps } from "../types";

export type FHIRContactPointEditableProps = EditableProps<ContactPoint> &
  ClientProps;

export const FHIRContactPointEditable = ({
  fhirVersion,
  value,
  client,
  onChange,
  issue,
  label,
}: FHIRContactPointEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRCodeEditable
          fhirVersion={fhirVersion}
          label="Use"
          client={client}
          open={true}
          system={"http://hl7.org/fhir/ValueSet/contact-point-use" as uri}
          value={value?.use}
          onChange={(use) => {
            onChange?.call(this, { ...value, use });
          }}
        />
        <FHIRCodeEditable
          fhirVersion={fhirVersion}
          label="System"
          client={client}
          open={true}
          system={"http://hl7.org/fhir/ValueSet/contact-point-system" as uri}
          value={value?.system}
          onChange={(system) => {
            onChange?.call(this, { ...value, system });
          }}
        />
        <FHIRStringEditable
          label="Value"
          value={value?.value}
          onChange={(v) => {
            onChange?.call(this, { ...value, value: v });
          }}
        />
      </div>
    </InputContainer>
  );
};
