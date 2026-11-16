import classNames from "classnames";
import React, { HTMLProps } from "react";

export const inputClassNames = ({
  hideBorder,
  issues,
}: {
  hideBorder: boolean;
  issues: string[];
}) =>
  classNames(
    "min-h-8",
    "flex relative px-2.5 py-1.5 font-mono text-xs rounded-none transition-colors",
    "placeholder-[var(--ci-muted)] disabled:opacity-50",
    "focus:outline-none focus:border-[var(--ci-signal)] focus:ring-1 focus:ring-[var(--ci-signal)] block w-full",
    "invalid:border-red-500 invalid:text-red-600 focus:invalid:border-red-500 focus:invalid:ring-red-500",
    {
      border: !hideBorder,
      "border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[var(--ci-ink)]":
        issues.length === 0,
      "text-red-500 border-red-500 bg-[var(--ci-surface-raised)]":
        issues.length !== 0,
    },
  );

export const DisplayIssues = ({ issues }: { issues: string[] }) => {
  return (
    issues.length !== 0 && (
      <div className="text-sm mt-1 text-red-600">{issues.join(".\n")}</div>
    )
  );
};

interface LabelProps extends HTMLProps<HTMLLabelElement> {
  label?: string;
  required?: boolean;
}

export const Label = (props: LabelProps) =>
  props.label && (
    <label
      {...props}
      className={classNames(props.className, {
        required: props.required,
      })}
    >
      {props.label}
    </label>
  );

export interface ContainerProps {
  children: React.ReactNode;
  issues?: string[];
  label?: string;
  disabled?: boolean;
  inlineLabel?: boolean;
  hideBorder?: boolean;
  labelProps?: HTMLProps<HTMLLabelElement>;
  required?: boolean;
  inputContainerClass?: string;
}

export const InputContainer = ({
  children,
  issues = [],
  label,
  required,
  inlineLabel = false,
  hideBorder = false,
  disabled = false,
  labelProps,
  inputContainerClass,
}: ContainerProps) => (
  <div
    className={classNames("group grow", {
      "flex flex-row items-center justify-center": inlineLabel,
    })}
    aria-disabled={disabled}
  >
    <Label
      {...labelProps}
      label={label}
      required={required}
      className={classNames(
        {
          "mb-1 block text-xs font-mono uppercase tracking-wider text-[var(--ci-muted)]":
            !inlineLabel,
          "mr-2 text-xs font-mono uppercase tracking-wider text-[var(--ci-muted)]": inlineLabel,
        },
        labelProps?.className,
      )}
    />

    <div
      className={classNames(
        inputClassNames({ hideBorder, issues }),
        inputContainerClass,
      )}
    >
      {children}
    </div>
    <DisplayIssues issues={issues} />
  </div>
);
