import dayjs from "dayjs";
import React, { useEffect, useState } from "react";

import { date } from "@oxidized-health/fhir-types/r4/types";

import { Input } from "../../base/input";
import { EditableProps } from "../types";

export type FHIRDateEditableProps = EditableProps<date> & {
  /**
   * String output format defaults to YYYY-MM-DD.
   */
  outputFormat?: string;
};

const dateRegex =
  /^([0-9]([0-9]([0-9][1-9]|[1-9]0)|[1-9]00)|[1-9]000)(-(0[1-9]|1[0-2])(-(0[1-9]|[1-2][0-9]|3[0-1]))?)?$/;

export const FHIRDateEditable = ({
  onChange,
  value,
  issue,
  label,
  outputFormat = "YYYY-MM-DD",
}: FHIRDateEditableProps) => {
  const [issues, setIssues] = useState<string[]>([]);
  useEffect(() => {
    const issues: string[] = [];
    if (value && !dateRegex.test(value)) {
      issues.push(`Invalid date format`);
    }
    if (issue) issues.push(issue);

    setIssues(issues);
  }, [value, issue]);

  return (
    <Input
      label={label}
      issues={issues}
      type="date"
      value={value && dayjs(value).format("YYYY-MM-DD")}
      onChange={(e) => {
        if (onChange) {
          const dateString = dayjs(e.target.value).format(outputFormat) as date;
          onChange(dateString);
        }
      }}
    />
  );
};
