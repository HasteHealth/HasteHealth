import React from "react";

import { Period } from "@haste-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRDateTimeEditable } from "../primitives/datetime";
import { EditableProps } from "../types";

export type FHIRPeriodEditableProps = EditableProps<Period>;

export const FHIRPeriodEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRPeriodEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRDateTimeEditable
          value={value?.start}
          label="Start"
          onChange={(start) => {
            onChange?.call(this, { ...value, start });
          }}
        />
        <FHIRDateTimeEditable
          value={value?.end}
          label="End"
          onChange={(end) => {
            onChange?.call(this, { ...value, end });
          }}
        />
      </div>
    </InputContainer>
  );
};
