/** Pieces shared by the per-type views. */
import { json } from "@codemirror/lang-json";
import { basicSetup } from "codemirror";
import React from "react";

export const JSON_EXTENSIONS = [basicSetup, json()];

/** Fills the editor's container, which supplies the height. */
export const EDITOR_THEME = { "&": { height: "100%", width: "100%" } };

/** A one line note above a panel saying what it is for. */
export function PanelHint({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="mb-2 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
      {children}
    </div>
  );
}

/** A warning the user must act on, e.g. input that will not be sent. */
export function PanelWarning({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="mb-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-900">
      {children}
    </div>
  );
}

/** An inline code chip, for parameter and field names. */
export function Chip({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <code className="mr-1 rounded bg-white px-1 py-0.5 font-mono">
      {children}
    </code>
  );
}
