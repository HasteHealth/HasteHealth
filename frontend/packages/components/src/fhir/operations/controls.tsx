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
      className="inline-flex overflow-hidden rounded-none border border-[var(--ci-line)] text-xs font-mono"
    >
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          disabled={option.disabled}
          title={option.title}
          aria-pressed={value === option.value}
          className={classNames(
            "border-l border-[var(--ci-line)] px-2.5 py-1 text-xs font-mono uppercase tracking-wider first:border-l-0 transition-colors",
            {
              "bg-[var(--ci-signal)] text-white font-semibold": value === option.value,
              "bg-[var(--ci-surface)] text-[var(--ci-muted)] hover:bg-[var(--ci-surface-raised)] hover:text-[var(--ci-ink)]":
                value !== option.value && !option.disabled,
              "cursor-not-allowed bg-[var(--ci-surface-sunken)] text-[var(--ci-ink-muted)] opacity-50":
                option.disabled,
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
      <span className="font-mono text-xs font-semibold text-[var(--ci-ink)]">
        {definition.name}
      </span>
      {isRequired(definition) && (
        <span className="text-xs text-red-500" title="Required">
          *
        </span>
      )}
      <span className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1.5 py-0.5 font-mono text-[10px] text-[var(--ci-muted)]">
        {typeLabel(definition)}
      </span>
      <span className="font-mono text-[10px] text-[var(--ci-muted)]">
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
    <p className="whitespace-pre-wrap text-xs text-[var(--ci-muted)]">
      {definition.documentation}
    </p>
  );
}
