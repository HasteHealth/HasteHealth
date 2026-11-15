import React from "react";
import { SunIcon, MoonIcon } from "@heroicons/react/24/outline";
import { useTheme } from "../theme/ThemeProvider";

export const ThemeToggle = ({ className = "" }: { className?: string }) => {
  const { theme, toggleTheme } = useTheme();

  return (
    <button
      type="button"
      onClick={toggleTheme}
      className={`p-2 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[var(--ci-ink)] hover:bg-[var(--ci-surface-sunken)] hover:border-[var(--ci-signal)] transition-colors focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)] ${className}`}
      aria-label={`Switch to ${theme === "dark" ? "light" : "dark"} mode`}
      title={`Switch to ${theme === "dark" ? "light" : "dark"} mode`}
    >
      {theme === "dark" ? (
        <SunIcon className="h-5 w-5 text-amber-400" aria-hidden="true" />
      ) : (
        <MoonIcon className="h-5 w-5 text-[var(--ci-ink)]" aria-hidden="true" />
      )}
    </button>
  );
};
