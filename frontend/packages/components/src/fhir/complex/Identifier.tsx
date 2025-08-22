import React from "react";

import { Identifier, uri } from "@oxidized-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRCodeEditable } from "../primitives/code";
import { FHIRStringEditable } from "../primitives/string";
import { FHIRUriEditable } from "../primitives/uri";
import { ClientProps, EditableProps } from "../types";

export type FHIRIdentifierEditableProps = EditableProps<Identifier> &
  ClientProps;

export const FHIRIdentifierEditable = ({
  fhirVersion,
  value,
  client,
  onChange,
  issue,
  label,
}: FHIRIdentifierEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRCodeEditable
          fhirVersion={fhirVersion}
          client={client}
          label="Use"
          open={true}
          system={"http://hl7.org/fhir/ValueSet/identifier-use" as uri}
          value={value?.use}
          onChange={(use) => {
            onChange?.call(this, { ...value, use });
          }}
        />
        <FHIRUriEditable
          label="System"
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
