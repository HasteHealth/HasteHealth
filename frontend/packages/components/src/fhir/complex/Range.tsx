import React from "react";

import { Range } from "@oxidized-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { EditableProps } from "../types";
import { FHIRSimpleQuantityEditable } from "./SimpleQuantity";

export type FHIRRangeEditableProps = EditableProps<Range>;

export const FHIRRangeEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRRangeEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRSimpleQuantityEditable
          label="Low"
          value={value?.low}
          onChange={(low) => {
            onChange?.call(this, { ...value, low });
          }}
        />
        <FHIRSimpleQuantityEditable
          label="High"
          value={value?.high}
          onChange={(high) => {
            onChange?.call(this, { ...value, high });
          }}
        />
      </div>
    </InputContainer>
  );
};
