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
    "flex relative px-2 py-1 group-aria-disabled:bg-gray-50",
    "placeholder-slate-400 disabled:bg-slate-50 disabled:text-slate-500 disabled:border-slate-200",
    "focus:outline-none focus:border-sky-500 focus:ring-sky-500 block w-full rounded-md sm:text-sm ",
    "invalid:border-red-500 invalid:text-red-600 focus:invalid:border-red-500 focus:invalid:ring-red-500 disabled:shadow-none",
    {
      border: !hideBorder,
      "focus:ring-1": !hideBorder,
      "border-slate-300": issues.length === 0 ? true : false,
      "text-red-500": issues.length !== 0 ? true : false,
      "border-red-500": issues.length !== 0 ? true : false,
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
    className={classNames("group flex-grow", {
      "flex flex-row items-center justify-center": inlineLabel,
    })}
    aria-disabled={disabled}
  >
    <Label
      {...labelProps}
      label={label}
      required={required}
      className={classNames("mr-1", labelProps?.className)}
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
