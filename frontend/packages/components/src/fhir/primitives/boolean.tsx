import React from "react";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

export type FHIRBooleanEditableProps = EditableProps<boolean>;

export const FHIRBooleanEditable = ({
  value,
  onChange,
  label,
  issue,
}: FHIRBooleanEditableProps) => {
  return (
    <div className="inline-block">
      <Input
        hideBorder
        label={label}
        issues={issue ? [issue] : []}
        type="checkbox"
        checked={value}
        onChange={(e) => {
          onChange?.call(this, e.target.checked);
        }}
      />
    </div>
  );
};
