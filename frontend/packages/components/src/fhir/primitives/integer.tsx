import React from "react";

import { integer } from "@haste-health/fhir-types/r4/types";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

export type FHIRIntegerEditableProps = EditableProps<integer>;

export const FHIRIntegerEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRIntegerEditableProps) => {
  return (
    <Input
      label={label}
      issues={issue ? [issue] : []}
      type="number"
      value={value ? Math.round(value) : value}
      step="0.1"
      onChange={(e) => {
        const value = parseInt(e.target.value);
        if (onChange && !isNaN(value)) {
          onChange(value as integer);
        }
      }}
    />
  );
};
