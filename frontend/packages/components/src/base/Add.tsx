import { PlusIcon } from "@heroicons/react/24/outline";
import React from "react";

export function Add({
  onChange,
  children,
}: Readonly<{ onChange: () => void; children?: React.ReactNode }>) {
  return (
    <button
      type="button"
      className="flex items-center font-mono text-xs text-[var(--ci-signal)] cursor-pointer hover:underline uppercase tracking-wider transition-colors"
      onClick={() => {
        onChange();
      }}
    >
      <PlusIcon className=" h-4 w-4 mr-1" /> {children || "Add"}
    </button>
  );
}
