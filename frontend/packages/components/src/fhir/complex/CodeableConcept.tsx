import { XMarkIcon } from "@heroicons/react/24/outline";
import React from "react";

import { CodeableConcept, Coding } from "@oxidized-health/fhir-types/r4/types";

import { FHIRCodingEditable } from ".";
import { Add } from "../../base";
import { InputContainer } from "../../base/containers";
import { ClientProps, EditableProps } from "../types";

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
    <InputContainer
      hideBorder={true}
      label={label}
      issues={issue ? [issue] : []}
    >
      <div className="flex flex-col space-y-1">
        {value?.coding?.map((coding, index) => (
          <div key={`${coding.code}-${coding.system}`} className="relative">
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
            <div
              className="absolute top-1 right-1 text-slate-400 cursor-pointer hover:text-slate-500 "
              onClick={() => {
                const newCoding: Coding[] = [...(value?.coding || [])];
                newCoding.splice(index, 1);
                onChange?.call(this, { ...value, coding: newCoding });
              }}
            >
              <XMarkIcon className="h-4 w-4" />
            </div>
          </div>
        ))}
        <div>
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
