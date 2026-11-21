/**
 * Views for an OperationDefinition: its source code, and its run history.
 *
 * The code lives on the resource as a `custom-code` extension, so editing it
 * is editing the draft and the panel's Save is what deploys it.
 */
import { indentLess, insertTab } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";
import { keymap } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { useAtomValue } from "jotai";
import React from "react";

import {
  CodeMirror,
  OperationInvocationPanel,
  OutcomePanel,
  Table,
} from "@haste-health/components";
import {
  AuditEvent,
  OperationDefinition,
  Resource,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../../db/client";
import { useRequest } from "../../../hooks/useRequest";
import { registerResourceViews } from "../registry";
import { Chip, EDITOR_THEME, PanelHint } from "./shared";

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

/** The operation's source, with the panel that runs it beside it. */
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
  const client = useAtomValue(getClient);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <PanelHint>
        The TypeScript this operation runs, stored as a <Chip>custom-code</Chip>{" "}
        extension.{" "}
        {saved
          ? "Save the resource to deploy a change."
          : "It is saved with the resource when you create it."}
      </PanelHint>

      {/* Side by side on a wide screen. Below it the panel stacks under the
          code, which gets a fixed height so the panel shows, and the two
          scroll together. */}
      <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto xl:flex-row xl:overflow-hidden">
        <div className="flex h-80 min-w-0 shrink-0 overflow-auto rounded-md border border-slate-200 xl:h-auto xl:flex-1">
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

        <div className="flex h-[40rem] shrink-0 flex-col overflow-hidden rounded-md border border-slate-200 xl:h-auto xl:w-[26rem]">
          <OperationInvocationPanel
            client={client}
            fhirVersion={R4}
            operationDefinition={operation}
            // The server runs the stored operation, so there is nothing to
            // run until it has been created.
            disabledReason={
              saved ? undefined : "Create the operation to run it."
            }
            runHint={
              dirty ? (
                <span className="text-amber-700">
                  Unsaved changes: running uses the last saved version.
                </span>
              ) : (
                "Runs the saved version of this operation."
              )
            }
          />
        </div>
      </div>
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
