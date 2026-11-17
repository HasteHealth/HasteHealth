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
        "text-nowrap inline-flex items-center rounded-none border px-2 py-0.5 font-mono text-[11px] font-medium uppercase tracking-wide transition-colors",
        {
          "border-sky-500/30 bg-sky-500/10 text-sky-500 dark:text-sky-400 hover:bg-sky-500/20":
            color === "blue",
          "border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[var(--ci-ink)] hover:border-[var(--ci-signal)]":
            color === "gray",
          "border-red-500/30 bg-red-500/10 text-red-600 dark:text-red-400 hover:bg-red-500/20":
            color === "red",
          "border-emerald-500/30 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-500/20":
            color === "green",
          "border-amber-500/30 bg-amber-500/10 text-amber-600 dark:text-amber-400 hover:bg-amber-500/20":
            color === "yellow",
          "border-indigo-500/30 bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 hover:bg-indigo-500/20":
            color === "indigo",
          "border-purple-500/30 bg-purple-500/10 text-purple-600 dark:text-purple-400 hover:bg-purple-500/20":
            color === "purple",
          "border-rose-500/30 bg-rose-500/10 text-rose-600 dark:text-rose-400 hover:bg-rose-500/20":
            color === "pink",
        },
      )}
    >
      {children}
    </span>
  );
}
