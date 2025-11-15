import React from "react";

import { decimal } from "@haste-health/fhir-types/r4/types";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

export type FHIRDecimalEditableProps = EditableProps<decimal>;

export const FHIRDecimalEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRDecimalEditableProps) => {
  return (
    <Input
      label={label}
      issues={issue ? [issue] : []}
      type="number"
      value={value}
      step="0.1"
      onChange={(e) => {
        const value = parseFloat(e.target.value);
        if (onChange && !isNaN(value)) {
          onChange(value as decimal);
        }
      }}
    />
  );
};
