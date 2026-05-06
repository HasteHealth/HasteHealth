import { XMarkIcon } from "@heroicons/react/24/outline";
import React from "react";

import { CodeableConcept, Coding } from "@haste-health/fhir-types/r4/types";

import { FHIRCodingEditable } from ".";
import { Add } from "../../base";
import { InputContainer } from "../../base/containers";
import { ClientProps, EditableProps } from "../types";
import { complexStackClass } from "./layout";

export type FHIRCodeableConceptEditableProps = EditableProps<CodeableConcept> &
  ClientProps;

export const FhirCodeableConceptEditable = ({
  fhirVersion,
  client,
  value,
  onChange,
  issue,
  label,
}: FHIRCodeableConceptEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className={complexStackClass}>
        {value?.coding?.map((coding, index) => (
          <div
            key={`${coding.code}-${coding.system}`}
            className="relative rounded-md border border-slate-200 bg-white p-2 pr-8"
          >
            <FHIRCodingEditable
              fhirVersion={fhirVersion}
              client={client}
              value={coding}
              onChange={(coding) => {
                if (coding) {
                  const newCoding: Coding[] = [...(value?.coding || [])];
                  newCoding.splice(index, 1, coding);
                  onChange?.call(this, { ...value, coding: newCoding });
                }
              }}
            />
            <button
              type="button"
              className="absolute right-1.5 top-1.5 rounded-sm p-0.5 text-slate-400 transition-colors hover:bg-slate-100 hover:text-slate-600"
              onClick={() => {
                const newCoding: Coding[] = [...(value?.coding || [])];
                newCoding.splice(index, 1);
                onChange?.call(this, { ...value, coding: newCoding });
              }}
            >
              <XMarkIcon className="h-4 w-4" />
            </button>
          </div>
        ))}
        <div className="pt-0.5">
          <Add
            onChange={() => {
              const newCoding: Coding[] = [...(value?.coding || [])];
              newCoding.push({});
              onChange?.call(this, { ...value, coding: newCoding });
            }}
          />
        </div>
      </div>
    </InputContainer>
  );
};
