import React from "react";
import { dateTime, Timing } from "@haste-health/fhir-types/r4/types";
import { InputContainer } from "../../base/containers";
import { FHIRDateTimeEditable } from "../primitives/datetime";
import { EditableProps } from "../types";
import { complexFieldGridClass } from "./layout";

export type FHIRTimingEditableProps = EditableProps<Timing>;

export const FHIRTimingEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRTimingEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className={complexFieldGridClass}>
        {(value?.event ?? []).map((ev, idx) => (
          <div key={idx} className="flex gap-2 items-center">
            <FHIRDateTimeEditable
              label={`Event ${idx + 1}`}
              value={ev}
              onChange={(newVal) => {
                const events = value?.event ? [...value.event] : [];
                events[idx] = newVal as dateTime;
                onChange?.({ ...value, event: events });
              }}
            />
            <button
              type="button"
              className="text-xs text-red-500 hover:underline"
              onClick={() => {
                const events = value?.event
                  ? value.event.filter((_, i) => i !== idx)
                  : [];
                onChange?.({ ...value, event: events });
              }}
            >
              Remove
            </button>
          </div>
        ))}
        <button
          type="button"
          className="text-xs text-blue-600 hover:underline mt-1 self-start"
          onClick={() => {
            const events = value?.event ? [...value.event, ""] : [""];
            onChange?.({ ...value, event: events as dateTime[] });
          }}
        >
          Add Event
        </button>
      </div>
    </InputContainer>
  );
};
