import React, { useEffect, useState } from "react";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

type OID = `urn:oid:${string}`;

export type FHIROIDEditableProps = EditableProps<OID>;

const oidRegex = /^urn:oid:[0-2](\.(0|[1-9][0-9]*))+$/;

function isOID(value: string): value is OID {
  return oidRegex.test(value);
}

export const FHIROIDEditable = ({
  onChange,
  value,
  issue,
  label,
}: FHIROIDEditableProps) => {
  const [issues, setIssues] = useState<string[]>([]);

  useEffect(() => {
    const issues: string[] = [];
    if (value && !isOID(value)) {
      issues.push(`Invalid oid format`);
    }
    if (issue) issues.push(issue);
    setIssues(issues);
  }, [value, issue]);

  return (
    <Input
      label={label}
      issues={issues}
      type="text"
      value={value}
      onChange={(e) => {
        const value = e.target.value;
        onChange?.call(this, value as OID);
      }}
    />
  );
};
