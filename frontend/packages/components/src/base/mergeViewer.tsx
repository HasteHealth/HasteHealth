import { MergeView } from "@codemirror/merge";
import { EditorState, Extension } from "@codemirror/state";
import { EditorView } from "codemirror";
import React, { useEffect, useRef, useState } from "react";

interface DiffViewProps {
  extensions?: Extension[];
  oldValue: string;
  newValue: string;
}

export function MergeViewer({
  extensions,
  oldValue,
  newValue,
}: Readonly<DiffViewProps>) {
  const [_view, setView] = useState<MergeView>();
  const [_state, setState] = useState<EditorState>();
  const root = useRef<HTMLDivElement | null>(null);

  // Initial view set up.
  useEffect(() => {
    if (root.current) {
      root.current.innerHTML = "";

      const view = new MergeView({
        a: {
          doc: oldValue,
          extensions: [
            ...(extensions ?? []),
            EditorView.editable.of(false),
            EditorState.readOnly.of(true),
          ],
        },
        b: {
          doc: newValue,
          extensions: [
            ...(extensions ?? []),
            EditorView.editable.of(false),
            EditorState.readOnly.of(true),
          ],
        },
        parent: root.current,
      });

      setView(view);
    }
  }, [root, setView, setState]);

  return <div className="flex flex-grow" ref={root} />;
}
