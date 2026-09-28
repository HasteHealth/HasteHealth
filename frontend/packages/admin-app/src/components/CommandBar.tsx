import { ExclamationTriangleIcon } from "@heroicons/react/24/outline";
import { PlayIcon } from "@heroicons/react/24/solid";
import classNames from "classnames";
import { useAtomValue } from "jotai";
import React, { useEffect, useMemo, useState } from "react";

import { FHIRSearchInput } from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../db/client";
import {
  Command,
  VERBS,
  Verb,
  describeRequest,
  isMutation,
  mutationWarning,
  parseCommand,
  targetPath,
} from "../query/model";

export interface CommandBarProps {
  /** The command currently shown, used to seed the input. */
  command?: Command;
  onRun: (command: Command) => void;
  /**
   * Called when the verb changes, so the console can switch to the write
   * editor without waiting for the command to be run.
   */
  onVerbChange?: (verb: Verb, command: Command | undefined) => void;
  busy?: boolean;
}

/** Colour for each verb, so the destructive ones read as such. */
const VERB_CLASS: Record<Verb, string> = {
  GET: "text-slate-700",
  POST: "text-emerald-700",
  PUT: "text-blue-700",
  PATCH: "text-violet-700",
  DELETE: "text-red-700",
};

/** Colours the Run button: amber for a mutation, brand for a read. */
function runButtonClass(parsed: Command | undefined): string {
  if (!parsed) return "cursor-not-allowed bg-slate-100 text-slate-400";
  return isMutation(parsed)
    ? "bg-amber-600 text-white hover:bg-amber-700"
    : "bg-brand-600 text-white hover:bg-brand-700";
}

/**
 * The console's one input.
 *
 * The whole request is a single editor: the path and the search after its
 * `?` are one document, completed by one source, so there is no seam to
 * cross while typing. The editor is the product's own FHIR input in path
 * mode, which is what keeps the search half - parameters, modifiers,
 * prefixes, chains - identical to everywhere else it is used.
 */
export function CommandBar({
  command,
  onRun,
  onVerbChange,
  busy,
}: Readonly<CommandBarProps>) {
  const client = useAtomValue(getClient);
  const [verb, setVerb] = useState<Verb>(command?.verb ?? "GET");
  const [text, setText] = useState(command ? targetPath(command) : "");

  // Adopt a command that changed underneath us: the sidebar, the back
  // button, a row click.
  useEffect(() => {
    setVerb(command?.verb ?? "GET");
    setText(command ? targetPath(command) : "");
  }, [command]);

  const parsed = useMemo(
    () => parseCommand(text.trim() === "" ? "" : `${verb} ${text}`),
    [verb, text],
  );
  const warning = parsed ? mutationWarning(parsed) : undefined;
  const request = parsed ? describeRequest(parsed) : undefined;

  const run = (value?: string) => {
    // No value means run what is already parsed; otherwise re-parse, since
    // the text may have changed since the last render.
    let next = parsed;
    if (value !== undefined) {
      const text = value.trim() === "" ? "" : `${verb} ${value}`;
      next = parseCommand(text);
    }
    if (next) onRun(next);
  };

  return (
    <div>
      <div
        className={classNames(
          "flex items-center gap-2 rounded-md border bg-white px-2 py-1.5 shadow-sm",
          warning ? "border-amber-300" : "border-slate-300",
          "focus-within:border-brand-500 focus-within:ring-1 focus-within:ring-brand-500",
        )}
      >
        <select
          aria-label="HTTP method"
          className={classNames(
            "shrink-0 rounded border-0 bg-slate-100 py-1 pl-2 pr-7 font-mono text-xs font-semibold focus:ring-0",
            VERB_CLASS[verb],
          )}
          value={verb}
          onChange={(event) => {
            const next = event.target.value as Verb;
            setVerb(next);
            // Choosing a write is itself the intent to write, so the console
            // switches to the editor now rather than after a separate press.
            onVerbChange?.(
              next,
              parseCommand(text.trim() === "" ? "" : `${next} ${text}`),
            );
          }}
        >
          {VERBS.map((v) => (
            <option key={v} value={v}>
              {v}
            </option>
          ))}
        </select>

        <div className="min-w-0 flex-1" data-command-input>
          <FHIRSearchInput
            bare
            path
            autoFocus
            client={client}
            fhirVersion={R4}
            value={text}
            onChange={setText}
            onSubmit={run}
            placeholder="Patient?name=Smith   ·   Patient/123   ·   _history"
          />
        </div>

        <button
          className={classNames(
            "shrink-0 rounded px-2 py-1 text-xs font-medium",
            runButtonClass(parsed),
          )}
          disabled={!parsed || busy}
          onClick={() => run()}
          type="button"
        >
          <span className="flex items-center gap-1">
            <PlayIcon className="h-3 w-3" />
            {busy ? "Running" : "Run"}
          </span>
        </button>
      </div>

      {/* What the typing actually sends, so the API is learnable by use. */}
      <div className="mt-1 flex min-h-5 items-center gap-2 px-1 text-xs">
        {request ? (
          <span className="font-mono text-slate-500">
            {request.method} {request.path}
          </span>
        ) : (
          text.trim() !== "" && (
            <span className="text-slate-400">Keep typing…</span>
          )
        )}
        {warning && (
          <span className="flex items-center gap-1 text-amber-700">
            <ExclamationTriangleIcon className="h-3.5 w-3.5" />
            {warning}
          </span>
        )}
      </div>
    </div>
  );
}
