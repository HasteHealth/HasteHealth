import React, { useEffect, useRef } from "react";

import { Shortcut, modLabel } from "../hooks/useShortcuts";

export interface ShortcutHelpProps {
  shortcuts: Shortcut[];
  onClose: () => void;
}

/** Renders one binding the way the platform writes it. */
function chord(shortcut: Shortcut): string {
  return [
    shortcut.mod ? modLabel() : undefined,
    shortcut.shift ? "Shift" : undefined,
    shortcut.key === " " ? "Space" : shortcut.key.toUpperCase(),
  ]
    .filter(Boolean)
    .join(" + ");
}

/**
 * The list of shortcuts, shown over the console.
 */
export function ShortcutHelp({
  shortcuts,
  onClose,
}: Readonly<ShortcutHelpProps>) {
  const dialog = useRef<HTMLDialogElement | null>(null);

  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    element?.setAttribute("closedby", "any");
  }, []);

  return (
    <dialog
      ref={dialog}
      aria-label="Keyboard shortcuts"
      className="max-h-[80vh] w-full max-w-md rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] text-[var(--ci-ink)] p-0 shadow-2xl backdrop:bg-black/60"
      onClose={onClose}
    >
      <div className="max-h-[80vh] overflow-auto">
        <header className="flex items-center border-b border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-4 py-2.5">
          <h2 className="flex-1 text-sm font-semibold text-[var(--ci-ink)]">
            Keyboard shortcuts
          </h2>
          <button
            className="rounded-none border border-[var(--ci-line)] px-2 py-0.5 font-mono text-xs text-[var(--ci-ink-muted)] hover:bg-[var(--ci-surface-sunken)] hover:text-[var(--ci-ink)]"
            onClick={onClose}
            type="button"
          >
            Esc
          </button>
        </header>
        <ul className="divide-y divide-[var(--ci-line)]">
          {shortcuts.map((shortcut) => (
            <li
              key={`${shortcut.key}-${shortcut.mod}-${shortcut.shift}`}
              className="flex items-center gap-3 px-4 py-2 hover:bg-[var(--ci-surface-sunken)] transition-colors"
            >
              <span className="flex-1 text-sm text-[var(--ci-ink)]">
                {shortcut.description}
              </span>
              <kbd className="shrink-0 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-sunken)] px-1.5 py-0.5 font-mono text-xs text-[var(--ci-ink-muted)]">
                {chord(shortcut)}
              </kbd>
            </li>
          ))}
        </ul>
      </div>
    </dialog>
  );
}
