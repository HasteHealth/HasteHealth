import React from "react";

import { Period } from "@haste-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRDateTimeEditable } from "../primitives/datetime";
import { EditableProps } from "../types";
import { complexPairGridClass } from "./layout";

export type FHIRPeriodEditableProps = EditableProps<Period>;

export const FHIRPeriodEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRPeriodEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className={complexPairGridClass}>
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
