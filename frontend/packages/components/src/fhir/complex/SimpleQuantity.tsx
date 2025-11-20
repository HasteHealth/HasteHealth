import React from "react";

import { Quantity } from "@haste-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRDecimalEditable, FHIRStringEditable } from "../primitives";
import { EditableProps } from "../types";

export type FHIRSimpleQuantityEditableProps = EditableProps<Quantity>;

export const FHIRSimpleQuantityEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRSimpleQuantityEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
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
