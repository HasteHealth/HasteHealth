/**
 * Type specific panels for the console.
 *
 * Most resources are worked on as JSON, but a couple are really programs: a
 * `ViewDefinition` is a projection you run, and an `OperationDefinition`
 * carries source code you invoke. Those contribute extra tabs that edit the
 * same draft the JSON tab does, so one Save persists whatever was changed.
 *
 * The tabs are always siblings of the panel's own. Nothing here renders a tab
 * strip inside a tab strip.
 */
import { javascript } from "@codemirror/lang-javascript";
import { json } from "@codemirror/lang-json";
import { indentLess, insertTab } from "@codemirror/commands";
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
  VIEW_DEFINITION_PANES,
  ViewDefinitionSqlRunner,
} from "@haste-health/components";
import {
  AuditEvent,
  OperationDefinition,
  Resource,
  ViewDefinition,
  instant,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import { Operation } from "@haste-health/operation-execution";

import { getClient } from "../../db/client";
import { useRequest } from "../../hooks/useRequest";
import { getErrorMessage } from "../../utilities";

/** The extension an operation's source code is stored in. */
const CODE_URL = "https://haste.health/Extension/custom-code";
const CODE_TYPE_URL = "https://haste.health/Extension/custom-code-type";

/** History far enough back that a new view still returns rows. */
const SINCE = "1980-01-01T00:00:00Z" as instant;

const JSON_EXTENSIONS = [basicSetup, json()];

const CODE_EXTENSIONS = [
  basicSetup,
  javascript({ typescript: true }),
  keymap.of([
    { key: "Tab", preventDefault: true, run: insertTab },
    { key: "Shift-Tab", preventDefault: true, run: indentLess },
    // The panel owns saving, so the browser's own dialog is suppressed.
    { key: "Mod-s", run: () => true },
  ]),
];

const EDITOR_THEME = { "&": { height: "100%", width: "100%" } };

/** The source an operation starts with, so a new one is runnable as created. */
const DEFAULT_OPERATION_CODE = `interface Context {
  request: {
    id?: string;
    resource?: string;
    parameters: unknown;
  };
}

export default async function (context: Context) {
  // \`parameters\` holds the inputs declared on this OperationDefinition.
  const { input } = context.request.parameters as { input?: string };

  // Each output is a named parameter, so the resource is returned inside a
  // Parameters under the name this operation declares.
  return {
    resourceType: "Parameters",
    parameter: [
      {
        name: "basic",
        resource: {
          resourceType: "Basic",
          code: { text: \`echo: \${input ?? ""}\` },
        },
      },
    ],
  };
}
`;

/** A short note above a panel, explaining what it is for. */
function PanelHint({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="mb-2 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
      {children}
    </div>
  );
}

/** The source code on `operation`, or the empty string when it has none. */
export function operationCode(
  operation: OperationDefinition | undefined,
): string {
  return (
    operation?.extension?.find((e) => e.url === CODE_URL)?.valueString ?? ""
  );
}

/** `operation` with its source code replaced, leaving other extensions be. */
function withOperationCode(
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

/**
 * Whether a type has panels here, so a caller can offer them before the
 * resource exists.
 */
export function hasTypePanels(resourceType: string | undefined): boolean {
  return (
    resourceType === "OperationDefinition" || resourceType === "ViewDefinition"
  );
}

/**
 * The resource a new one of `resourceType` starts from, so the specialized
 * editors have something to show on a create.
 */
export function typeTemplate(resourceType: string): Resource | undefined {
  switch (resourceType) {
    case "OperationDefinition":
      return withOperationCode(
        {
          resourceType: "OperationDefinition",
          name: "NewOperation",
          status: "draft",
          kind: "operation",
          code: "new-operation",
          system: true,
          type: false,
          instance: false,
          // An input is only sent if it is declared here: the client builds
          // the Parameters body from this list, so an undeclared name is
          // dropped rather than transmitted.
          parameter: [
            {
              name: "input",
              use: "in",
              min: 0,
              max: "1",
              type: "string",
              documentation: "Replace with the inputs this operation takes.",
            },
            // Each declared output becomes a named parameter in the returned
            // Parameters. The one exception is a single output named `return`
            // typed as a resource, which FHIR treats as the response body
            // itself; that needs `min` 1, `max` "1" and a resource type.
            {
              name: "basic",
              use: "out",
              min: 1,
              max: "1",
              type: "Basic",
              documentation:
                "The resource this operation returns. Rename it and narrow the type to whatever this operation produces.",
            },
          ],
        } as unknown as OperationDefinition,
        DEFAULT_OPERATION_CODE,
      ) as Resource;
    case "ViewDefinition":
      return {
        resourceType: "ViewDefinition",
        status: "draft",
        resource: "Patient",
        select: [
          {
            column: [
              {
                name: "id",
                path: "id",
                type: "http://hl7.org/fhirpath/System.String",
              },
              { name: "birth_date", path: "$this.birthDate", type: "date" },
            ],
          },
        ],
      } as unknown as Resource;
    default:
      return undefined;
  }
}

/**
 * Runs the operation against parameters typed as JSON, and shows what came
 * back. The operation is invoked as it is currently saved, so unsaved edits
 * are called out rather than silently ignored.
 */
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
  const declared = (operation.parameter ?? []).filter((p) => p.use === "in");
  // The client builds the Parameters body from the declared inputs only, so a
  // name that is not declared never reaches the server. Saying so here beats
  // an empty request the user has to discover in the network tab.
  const [parameters, setParameters] = useState(() =>
    JSON.stringify(
      Object.fromEntries(declared.map((p) => [p.name as string, ""])),
      null,
      2,
    ),
  );
  const [output, setOutput] = useState<unknown>();
  const [running, setRunning] = useState(false);

  const typedNames = useMemo(() => {
    try {
      const parsed = JSON.parse(parameters) as Record<string, unknown>;
      return parsed && typeof parsed === "object" && !Array.isArray(parsed)
        ? Object.keys(parsed)
        : [];
    } catch {
      return [];
    }
  }, [parameters]);

  const undeclared = typedNames.filter(
    (name) => !declared.some((p) => p.name === name),
  );

  const run = () => {
    let parsed: unknown;
    try {
      parsed = JSON.parse(parameters);
    } catch {
      Toaster.error("Parameters are not valid JSON.");
      return;
    }
    setRunning(true);
    Toaster.promise(
      client
        .invoke_system(new Operation(operation), {}, R4, parsed)
        .finally(() => setRunning(false)),
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
          This runs the saved operation. Save first to invoke your current
          edits.
        </PanelHint>
      )}
      {declared.length === 0 ? (
        <div className="mb-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-900">
          This operation declares no input parameters, so nothing you type here
          is sent. Add a parameter with <code className="font-mono">use</code>{" "}
          of <code className="font-mono">in</code> on the OperationDefinition
          first.
        </div>
      ) : (
        <div className="mb-2 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
          Declared inputs:{" "}
          {declared.map((p) => (
            <code
              key={p.name as string}
              className="mr-1 rounded bg-white px-1 py-0.5 font-mono"
            >
              {p.name as string}
              {p.type ? `: ${p.type as string}` : ""}
            </code>
          ))}
        </div>
      )}
      {undeclared.length > 0 && (
        <div className="mb-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-900">
          Not declared on this operation, so{" "}
          {undeclared.length === 1 ? "it" : "they"} will not be sent:{" "}
          {undeclared.map((name) => (
            <code
              key={name}
              className="mr-1 rounded bg-white px-1 py-0.5 font-mono"
            >
              {name}
            </code>
          ))}
        </div>
      )}
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col">
          <span className="mb-1 text-xs font-medium text-slate-600">Input</span>
          <div className="flex h-56 overflow-auto rounded-md border border-slate-200">
            <CodeMirror
              extensions={JSON_EXTENSIONS}
              value={parameters}
              theme={EDITOR_THEME}
              onChange={setParameters}
            />
          </div>
        </div>
        <div className="flex flex-col">
          <span className="mb-1 text-xs font-medium text-slate-600">
            Output
          </span>
          <div className="flex h-56 overflow-auto rounded-md border border-slate-200">
            <CodeMirror
              readOnly
              extensions={JSON_EXTENSIONS}
              value={
                output === undefined
                  ? "// Nothing run yet."
                  : JSON.stringify(output, null, 2)
              }
              theme={EDITOR_THEME}
            />
          </div>
        </div>
      </div>
      <div className="mt-3 flex justify-end gap-2">
        <Button
          buttonType="secondary"
          buttonSize="small"
          onClick={() => setOpen(false)}
        >
          Close
        </Button>
        <Button buttonSize="small" disabled={running} onClick={run}>
          {running ? "Running…" : "Send"}
        </Button>
      </div>
    </div>
  );
}

/** Recent executions of this operation, newest first. */
function OperationLogs({ operationId }: Readonly<{ operationId: string }>) {
  const client = useAtomValue(getClient);
  const {
    data: auditEvents,
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

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <PanelHint>Recent executions of this operation.</PanelHint>
      {!loading && (auditEvents?.length ?? 0) === 0 ? (
        <p className="px-1 py-3 text-sm text-slate-500">
          This operation has not run yet.
        </p>
      ) : (
        <Table
          isLoading={loading}
          data={auditEvents ?? []}
          columns={[
            {
              id: "recorded",
              content: "When",
              selector: "$this.recorded",
              selectorType: "fhirpath",
            },
            {
              id: "outcome",
              content: "Outcome",
              selector: "$this.outcome",
              selectorType: "fhirpath",
            },
            {
              id: "agent",
              content: "Agent",
              selector: "$this.agent.name",
              selectorType: "fhirpath",
            },
            {
              id: "description",
              content: "Description",
              selector: "$this.outcomeDesc",
              selectorType: "fhirpath",
            },
          ]}
        />
      )}
    </div>
  );
}

/**
 * The operation's source, plus a way to run it. Edits go to the draft, so the
 * panel's own Save is what persists them.
 */
function OperationCodePanel({
  operation,
  dirty,
  readOnly,
  saved,
  onChange,
}: Readonly<{
  operation: OperationDefinition;
  dirty: boolean;
  readOnly: boolean;
  /** False while the resource is still being created. */
  saved: boolean;
  onChange: (resource: Resource) => void;
}>) {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <PanelHint>
        The TypeScript this operation runs, stored on the resource as a{" "}
        <code className="font-mono">custom-code</code> extension.{" "}
        {saved
          ? "Save the resource to deploy a change."
          : "It is saved with the resource when you create it."}
      </PanelHint>
      <div className="flex min-h-64 flex-1 overflow-auto rounded-md border border-slate-200">
        <CodeMirror
          readOnly={readOnly}
          extensions={CODE_EXTENSIONS}
          value={operationCode(operation)}
          theme={EDITOR_THEME}
          onChange={
            readOnly
              ? undefined
              : (value) => onChange(withOperationCode(operation, value))
          }
        />
      </div>
      {saved && (
        <div className="flex items-center gap-2 py-2">
          <Modal
            modalTitle={`Invoke ${operation.code ?? "operation"}`}
            ModalContent={(setOpen) => (
              <InvocationModal
                operation={operation}
                dirty={dirty}
                setOpen={setOpen}
              />
            )}
          >
            {(setOpen) => (
              <Button buttonSize="small" onClick={() => setOpen(true)}>
                <span className="flex items-center">
                  <PlayIcon className="mr-1 h-4 w-4" />
                  Invoke
                </span>
              </Button>
            )}
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

/**
 * One pane of the SQL runner.
 *
 * The runner owns the query it has run, so it is shown one pane at a time
 * rather than once per tab: the host's tab strip picks the pane, and the
 * runner never renders a strip of its own.
 */
function ViewDefinitionPane({
  view,
  pane,
  readOnly,
  onPaneChange,
  onChange,
}: Readonly<{
  view: ViewDefinition;
  pane: number;
  readOnly: boolean;
  onPaneChange: (pane: number) => void;
  onChange: (resource: Resource) => void;
}>) {
  const client = useAtomValue(getClient);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {pane === 0 && (
        <PanelHint>
          The columns this view projects. Run it to see the rows it produces.
        </PanelHint>
      )}
      <ViewDefinitionSqlRunner
        client={client}
        viewDefinition={view}
        setViewDefinition={
          readOnly ? () => {} : (next) => onChange(next as Resource)
        }
        editorExtensions={JSON_EXTENSIONS}
        defaultPageSize={10}
        fhirVersion={R4}
        since={SINCE}
        activePane={pane}
        onActivePaneChange={onPaneChange}
      />
    </div>
  );
}

/** A tab as {@link Tabs} takes it. */
export interface TypeTab {
  id: string;
  title: string;
  content: React.ReactNode;
}

export interface TypeTabsOptions {
  /** The resource as edited, which is what these panels show. */
  resource: Resource;
  /** True when the draft differs from what the server last returned. */
  dirty?: boolean;
  readOnly?: boolean;
  /** False while the resource is still being created. */
  saved?: boolean;
  onChange: (resource: Resource) => void;
  /** Which SQL runner pane is showing, and how to change it. */
  viewPane?: number;
  onViewPaneChange?: (pane: number) => void;
}

/**
 * The extra tabs `resource`'s type earns, in the order they should appear
 * before the generic ones. Empty for a type with no specialized editor.
 */
export function typeTabs({
  resource,
  dirty = false,
  readOnly = false,
  saved = true,
  onChange,
  viewPane = 0,
  onViewPaneChange = () => {},
}: TypeTabsOptions): TypeTab[] {
  switch (resource.resourceType) {
    case "OperationDefinition":
      return [
        {
          id: "code",
          title: "Code",
          content: (
            <OperationCodePanel
              operation={resource as OperationDefinition}
              dirty={dirty}
              readOnly={readOnly}
              saved={saved}
              onChange={onChange}
            />
          ),
        },
        ...(saved
          ? [
              {
                id: "logs",
                title: "Logs",
                content: <OperationLogs operationId={resource.id as string} />,
              },
            ]
          : []),
      ];

    case "ViewDefinition":
      // The runner's panes become tabs here, so there is one strip rather
      // than the runner's nested inside the panel's.
      return VIEW_DEFINITION_PANES.map((definition) => ({
        id: `view-${definition.id}`,
        title: definition.id === 0 ? "Projection" : definition.title,
        content: (
          <ViewDefinitionPane
            view={resource as ViewDefinition}
            pane={definition.id}
            readOnly={readOnly}
            onPaneChange={onViewPaneChange}
            onChange={onChange}
          />
        ),
      }));

    default:
      return [];
  }
}
