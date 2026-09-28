/**
 * Views for an OperationDefinition: its source code, and its run history.
 *
 * The code lives on the resource as a `custom-code` extension, so editing it
 * is editing the draft and the panel's Save is what deploys it.
 */
import { indentLess, insertTab } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";
import { keymap } from "@codemirror/view";
import { PlayIcon } from "@heroicons/react/24/outline";
import { basicSetup } from "codemirror";
import { useAtomValue } from "jotai";
import React, { useMemo, useState } from "react";

import {
  Button,
  CodeMirror,
  Modal,
  OutcomePanel,
  Table,
  Toaster,
} from "@haste-health/components";
import {
  AuditEvent,
  OperationDefinition,
  Resource,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../../db/client";
import { useRequest } from "../../../hooks/useRequest";
import { getErrorMessage } from "../../../utilities";
import {
  InvokeLevel,
  InvokeTarget,
  declaredLevels,
  declaredResourceTypes,
  describeInvocation,
  invocationBlocker,
  invoke,
} from "../invoke";
import { registerResourceViews } from "../registry";
import {
  Chip,
  EDITOR_THEME,
  JSON_EXTENSIONS,
  PanelHint,
  PanelWarning,
} from "./shared";

/** Where an operation's source code is stored on the resource. */
const CODE_URL = "https://haste.health/Extension/custom-code";
const CODE_TYPE_URL = "https://haste.health/Extension/custom-code-type";

const CODE_EXTENSIONS = [
  basicSetup,
  javascript({ typescript: true }),
  keymap.of([
    { key: "Tab", preventDefault: true, run: insertTab },
    { key: "Shift-Tab", preventDefault: true, run: indentLess },
    // The panel owns saving, so suppress the browser's save dialog.
    { key: "Mod-s", run: () => true },
  ]),
];

/** The source code on `operation`, or "" when it has none. */
function codeOf(operation: OperationDefinition): string {
  return (
    operation.extension?.find((e) => e.url === CODE_URL)?.valueString ?? ""
  );
}

/** `operation` with its source replaced, leaving other extensions alone. */
export function withCode(
  operation: OperationDefinition,
  code: string,
): OperationDefinition {
  return {
    ...operation,
    extension: [
      ...(operation.extension?.filter((e) => e.url !== CODE_URL) ?? []),
      {
        url: CODE_URL,
        valueString: code,
        extension: [{ url: CODE_TYPE_URL, valueString: "text/typescript" }],
      },
    ],
  } as OperationDefinition;
}

/** The inputs `operation` declares; only these are sent. */
function declaredInputs(operation: OperationDefinition) {
  return (operation.parameter ?? []).filter((p) => p.use === "in");
}

/** Top level keys of `text`, or [] when it is not a JSON object. */
function objectKeys(text: string): string[] {
  try {
    const parsed = JSON.parse(text) as unknown;
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? Object.keys(parsed as Record<string, unknown>)
      : [];
  } catch {
    return [];
  }
}

/** Picks what to invoke against: the level, and its type and id. */
function TargetControls({
  operation,
  target,
  onChange,
}: Readonly<{
  operation: OperationDefinition;
  target: InvokeTarget;
  onChange: (target: InvokeTarget) => void;
}>) {
  const levels = declaredLevels(operation);
  const resourceTypes = declaredResourceTypes(operation);
  const field = "mt-0.5 rounded border border-slate-300 px-2 py-1 text-xs";

  return (
    <div className="mb-2 flex flex-wrap items-end gap-2">
      <label className="flex flex-col text-xs text-slate-600">
        <span>Level</span>
        <select
          className={field}
          value={target.level}
          onChange={(e) =>
            onChange({ ...target, level: e.target.value as InvokeLevel })
          }
        >
          {levels.map((level) => (
            <option key={level} value={level}>
              {level}
            </option>
          ))}
        </select>
      </label>

      {/* Free text when the operation names no types, since it applies to any
          of them. */}
      {target.level !== "system" && (
        <label className="flex flex-col text-xs text-slate-600">
          <span>Resource type</span>
          {resourceTypes.length > 0 ? (
            <select
              className={field}
              value={target.resourceType ?? ""}
              onChange={(e) =>
                onChange({ ...target, resourceType: e.target.value })
              }
            >
              {resourceTypes.map((type) => (
                <option key={type} value={type}>
                  {type}
                </option>
              ))}
            </select>
          ) : (
            <input
              className={`${field} font-mono`}
              placeholder="Patient"
              value={target.resourceType ?? ""}
              onChange={(e) =>
                onChange({ ...target, resourceType: e.target.value })
              }
            />
          )}
        </label>
      )}

      {target.level === "instance" && (
        <label className="flex flex-col text-xs text-slate-600">
          <span>Id</span>
          <input
            className={`${field} font-mono`}
            placeholder="123"
            value={target.id ?? ""}
            onChange={(e) => onChange({ ...target, id: e.target.value })}
          />
        </label>
      )}

      <span className="ml-auto font-mono text-xs text-slate-500">
        {describeInvocation(operation, target)}
      </span>
    </div>
  );
}

/** Runs the operation and shows what came back. */
function InvocationModal({
  operation,
  dirty,
  setOpen,
}: Readonly<{
  operation: OperationDefinition;
  dirty: boolean;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}>) {
  const client = useAtomValue(getClient);
  const declared = declaredInputs(operation);

  const [target, setTarget] = useState<InvokeTarget>(() => ({
    level: declaredLevels(operation)[0] ?? "system",
    resourceType: declaredResourceTypes(operation)[0],
  }));
  // Start from the declared names so the right shape is the default.
  const [parameters, setParameters] = useState(() =>
    JSON.stringify(
      Object.fromEntries(declared.map((p) => [p.name as string, ""])),
      null,
      2,
    ),
  );
  const [output, setOutput] = useState<unknown>();
  const [running, setRunning] = useState(false);

  // Undeclared names are dropped when the body is built, so say so rather
  // than let the user find an empty request in the network tab.
  const undeclared = useMemo(
    () =>
      objectKeys(parameters).filter(
        (name) => !declared.some((p) => p.name === name),
      ),
    [parameters, declared],
  );

  const blocker = invocationBlocker(operation, target);

  const run = () => {
    let input: unknown;
    try {
      input = JSON.parse(parameters);
    } catch {
      Toaster.error("Parameters are not valid JSON.");
      return;
    }
    setRunning(true);
    Toaster.promise(
      invoke(client, operation, target, input).finally(() => setRunning(false)),
      {
        loading: "Invoking",
        success: (result) => {
          setOutput(result);
          return "Invocation succeeded";
        },
        error: (error) => getErrorMessage(error),
      },
    );
  };

  return (
    <div>
      {dirty && (
        <PanelHint>
          This runs the saved operation. Save first to invoke your edits.
        </PanelHint>
      )}

      <TargetControls
        operation={operation}
        target={target}
        onChange={setTarget}
      />

      {declared.length === 0 ? (
        <PanelWarning>
          This operation declares no inputs, so nothing typed here is sent. Add
          a parameter with <Chip>use</Chip> of <Chip>in</Chip> first.
        </PanelWarning>
      ) : (
        <PanelHint>
          Declared inputs:{" "}
          {declared.map((p) => (
            <Chip key={p.name as string}>
              {p.name as string}
              {p.type ? `: ${p.type as string}` : ""}
            </Chip>
          ))}
        </PanelHint>
      )}

      {undeclared.length > 0 && (
        <PanelWarning>
          Not declared, so {undeclared.length === 1 ? "it" : "they"} will not be
          sent:{" "}
          {undeclared.map((name) => (
            <Chip key={name}>{name}</Chip>
          ))}
        </PanelWarning>
      )}

      <div className="grid grid-cols-2 gap-3">
        <Editor label="Input" value={parameters} onChange={setParameters} />
        <Editor
          label="Output"
          value={
            output === undefined
              ? "// Nothing run yet."
              : JSON.stringify(output, null, 2)
          }
        />
      </div>

      <div className="mt-3 flex items-center justify-end gap-2">
        {blocker && (
          <span className="mr-auto text-xs text-amber-700">{blocker}</span>
        )}
        <Button
          buttonType="secondary"
          buttonSize="small"
          onClick={() => setOpen(false)}
        >
          Close
        </Button>
        <Button
          buttonSize="small"
          disabled={running || blocker !== undefined}
          onClick={run}
        >
          {running ? "Running…" : "Send"}
        </Button>
      </div>
    </div>
  );
}

/** A labelled JSON editor; read only when `onChange` is omitted. */
function Editor({
  label,
  value,
  onChange,
}: Readonly<{
  label: string;
  value: string;
  onChange?: (value: string) => void;
}>) {
  return (
    <div className="flex flex-col">
      <span className="mb-1 text-xs font-medium text-slate-600">{label}</span>
      <div className="flex h-56 overflow-auto rounded-md border border-slate-200">
        <CodeMirror
          readOnly={!onChange}
          extensions={JSON_EXTENSIONS}
          value={value}
          theme={EDITOR_THEME}
          onChange={onChange}
        />
      </div>
    </div>
  );
}

/** `Modal`'s content: the invocation dialog for `operation`. */
function renderInvocationModal(operation: OperationDefinition, dirty: boolean) {
  return function ModalContent(
    setOpen: React.Dispatch<React.SetStateAction<boolean>>,
  ) {
    return (
      <InvocationModal operation={operation} dirty={dirty} setOpen={setOpen} />
    );
  };
}

/** `Modal`'s trigger: opens the invocation dialog. */
function renderInvokeButton(setOpen: (open: boolean) => void) {
  return (
    <Button buttonSize="small" onClick={() => setOpen(true)}>
      <span className="flex items-center">
        <PlayIcon className="mr-1 h-4 w-4" />
        Invoke
      </span>
    </Button>
  );
}

/** The operation's source, plus a way to run it. */
function CodeView({
  operation,
  dirty,
  readOnly,
  saved,
  onChange,
}: Readonly<{
  operation: OperationDefinition;
  dirty: boolean;
  readOnly: boolean;
  saved: boolean;
  onChange: (resource: Resource) => void;
}>) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <PanelHint>
        The TypeScript this operation runs, stored as a <Chip>custom-code</Chip>{" "}
        extension.{" "}
        {saved
          ? "Save the resource to deploy a change."
          : "It is saved with the resource when you create it."}
      </PanelHint>

      <div className="flex min-h-64 flex-1 overflow-auto rounded-md border border-slate-200">
        <CodeMirror
          readOnly={readOnly}
          extensions={CODE_EXTENSIONS}
          value={codeOf(operation)}
          theme={EDITOR_THEME}
          onChange={
            readOnly ? undefined : (v) => onChange(withCode(operation, v))
          }
        />
      </div>

      {/* Invoking needs a saved operation, so it only appears once there is
          one. */}
      {saved && (
        <div className="flex items-center gap-2 py-2">
          <Modal
            modalTitle={`Invoke ${operation.code ?? "operation"}`}
            ModalContent={renderInvocationModal(operation, dirty)}
          >
            {renderInvokeButton}
          </Modal>
          {dirty && (
            <span className="text-xs text-amber-700">
              Unsaved changes: invoking runs the last saved version.
            </span>
          )}
        </div>
      )}
    </div>
  );
}

/** Recent executions of this operation, newest first. */
function LogsView({ operationId }: Readonly<{ operationId: string }>) {
  const client = useAtomValue(getClient);
  const {
    data: events,
    loading,
    outcome,
  } = useRequest<AuditEvent[]>(
    () =>
      client
        .search_type({}, R4, "AuditEvent", [
          { name: "entity", value: [operationId] },
          { name: "_sort", value: ["-date"] },
          { name: "_count", value: ["50"] },
        ])
        .then((response) => response.resources as AuditEvent[]),
    [client, operationId],
  );

  if (outcome) return <OutcomePanel outcome={outcome} />;

  const columns = [
    { id: "recorded", content: "When", path: "$this.recorded" },
    { id: "outcome", content: "Outcome", path: "$this.outcome" },
    { id: "agent", content: "Agent", path: "$this.agent.name" },
    { id: "description", content: "Description", path: "$this.outcomeDesc" },
  ].map(({ id, content, path }) => ({
    id,
    content,
    selector: path,
    selectorType: "fhirpath" as const,
  }));

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <PanelHint>Recent executions of this operation.</PanelHint>
      {!loading && (events?.length ?? 0) === 0 ? (
        <p className="px-1 py-3 text-sm text-slate-500">
          This operation has not run yet.
        </p>
      ) : (
        <Table isLoading={loading} data={events ?? []} columns={columns} />
      )}
    </div>
  );
}

registerResourceViews("OperationDefinition", {
  views: ({ resource, dirty, readOnly, saved, onChange }) => [
    {
      id: "code",
      title: "Code",
      content: (
        <CodeView
          operation={resource as OperationDefinition}
          dirty={dirty}
          readOnly={readOnly}
          saved={saved}
          onChange={onChange}
        />
      ),
    },
    // No logs before the operation exists: there is nothing to look up.
    ...(saved
      ? [
          {
            id: "logs",
            title: "Logs",
            content: <LogsView operationId={resource.id as string} />,
          },
        ]
      : []),
  ],
});
