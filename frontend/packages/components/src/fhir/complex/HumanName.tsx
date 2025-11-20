import React from "react";

import { HumanName } from "@haste-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRStringEditable } from "../primitives/string";
import { EditableProps } from "../types";

export type FHIRHumanNameEditableProps = EditableProps<HumanName>;

export const FHIRHumanNameEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRHumanNameEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRStringEditable
          label="First"
          value={value?.given?.[0] || ""}
          onChange={(firstName) => {
            const given = value?.given ? [...value.given] : [];

            if (firstName !== undefined) {
              given[0] = firstName;
            }
            onChange?.call(this, { ...value, given });
          }}
        />
        <FHIRStringEditable
          label="Middle"
          value={value?.given?.[1] || ""}
          onChange={(middleName) => {
            const given = value?.given ? [...value.given] : [];
            if (!given[0]) given[0] = "";
            if (middleName !== undefined) {
              given[1] = middleName;
            }
            onChange?.call(this, { ...value, given });
          }}
        />
        <FHIRStringEditable
          label="Last"
          value={value?.family}
          onChange={(family) => {
            onChange?.call(this, { ...value, family });
          }}
        />
      </div>
    </InputContainer>
  );
};
