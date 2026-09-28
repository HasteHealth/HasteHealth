import React from "react";

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

/** The list of shortcuts, shown over the console on `?`. */
export function ShortcutHelp({ shortcuts, onClose }: Readonly<ShortcutHelpProps>) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/30 p-4"
      onClick={onClose}
      role="presentation"
    >
      <div
        className="max-h-[80vh] w-full max-w-md overflow-auto rounded-lg border border-slate-200 bg-white shadow-xl"
        onClick={(event) => event.stopPropagation()}
        role="dialog"
        aria-label="Keyboard shortcuts"
      >
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
    </div>
  );
}
