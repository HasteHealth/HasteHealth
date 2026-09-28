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
 *
 * A native `<dialog>` rather than a styled div: it traps focus, closes on Esc
 * and announces itself as a dialog without any of that being reimplemented.
 */
export function ShortcutHelp({
  shortcuts,
  onClose,
}: Readonly<ShortcutHelpProps>) {
  const dialog = useRef<HTMLDialogElement | null>(null);

  useEffect(() => {
    // `showModal` is what gives the backdrop and the focus trap, so the
    // dialog is opened here rather than with the `open` attribute.
    dialog.current?.showModal();
  }, []);

  return (
    <dialog
      ref={dialog}
      aria-label="Keyboard shortcuts"
      className="max-h-[80vh] w-full max-w-md rounded-lg border border-slate-200 bg-white p-0 shadow-xl backdrop:bg-slate-900/30"
      onClose={onClose}
      // A click on the backdrop reports the dialog itself as the target; the
      // content sits in a child, so anything inside it is ignored here.
      onClick={(event) => {
        if (event.target === dialog.current) onClose();
      }}
    >
      <div className="max-h-[80vh] overflow-auto">
        <header className="flex items-center border-b border-slate-200 px-4 py-2">
          <h2 className="flex-1 text-sm font-medium text-slate-800">
            Keyboard shortcuts
          </h2>
          <button
            className="rounded px-2 py-1 text-xs text-slate-500 hover:bg-slate-100"
            onClick={onClose}
            type="button"
          >
            Esc
          </button>
        </header>
        <ul className="divide-y divide-slate-100">
          {shortcuts.map((shortcut) => (
            <li
              key={`${shortcut.key}-${shortcut.mod}-${shortcut.shift}`}
              className="flex items-center gap-3 px-4 py-2"
            >
              <span className="flex-1 text-sm text-slate-700">
                {shortcut.description}
              </span>
              <kbd className="shrink-0 rounded border border-slate-300 bg-slate-50 px-1.5 py-0.5 font-mono text-xs text-slate-600">
                {chord(shortcut)}
              </kbd>
            </li>
          ))}
        </ul>
      </div>
    </dialog>
  );
}
