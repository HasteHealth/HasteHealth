import React from "react";
import { instant, Signature } from "@haste-health/fhir-types/r4/types";
import { InputContainer } from "../../base/containers";
import { FHIRStringEditable } from "../primitives/string";
import { ClientProps, EditableProps } from "../types";
import { FHIRInstantEditable } from "../primitives";
import { FHIRReferenceEditable } from "./Reference";
import { FHIRCodingEditable } from "./Coding";

export type FHIRSignatureEditableProps = EditableProps<Signature> & ClientProps;

export const FHIRSignatureEditable = ({
  value,
  onChange,
  issue,
  label,
  client,
  fhirVersion,
}: FHIRSignatureEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className="flex flex-col gap-2">
        <FHIRCodingEditable
          client={client}
          fhirVersion={fhirVersion}
          label="Type"
          value={value?.type?.[0]}
          onChange={(v) =>
            onChange?.({ ...value, type: v ? [v] : [] } as Signature)
          }
        />
        <FHIRInstantEditable
          label="When"
          value={value?.when}
          onChange={(v) =>
            onChange?.({ ...value, when: v as instant } as Signature)
          }
        />
        <FHIRReferenceEditable
          client={client}
          fhirVersion={fhirVersion}
          label="Who"
          value={value?.who}
          onChange={(v) => onChange?.({ ...value, who: v } as Signature)}
        />
      </div>
    </InputContainer>
  );
};
