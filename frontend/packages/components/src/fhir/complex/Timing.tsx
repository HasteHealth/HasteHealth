import React from "react";
import { dateTime, Timing } from "@haste-health/fhir-types/r4/types";

export interface FHIRTimingEditableProps {
  value?: Timing;
  label?: string;
  onChange: (v: Timing | undefined) => void;
}

/**
 * Simple editable Timing component for FHIR Timing datatype.
 * Only supports basic display/edit of event array and code (repeat/when/frequency not implemented).
 */
export const FHIRTimingEditable: React.FC<FHIRTimingEditableProps> = ({
  value,
  label,
  onChange,
}) => {
  const [local, setLocal] = React.useState<Timing | undefined>(value);

  React.useEffect(() => {
    setLocal(value);
  }, [value]);

  const handleEventChange = (idx: number, newVal: string) => {
    const events = local?.event ? [...local.event] : [];
    events[idx] = newVal as dateTime;
    const updated: Timing = { ...local, event: events };
    setLocal(updated);
    onChange(updated);
  };

  const handleAddEvent = () => {
    const events = local?.event
      ? ([...local.event, ""] as dateTime[])
      : ([""] as dateTime[]);
    const updated: Timing = { ...local, event: events };
    setLocal(updated);
    onChange(updated);
  };

  const handleRemoveEvent = (idx: number) => {
    const events = local?.event ? local.event.filter((_, i) => i !== idx) : [];
    const updated: Timing = { ...local, event: events };
    setLocal(updated);
    onChange(updated);
  };

  return (
    <div className="flex flex-col gap-2">
      {label && <label className="font-medium text-sm mb-1">{label}</label>}
      <div className="flex flex-col gap-1">
        {(local?.event ?? []).map((ev, idx) => (
          <div key={idx} className="flex gap-2 items-center">
            <input
              type="datetime-local"
              className="border rounded px-2 py-1 text-sm flex-1"
              value={ev}
              onChange={(e) => handleEventChange(idx, e.target.value)}
            />
            <button
              type="button"
              className="text-xs text-red-500 hover:underline"
              onClick={() => handleRemoveEvent(idx)}
            >
              Remove
            </button>
          </div>
        ))}
        <button
          type="button"
          className="text-xs text-blue-600 hover:underline mt-1 self-start"
          onClick={handleAddEvent}
        >
          Add Event
        </button>
      </div>
    </div>
  );
};
