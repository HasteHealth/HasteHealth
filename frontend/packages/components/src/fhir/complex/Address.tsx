import React from "react";

import { Address } from "@oxidized-health/fhir-types/r4/types";

import { InputContainer } from "../../base/containers";
import { FHIRStringEditable } from "../primitives/string";
import { EditableProps } from "../types";

export type FHIRAddressEditableProps = EditableProps<Address>;

export const FHIRAddressEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRAddressEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-1 space-x-1">
        <FHIRStringEditable
          label="Line"
          value={value?.line?.[0] || ""}
          onChange={(line) => {
            if (line !== undefined) {
              onChange?.call(this, { ...value, line: [line] });
            }
          }}
        />
        <FHIRStringEditable
          label="City"
          value={value?.city || ""}
          onChange={(city) => {
            onChange?.call(this, { ...value, city });
          }}
        />
        {/* <String label="District" value={value?.district || ""} /> */}
        <FHIRStringEditable
          label="State"
          value={value?.state || ""}
          onChange={(state) => {
            onChange?.call(this, { ...value, state });
          }}
        />
        <FHIRStringEditable
          label="Postal Code"
          value={value?.postalCode || ""}
          onChange={(postalCode) => {
            onChange?.call(this, { ...value, postalCode });
          }}
        />
        <FHIRStringEditable
          label="Country"
          value={value?.country || ""}
          onChange={(country) => {
            onChange?.call(this, { ...value, country });
          }}
        />
      </div>
    </InputContainer>
  );
};
