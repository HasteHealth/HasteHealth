import React from "react";
import { Money } from "@haste-health/fhir-types/r4/types";
import { InputContainer } from "../../base/containers";
import { FHIRDecimalEditable } from "../primitives/decimal";
import { FHIRCodeEditable } from "../primitives/code";
import { ClientProps, EditableProps } from "../types";
import { complexFieldGridClass } from "./layout";

export type FHIRMoneyEditableProps = EditableProps<Money> & ClientProps;

export const FHIRMoneyEditable = ({
  value,
  onChange,
  issue,
  label,
  client,
  fhirVersion,
}: FHIRMoneyEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className={complexFieldGridClass}>
        <FHIRDecimalEditable
          label="Value"
          value={value?.value}
          onChange={(v) => onChange?.({ ...value, value: v })}
        />
        <FHIRCodeEditable
          client={client}
          label="Currency"
          fhirVersion={fhirVersion}
          value={value?.currency}
          onChange={(v) => onChange?.({ ...value, currency: v })}
        />
      </div>
    </InputContainer>
  );
};
