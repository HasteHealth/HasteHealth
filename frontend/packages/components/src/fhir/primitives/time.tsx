import React, { useEffect, useState } from "react";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

export type FHIRTimeEditableProps = EditableProps<string>;

const timeRegex = /^([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?$/;

export const FHIRTimeEditable = ({
  onChange,
  value,
  issue,
  label,
}: FHIRTimeEditableProps) => {
  const [issues, setIssues] = useState<string[]>([]);
  useEffect(() => {
    const issues: string[] = [];
    if (value && !timeRegex.test(value)) {
      issues.push(`Invalid date format`);
    }
    if (issue) issues.push(issue);

    setIssues(issues);
  }, [value, issue]);

  return (
    <Input
      label={label}
      issues={issues}
      step="1"
      type="time"
      value={value}
      onChange={(e) => {
        onChange?.call(this, e.target.value);
      }}
    />
  );
};
