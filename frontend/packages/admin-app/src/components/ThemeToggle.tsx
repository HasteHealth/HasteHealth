import { MoonIcon, SunIcon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import React from "react";

import { useTheme } from "../theme/ThemeProvider";

export const ThemeToggle = ({
  className,
}: Readonly<{ className?: string }>) => {
  const { theme, toggleTheme } = useTheme();
  const label = `Switch to ${theme === "dark" ? "light" : "dark"} mode`;
  const Icon = theme === "dark" ? SunIcon : MoonIcon;

  return (
    <button
      type="button"
      onClick={toggleTheme}
      className={classNames(
        "rounded-sm p-1 text-ink-muted hover:bg-surface-muted hover:text-ink-body",
        className,
      )}
      aria-label={label}
      title={label}
    >
      <Icon className="h-5 w-5" aria-hidden="true" />
    </button>
  );
};
