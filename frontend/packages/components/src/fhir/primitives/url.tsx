import React from "react";

import { url } from "@haste-health/fhir-types/r4/types";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

export type FHIRStringEditableProps = EditableProps<url> & {
  disabled?: boolean;
  inputProps?: Parameters<typeof Input>[0];
};

export const FHIRUrlEditable = ({
  onChange,
  value,
  issue,
  label,
  disabled = false,
  inputProps,
}: FHIRStringEditableProps) => {
  return (
    <Input
      {...inputProps}
      disabled={disabled}
      label={label}
      issues={issue ? [issue] : []}
      type="text"
      value={value}
      onChange={(e) => {
        onChange?.call(this, e.target.value as url);
      }}
    />
  );
};
