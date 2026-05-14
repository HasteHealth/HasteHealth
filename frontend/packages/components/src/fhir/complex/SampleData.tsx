import React from "react";
import { SampledData } from "@haste-health/fhir-types/r4/types";
import { InputContainer } from "../../base/containers";
import { FHIRStringEditable } from "../primitives/string";
import { EditableProps } from "../types";
import { FHIRSimpleQuantityEditable } from "./SimpleQuantity";

export type FHIRSampledDataEditableProps = EditableProps<SampledData>;

export const FHIRSampledDataEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRSampledDataEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-col gap-2">
        <FHIRSimpleQuantityEditable
          label="Origin"
          value={value?.origin}
          onChange={(v) => onChange?.({ ...value, origin: v } as SampledData)}
        />
        <FHIRStringEditable
          label="Data"
          value={value?.data}
          onChange={(v) => onChange?.({ ...value, data: v } as SampledData)}
        />
      </div>
    </InputContainer>
  );
};
