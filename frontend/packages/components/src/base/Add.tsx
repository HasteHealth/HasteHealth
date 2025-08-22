import { PlusIcon } from "@heroicons/react/24/outline";
import React from "react";

export function Add({
  onChange,
  children,
}: Readonly<{ onChange: () => void; children?: React.ReactNode }>) {
  return (
    <span
      className="flex items-center text-xs  text-blue-400 cursor-pointer hover:text-blue-500"
      onClick={() => {
        onChange();
      }}
    >
      <PlusIcon className=" h-4 w-4 mr-1" /> {children || "Add"}
    </span>
  );
}
