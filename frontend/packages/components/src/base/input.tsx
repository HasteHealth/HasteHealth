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
      inputContainerClass="bg-white"
    >
      <>
        <input
          {...inputProps}
          className={classNames(
            "w-full outline-none border-none group-aria-disabled:bg-gray-50",
            inputProps?.className,
          )}
        />
        {icon}
      </>
    </InputContainer>
  );
};
