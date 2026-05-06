import React from "react";

import { DisplayIssues } from "../../base/containers";
import { EditableProps } from "../types";

export type FHIRBooleanEditableProps = EditableProps<boolean>;

export const FHIRBooleanEditable = ({
  value,
  onChange,
  label,
  issue,
}: FHIRBooleanEditableProps) => {
  return (
    <div className="flex flex-col">
      <label className="inline-flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          className="h-4 w-4 rounded border-slate-300 text-brand-600 focus:ring-brand-400"
          checked={value ?? false}
          onChange={(e) => {
            onChange?.call(this, e.target.checked);
          }}
        />
        {label && (
          <span className="text-xs font-medium leading-4 text-slate-700">
            {label}
          </span>
        )}
      </label>
      <DisplayIssues issues={issue ? [issue] : []} />
    </div>
  );
};
