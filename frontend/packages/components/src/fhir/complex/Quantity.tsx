import React from "react";

import { Quantity, uri } from "@oxidized-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import {
  FHIRCodeEditable,
  FHIRDecimalEditable,
  FHIRStringEditable,
} from "../primitives";
import { ClientProps, EditableProps } from "../types";

export type FHIRQuantityEditableProps = EditableProps<Quantity> & ClientProps;

export const FHIRQuantityEditable = ({
  fhirVersion,
  value,
  onChange,
  issue,
  label,
  client,
}: FHIRQuantityEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRCodeEditable
          fhirVersion={fhirVersion}
          client={client}
          label="Comparator"
          open={true}
          system={"http://hl7.org/fhir/ValueSet/quantity-comparator" as uri}
          value={value?.comparator}
          onChange={(comparator) => {
            onChange?.call(this, { ...value, comparator });
          }}
        />
        <FHIRDecimalEditable
          label="Value"
          value={value?.value}
          onChange={(valueDec) => {
            onChange?.call(this, { ...value, value: valueDec });
          }}
        />
        <FHIRStringEditable
          label="Unit"
          value={value?.unit}
          onChange={(unit) => {
            onChange?.call(this, { ...value, unit });
          }}
        />
      </div>
    </InputContainer>
  );
};
