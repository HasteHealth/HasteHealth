import {
  Annotation,
  EditorState,
  Extension,
  StateEffect,
} from "@codemirror/state";
import { EditorView, ViewUpdate } from "@codemirror/view";
import React, { useEffect, useRef, useState } from "react";

const SyncAnnotation = Annotation.define();

function getIsSyncUpdate(viewUpdate: ViewUpdate) {
  const transactions = viewUpdate.transactions;
  return transactions.reduce((isUserTriggered: boolean, transaction) => {
    const syncAnnotationExists =
      transaction.annotation(SyncAnnotation) !== undefined;
    return isUserTriggered || syncAnnotationExists;
  }, false);
}

function resync(view: EditorView | undefined, value: string | undefined = "") {
  if (view) {
    const currentValue = view.state.doc.toString();
    const currentSelection = view.state.selection.ranges[0];
    const offsetFromEnd = currentValue.length - currentSelection.head;
    if (value !== currentValue) {
      view.dispatch({
        selection: {
          anchor: Math.max(0, value.length - offsetFromEnd),
          head: Math.max(0, value.length - offsetFromEnd),
        },
        annotations: [SyncAnnotation.of("syncing")],
        changes: {
          from: 0,
          to: currentValue.length,
          insert: value,
        },
      });
    }
  }
}

function createUpdateListener(
  onChange: CodeMirrorProps["onChange"],
): Extension {
  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      const doc = update.state.doc;
      const newValue = doc.toString();
      const isSyncUpdate = getIsSyncUpdate(update);
      if (!isSyncUpdate && onChange) {
        onChange(newValue, update);
      }
    }
  });
  return updateListener;
}

export interface CodeMirrorProps {
  readOnly?: boolean;
  value?: string;
  autoFocus?: boolean;
  extensions?: Extension[];
  theme?: Parameters<typeof EditorView.theme>[0];
  onChange?: (value: string, viewUpdate: ViewUpdate) => void;
}

function createExtensions({
  readOnly,
  extensions,
  theme,
  onChange,
}: {
  readOnly: boolean;
  extensions?: Extension[];
  theme?: Parameters<typeof EditorView.theme>[0];
  onChange: CodeMirrorProps["onChange"];
}) {
  const extensionsToUse: Extension[] = [...(extensions || [])];
  if (theme) extensionsToUse?.push(EditorView.theme(theme));
  if (onChange) extensionsToUse?.push(createUpdateListener(onChange));
  if (readOnly) extensionsToUse?.push(EditorView.editable.of(false));

  return extensionsToUse;
}

export const CodeMirror = ({
  readOnly = false,
  value,
  autoFocus,
  extensions = [],
  theme = {},
  onChange,
}: CodeMirrorProps) => {
  const [view, setView] = useState<EditorView>();
  const [, setState] = useState<EditorState>();
  const root = useRef<HTMLDivElement | null>(null);

  // Initial view set up.
  useEffect(() => {
    if (root.current) {
      root.current.innerHTML = "";

      const state = EditorState.create({
        doc: value || "",
        extensions: createExtensions({
          readOnly,
          extensions,
          theme,
          onChange,
        }),
      });
      const view = new EditorView({
        state: state,
        parent: root.current,
      });
      if (autoFocus) {
        view.focus();
      }

      setView(view);
      setState(state);
    }
  }, [root, setView, setState, readOnly]);

  // Synchronization on new value passed in.
  useEffect(() => {
    resync(view, value);
  }, [view, value]);

  // Reconfiguring extensions.
  useEffect(() => {
    if (view) {
      view.dispatch({
        effects: StateEffect.reconfigure.of(
          createExtensions({
            readOnly,
            extensions,
            theme,
            onChange,
          }),
        ),
      });
    }
  }, [view, extensions, theme, onChange, readOnly]);

  return <div className="flex flex-grow" ref={root} />;
};
