import { useSetAtom } from "jotai";
import React from "react";
import { MagnifyingGlassIcon } from "@heroicons/react/24/outline";

import { openSearchModalAtom } from "./SearchModal";

function Search() {
  const openSearchModal = useSetAtom(openSearchModalAtom);

  return (
    <>
      <button
        onClick={() => openSearchModal(() => true)}
        className="cursor-pointer flex grow max-w-screen-sm py-1.5 px-3 border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] rounded-none space-x-2 items-center hover:border-[var(--ci-signal)] transition-colors focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)] text-left"
        type="button"
      >
        <MagnifyingGlassIcon className="h-4 w-4 text-[var(--ci-ink-subtle)] shrink-0" />
        <span className="flex-1 text-[var(--ci-ink-muted)] text-xs font-mono">
          Search resources, paths...
        </span>
        <kbd className="flex-none px-1.5 py-0.5 rounded-none font-mono text-[10px] border border-[var(--ci-line)] bg-[var(--ci-surface-sunken)] text-[var(--ci-ink-muted)]">
          ⌘K
        </kbd>
      </button>
      <div>
        <React.Suspense fallback={<div />}></React.Suspense>
      </div>
    </>
  );
}

export default Search;
