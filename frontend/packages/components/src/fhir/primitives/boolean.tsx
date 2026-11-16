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
          className="h-4 w-4 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[var(--ci-signal)] focus:ring-[var(--ci-signal)]"
          checked={value ?? false}
          onChange={(e) => {
            onChange?.call(this, e.target.checked);
          }}
        />
        {label && (
          <span className="font-mono text-xs text-[var(--ci-ink)]">
            {label}
          </span>
        )}
      </label>
      <DisplayIssues issues={issue ? [issue] : []} />
    </div>
  );
};
