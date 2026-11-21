import classNames from "classnames";
import React from "react";

type ButtonType = "primary" | "secondary" | "danger";
type ButtonSize = "small" | "medium" | "large";

export interface ButtonProps
  extends React.DetailedHTMLProps<
    React.ButtonHTMLAttributes<HTMLButtonElement>,
    HTMLButtonElement
  > {
  children: React.ReactNode;
  buttonSize?: ButtonSize;
  buttonType?: ButtonType;
}

function btnTypeClass(type: ButtonType) {
  switch (type) {
    case "primary":
      return "btn-primary";
    case "secondary":
      return "btn-secondary";
    case "danger":
      return "btn-error";
  }
}

function btnSize(size: ButtonSize) {
  switch (size) {
    case "small":
      return "text-xs";
    case "medium":
      return "text-base";
    case "large":
      return "text-lg";
  }
}

export const Button = ({
  buttonType: type,
  buttonSize: size,
  ...props
}: ButtonProps) => {
  const { onClick, children } = props;

  return (
    <button
      onClick={onClick}
      {...props}
      className={classNames(
        "btn",
        btnTypeClass(type ? type : "primary"),
        btnSize(size ? size : "medium"),
        props.className,
      )}
    >
      {children}
    </button>
  );
};
