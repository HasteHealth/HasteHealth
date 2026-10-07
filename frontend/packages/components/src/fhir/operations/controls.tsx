import classNames from "classnames";
import React from "react";

import { OperationDefinitionParameter } from "@haste-health/fhir-types/r4/types";

import { cardinality, isRequired, typeLabel } from "./parameters";

export interface SegmentOption<T extends string> {
  value: T;
  label: string;
  disabled?: boolean;
  /** Why the option is disabled, shown on hover. */
  title?: string;
}

/** A small row of buttons where one is selected. */
export function Segmented<T extends string>({
  value,
  options,
  onChange,
  ariaLabel,
}: Readonly<{
  value: T;
  options: SegmentOption<T>[];
  onChange: (value: T) => void;
  ariaLabel: string;
}>) {
  return (
    <fieldset
      aria-label={ariaLabel}
      className="inline-flex overflow-hidden rounded-md border border-slate-300 text-xs"
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          disabled={option.disabled}
          title={option.title}
          aria-pressed={value === option.value}
          className={classNames(
            "border-l border-slate-300 px-2.5 py-1 font-medium first:border-l-0",
            {
              "bg-brand-600 text-white": value === option.value,
              "bg-white text-slate-700 hover:bg-slate-50":
                value !== option.value && !option.disabled,
              "cursor-not-allowed bg-slate-50 text-slate-300": option.disabled,
            },
          )}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </fieldset>
  );
}

/** A parameter's name, type and cardinality, with an action on the right. */
export function ParameterHeader({
  definition,
  action,
}: Readonly<{
  definition: OperationDefinitionParameter;
  action?: React.ReactNode;
}>) {
  return (
    <div className="flex min-h-6 flex-wrap items-center gap-1.5">
      <span className="font-mono text-sm font-medium text-slate-900">
        {definition.name}
      </span>
      {isRequired(definition) && (
        <span className="text-xs text-red-600" title="Required">
          *
        </span>
      )}
      <span className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[11px] text-slate-600">
        {typeLabel(definition)}
      </span>
      <span className="font-mono text-[11px] text-slate-500">
        {cardinality(definition)}
      </span>
      <div className="flex-1" />
      {action}
    </div>
  );
}

/** A parameter's documentation, if it has any. */
export function ParameterDocumentation({
  definition,
}: Readonly<{ definition: OperationDefinitionParameter }>) {
  if (!definition.documentation) return null;
  return (
    <p className="whitespace-pre-wrap text-xs text-slate-500">
      {definition.documentation}
    </p>
  );
}
