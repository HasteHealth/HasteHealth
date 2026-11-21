import {
  acceptCompletion,
  autocompletion,
  closeCompletion,
  completionStatus,
  startCompletion,
} from "@codemirror/autocomplete";
import { EditorState, Extension, Prec } from "@codemirror/state";
import {
  EditorView,
  keymap,
  placeholder as placeholderExt,
} from "@codemirror/view";
import React, { useEffect, useMemo, useRef } from "react";

import { ClientProps } from "../types";
import {
  OperationCatalog,
  fhirPathCompletions,
  fhirSearchCompletions,
} from "./completion";
import { fhirSearchHighlightTheme, fhirSearchHighlighting } from "./highlight";
import { SearchMetadata } from "./metadata";

export interface FHIRSearchInputProps extends ClientProps {
  /** The query text, e.g. `Patient?name:exact=Smith`. */
  value?: string;
  /** Locks the input to one type; the text is then parameters only. */
  resourceType?: string;
  placeholder?: string;
  autoFocus?: boolean;
  readOnly?: boolean;
  /** Called on every edit. */
  onChange?: (value: string) => void;
  /** Called when Enter is pressed and no completion is open. */
  onSubmit?: (value: string) => void;
  /** Drops the border and padding, for an editor inside a command bar. */
  bare?: boolean;
  /**
   * Completes a whole path, `Patient/123/_history?_count=5`, taking the type
   * from the text. `resourceType` is then unused.
   */
  path?: boolean;
  /** Operations to offer after a `$`, in path mode. */
  operations?: OperationCatalog;
}

/** Keeps the document to one line by stripping newlines. */
const singleLine = EditorState.transactionFilter.of((transaction) => {
  if (!transaction.docChanged || transaction.newDoc.lines === 1) {
    return transaction;
  }
  // Flatten pasted multi-line text rather than rejecting it.
  const text = transaction.newDoc.toString().replace(/[\n\r]+/g, " ");
  return {
    changes: { from: 0, to: transaction.startState.doc.length, insert: text },
    selection: {
      anchor: Math.min(transaction.newSelection.main.head, text.length),
    },
  };
});

/** Styling for an editor inside a control that has its own frame. */
const bareTheme = EditorView.theme({
  "&": { width: "100%", fontSize: "0.875rem" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": {
    fontFamily:
      "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    overflowX: "auto",
    overflowY: "hidden",
    lineHeight: "1.5rem",
  },
  ".cm-content": { padding: "0", whiteSpace: "pre" },
  ".cm-line": { padding: "0" },
  ".cm-tooltip-autocomplete": {
    fontFamily:
      "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    fontSize: "0.8125rem",
  },
  ".cm-completionDetail": {
    marginLeft: "0.75rem",
    fontStyle: "normal",
    opacity: "0.6",
  },
});

/** Styling that makes the editor read as a one line text input. */
const inputTheme = EditorView.theme({
  "&": {
    width: "100%",
    fontSize: "0.875rem",
  },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": {
    fontFamily:
      "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    overflowX: "auto",
    overflowY: "hidden",
  },
  ".cm-content": {
    padding: "0.5rem 0.75rem",
    // A single line never wraps; it scrolls sideways instead.
    whiteSpace: "pre",
  },
  ".cm-line": { padding: "0" },
  ".cm-tooltip-autocomplete": {
    fontFamily:
      "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
    fontSize: "0.8125rem",
  },
  ".cm-completionDetail": {
    marginLeft: "0.75rem",
    fontStyle: "normal",
    opacity: "0.6",
  },
});

/**
 * A one line editor for FHIR search queries. Completion follows the cursor:
 * types at the start, search parameters after the `?`, modifiers after a `:`,
 * prefixes or codes after the `=`.
 *
 * Suggestions come from the server it points at, so they match what that
 * server accepts.
 */
export function FHIRSearchInput({
  client,
  fhirVersion,
  value = "",
  resourceType,
  placeholder = "Patient?name=Smith",
  autoFocus,
  readOnly = false,
  bare = false,
  path = false,
  operations,
  onChange,
  onSubmit,
}: Readonly<FHIRSearchInputProps>) {
  const parent = useRef<HTMLDivElement | null>(null);
  const view = useRef<EditorView | null>(null);

  // Read through a ref so new callbacks don't rebuild the editor.
  const handlers = useRef({ onChange, onSubmit });
  handlers.current = { onChange, onSubmit };

  const metadata = useMemo(
    () => new SearchMetadata(client, fhirVersion),
    [client, fhirVersion],
  );

  const extensions = useMemo((): Extension[] => {
    return [
      singleLine,
      bare ? bareTheme : inputTheme,
      fhirSearchHighlighting(Boolean(resourceType), path),
      fhirSearchHighlightTheme,
      placeholderExt(placeholder),
      autocompletion({
        override: [
          path
            ? fhirPathCompletions({ metadata, operations })
            : fhirSearchCompletions({ metadata, resourceType }),
        ],
        // The query is short, so fuzzy matches are more noise than help.
        filterStrict: true,
        icons: false,
        activateOnTyping: true,
      }),
      // Above the default keymap, so Enter submits and Tab completes.
      Prec.high(
        keymap.of([
          {
            key: "Enter",
            run: (editorView) => {
              // An open completion menu takes Enter first.
              if (completionStatus(editorView.state) === "active") return false;
              handlers.current.onSubmit?.(editorView.state.doc.toString());
              return true;
            },
          },
          { key: "Tab", run: acceptCompletion },
          { key: "Escape", run: closeCompletion },
          { key: "Mod-Space", run: startCompletion },
        ]),
      ),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          handlers.current.onChange?.(update.state.doc.toString());
        }
      }),
      EditorView.editable.of(!readOnly),
    ];
  }, [metadata, resourceType, placeholder, readOnly, bare, path, operations]);

  useEffect(() => {
    if (!parent.current) return undefined;
    const editorView = new EditorView({
      state: EditorState.create({ doc: value, extensions }),
      parent: parent.current,
    });
    view.current = editorView;
    if (autoFocus) editorView.focus();
    return () => {
      editorView.destroy();
      view.current = null;
    };
    // The effect below syncs `value`; rebuilding would lose the cursor.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [extensions]);

  // Adopt an outside `value` without disturbing the cursor.
  useEffect(() => {
    const editorView = view.current;
    if (!editorView) return;
    const current = editorView.state.doc.toString();
    if (current === value) return;
    const head = Math.min(editorView.state.selection.main.head, value.length);
    editorView.dispatch({
      changes: { from: 0, to: current.length, insert: value },
      selection: { anchor: head },
    });
  }, [value]);

  return (
    <div
      className={
        bare
          ? "w-full"
          : "w-full rounded-md border border-gray-300 bg-white focus-within:border-indigo-500 focus-within:ring-1 focus-within:ring-indigo-500"
      }
      ref={parent}
    />
  );
}
