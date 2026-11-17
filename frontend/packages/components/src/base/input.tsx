import classNames from "classnames";
import React, {
  DetailedHTMLProps,
  HTMLProps,
  InputHTMLAttributes,
  ReactNode,
} from "react";

import { InputContainer } from "./containers";

export interface InputProps
  extends DetailedHTMLProps<
    InputHTMLAttributes<HTMLInputElement>,
    HTMLInputElement
  > {
  issues?: string[];
  hideBorder?: boolean;
  labelProps?: HTMLProps<HTMLLabelElement>;
  label?: string;
  icon?: ReactNode;
}

export const Input = (props: InputProps) => {
  const {
    label,
    labelProps,
    hideBorder = false,
    issues = [],
    icon,
    required,
    ...inputProps
  } = props;
  return (
    <InputContainer
      required={required}
      hideBorder={hideBorder}
      disabled={inputProps.disabled}
      label={label}
      labelProps={labelProps}
      issues={issues}
      inputContainerClass="bg-[var(--ci-surface-raised)]"
    >
      <>
        <input
          {...inputProps}
          className={classNames(
            "w-full outline-none border-none bg-transparent text-[var(--ci-ink)] font-mono text-xs placeholder-[var(--ci-muted)]",
            inputProps?.className,
          )}
        />
        {icon}
      </>
    </InputContainer>
  );
};
