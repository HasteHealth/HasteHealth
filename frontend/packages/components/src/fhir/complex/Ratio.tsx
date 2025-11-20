import React from "react";

import { Ratio } from "@haste-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { EditableProps } from "../types";
import { FHIRSimpleQuantityEditable } from "./SimpleQuantity";

export type FHIRRatioEditableProps = EditableProps<Ratio>;

export const FHIRRatioEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRRatioEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRSimpleQuantityEditable
          label="Numerator"
          value={value?.numerator}
          onChange={(numerator) => {
            onChange?.call(this, { ...value, numerator });
          }}
        />
        <FHIRSimpleQuantityEditable
          label="Denominator"
          value={value?.denominator}
          onChange={(denominator) => {
            onChange?.call(this, { ...value, denominator });
          }}
        />
      </div>
    </InputContainer>
  );
};
