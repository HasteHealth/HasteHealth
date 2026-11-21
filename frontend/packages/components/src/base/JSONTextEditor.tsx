import { json } from "@codemirror/lang-json";
import {
  CheckCircleIcon,
  ExclamationCircleIcon,
  SparklesIcon,
} from "@heroicons/react/24/outline";
import { basicSetup } from "codemirror";
import React, { useMemo } from "react";

import { Button } from "./button";
import { CodeMirror } from "./codemirror";

const extensions = [basicSetup, json()];

export interface JSONTextEditorProps {
  value: string;
  onChange: (value: string) => void;
  readOnly?: boolean;
  /** Shown above the editor, e.g. what the body is for. */
  hint?: string;
}

/** The parse error for `text`, if it does not parse. */
function parseError(text: string): string | undefined {
  if (text.trim() === "") return undefined;
  try {
    JSON.parse(text);
    return undefined;
  } catch (error) {
    return error instanceof Error ? error.message : "Invalid JSON";
  }
}

/**
 * Edits arbitrary JSON as text.
 *
 * Unlike the resource editor it never reformats under the cursor, which matters
 * for a body that is not a resource (a JSON Patch array) or does not parse yet.
 */
export function JSONTextEditor({
  value,
  onChange,
  readOnly,
  hint,
}: Readonly<JSONTextEditorProps>) {
  const error = useMemo(() => parseError(value), [value]);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="mb-2 flex items-center gap-2">
        {hint && <span className="text-xs text-slate-500">{hint}</span>}
        <div className="flex flex-1 items-center justify-end gap-2">
          {error ? (
            <span
              className="flex items-center text-xs text-red-600"
              title={error}
            >
              <ExclamationCircleIcon className="mr-1 h-4 w-4" />
              <span className="max-w-80 truncate">{error}</span>
            </span>
          ) : (
            value.trim() !== "" && (
              <span className="flex items-center text-xs text-emerald-600">
                <CheckCircleIcon className="mr-1 h-4 w-4" />
                Valid
              </span>
            )
          )}
          {!readOnly && (
            <Button
              buttonSize="small"
              buttonType="secondary"
              disabled={error !== undefined}
              onClick={() => {
                try {
                  onChange(JSON.stringify(JSON.parse(value), null, 2));
                } catch {
                  // Unreachable: the button is disabled when invalid.
                }
              }}
            >
              <span className="flex items-center">
                <SparklesIcon className="mr-1 h-4 w-4" />
                Format
              </span>
            </Button>
          )}
        </div>
      </div>

      <div className="flex min-h-0 flex-1 overflow-auto border">
        <CodeMirror
          readOnly={readOnly}
          extensions={extensions}
          value={value}
          theme={{ "&": { height: "100%", width: "100%" } }}
          onChange={onChange}
        />
      </div>
    </div>
  );
}
