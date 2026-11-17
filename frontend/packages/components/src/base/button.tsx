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
      return "text-xs px-2.5 py-1";
    case "medium":
      return "text-sm px-3 py-2";
    case "large":
      return "text-base px-4 py-2.5";
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
