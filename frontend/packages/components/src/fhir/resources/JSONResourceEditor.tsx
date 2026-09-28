import { json } from "@codemirror/lang-json";
import {
  ArrowDownTrayIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  SparklesIcon,
} from "@heroicons/react/24/outline";
import { basicSetup } from "codemirror";
import React, { useEffect, useMemo, useState } from "react";

import { Button } from "../../base/button";
import { CodeMirror } from "../../base/codemirror";
import { Resource } from "@haste-health/fhir-types/r4/types";

const extensions = [basicSetup, json()];

export interface JSONResourceEditorProps {
  resource: Resource | undefined;
  /** Called with the parsed resource when the text is valid JSON. */
  onChange?: (resource: Resource) => void;
}

/** Pretty prints `text`, or `undefined` when it is not valid JSON. */
function format(text: string): string | undefined {
  try {
    return JSON.stringify(JSON.parse(text), null, 2);
  } catch {
    return undefined;
  }
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
 * Edits a resource as raw JSON, the form a developer wants to see.
 *
 * The text is the source of truth while typing and the parsed resource is only
 * handed up when valid, so a half typed edit never destroys the document.
 */
export function JSONResourceEditor({
  resource,
  onChange,
}: Readonly<JSONResourceEditorProps>) {
  const incoming = useMemo(
    () => (resource ? JSON.stringify(resource, null, 2) : "{}"),
    [resource],
  );
  const [text, setText] = useState(incoming);

  // Adopt a resource replaced from outside (a reload, a save response), but
  // not the echo of our own edit, which would reformat under the cursor.
  const [lastSent, setLastSent] = useState<string>();
  useEffect(() => {
    if (incoming === lastSent) return;
    setText(incoming);
  }, [incoming, lastSent]);

  const error = useMemo(() => parseError(text), [text]);

  const onTextChange = (value: string) => {
    setText(value);
    if (!onChange) return;
    try {
      const parsed = JSON.parse(value) as Resource;
      // Recognise this same resource coming back down as our own echo.
      setLastSent(JSON.stringify(parsed, null, 2));
      onChange(parsed);
    } catch {
      // Invalid mid edit is normal; the next valid keystroke updates.
    }
  };

  return (
    <div className="flex flex-1 flex-col">
      <div className="mb-2 flex items-center gap-2">
        <Button
          buttonSize="small"
          buttonType="secondary"
          disabled={error !== undefined}
          onClick={() => {
            const formatted = format(text);
            if (formatted !== undefined) onTextChange(formatted);
          }}
        >
          <span className="flex items-center">
            <SparklesIcon className="mr-1 h-4 w-4" />
            Format
          </span>
        </Button>
        <Button
          buttonSize="small"
          buttonType="secondary"
          disabled={error !== undefined}
          onClick={() => {
            const minified = (() => {
              try {
                return JSON.stringify(JSON.parse(text));
              } catch {
                return undefined;
              }
            })();
            if (minified !== undefined) onTextChange(minified);
          }}
        >
          <span className="flex items-center">
            <ArrowDownTrayIcon className="mr-1 h-4 w-4 rotate-90" />
            Minify
          </span>
        </Button>

        <div className="flex flex-1 justify-end">
          {error ? (
            <span
              className="flex items-center text-xs text-red-600"
              title={error}
            >
              <ExclamationCircleIcon className="mr-1 h-4 w-4" />
              <span className="max-w-96 truncate">{error}</span>
            </span>
          ) : (
            <span className="flex items-center text-xs text-emerald-600">
              <CheckCircleIcon className="mr-1 h-4 w-4" />
              Valid JSON
            </span>
          )}
        </div>
      </div>

      <div className="flex flex-1 overflow-auto border">
        <CodeMirror
          extensions={extensions}
          value={text}
          theme={{ "&": { height: "100%", width: "100%" } }}
          onChange={onTextChange}
        />
      </div>
    </div>
  );
}
