import classNames from "classnames";
import React, { DetailedHTMLProps, InputHTMLAttributes } from "react";

interface TagProps
  extends DetailedHTMLProps<
    InputHTMLAttributes<HTMLSpanElement>,
    HTMLSpanElement
  > {
  className?: string;
  children: React.ReactNode;
  color?:
    | "blue"
    | "gray"
    | "red"
    | "green"
    | "yellow"
    | "indigo"
    | "purple"
    | "pink";
}

export function Tag({
  className,
  children,
  color = "blue",
  ...spanProps
}: Readonly<TagProps>) {
  return (
    <span
      {...spanProps}
      className={classNames(
        className,
        "text-nowrap inline-block text-xs font-medium me-2 px-2.5 py-0.5",
        {
          "hover:bg-teal-200 hover:text-teal-900 bg-teal-200 text-teal-800 rounded":
            color === "blue",
          "hover:bg-gray-200 hover:text-gray-900 bg-gray-100 text-gray-800 rounded":
            color === "gray",
          "hover:bg-red-200 hover:text-red-900 bg-red-100 text-red-800 rounded":
            color === "red",
          "hover:bg-green-200 hover:text-green-900 bg-green-100 text-green-800 rounded":
            color === "green",
          "hover:bg-yellow-200 hover:text-yellow-900 bg-yellow-100 text-yellow-800 rounded":
            color === "yellow",
          "hover:bg-indigo-200 hover:text-indigo-900 bg-indigo-100 text-indigo-800 rounded":
            color === "indigo",
          "hover:bg-purple-200 hover:text-purple-900 bg-purple-100 text-purple-800 rounded":
            color === "purple",
          "hover:bg-pink-200 hover:text-pink-900 bg-pink-100 text-pink-800 rounded":
            color === "pink",
        }
      )}
    >
      {children}
    </span>
  );
}
