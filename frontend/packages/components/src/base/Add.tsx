import { PlusIcon } from "@heroicons/react/24/outline";
import React from "react";

export function Add({
  onChange,
  children,
}: Readonly<{ onChange: () => void; children?: React.ReactNode }>) {
  return (
    <span
      className="flex items-center text-xs text-brand-500 cursor-pointer hover:text-brand-600"
      onClick={() => {
        onChange();
      }}
    >
      <PlusIcon className=" h-4 w-4 mr-1" /> {children || "Add"}
    </span>
  );
}
