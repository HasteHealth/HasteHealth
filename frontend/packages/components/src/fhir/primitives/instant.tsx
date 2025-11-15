import dayjs from "dayjs";
import React, { useEffect, useState } from "react";

import { instant } from "@haste-health/fhir-types/r4/types";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

export type FHIRInstantEditableProps = EditableProps<instant>;

const instantRegex =
  /^([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)-(0[1-9]|1[0-2])-(0[1-9]|[1-2][0-9]|3[0-1])T([01][0-9]|2[0-3]):[0-5][0-9]:([0-5][0-9]|60)(\.[0-9]+)?(Z|(\+|-)((0[0-9]|1[0-3]):[0-5][0-9]|14:00))$/;

export const FHIRInstantEditable = ({
  onChange,
  value,
  issue,
  label,
}: FHIRInstantEditableProps) => {
  const [issues, setIssues] = useState<string[]>([]);
  useEffect(() => {
    const issues: string[] = [];
    if (value && !instantRegex.test(value)) {
      issues.push(`Invalid date format`);
    }
    if (issue) issues.push(issue);
    setIssues(issues);
  }, [value, issue]);

  return (
    <Input
      label={label}
      issues={issues}
      type="datetime-local"
      value={dayjs(value, "YYYY-MM-DDThh:mm:ss.SSSZ").format(
        "YYYY-MM-DDThh:mm:ss.SSS"
      )}
      onChange={(e) => {
        if (onChange) {
          const dateString = dayjs(e.target.value).format(
            "YYYY-MM-DDThh:mm:ss.SSSZ"
          );
          onChange(dateString as instant);
        }
      }}
    />
  );
};
