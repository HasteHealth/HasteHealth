import { indentLess, insertTab } from "@codemirror/commands";
import { javascript } from "@codemirror/lang-javascript";
import { json } from "@codemirror/lang-json";
import { keymap } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";

import {
  Button,
  CodeMirror,
  OperationInvocationPanel,
  Table,
  Tabs,
  Toaster,
} from "@haste-health/components";
import {
  AuditEvent,
  OperationDefinition,
  ResourceType,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import { HasteHealthDeployOperation } from "@haste-health/generated-ops/lib/r4/ops";

import ResourceEditorComponent, {
  AdditionalContent,
} from "../../components/ResourceEditor";
import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

const extensions = [
  basicSetup,
  javascript({ typescript: true }),
  keymap.of([
    {
      key: "Tab",
      preventDefault: true,
      run: insertTab,
    },
    {
      key: "Shift-Tab",
      preventDefault: true,
      run: indentLess,
    },
    {
      key: "Mod-s", // Mod is Ctrl on Windows/Linux and Cmd on macOS
      run: () => {
        // Returning true prevents the default browser save behavior
        return true;
      },
    },
  ]),
];

function getOperationCode(operation: OperationDefinition | undefined): string {
  const code: string =
    operation?.extension?.find(
      (e) => e.url === "https://haste.health/Extension/custom-code",
    )?.valueString ?? "";
  return code;
}

const DeployModal = ({
  operation,
  setOpen,
}: {
  operation: OperationDefinition | undefined;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
  const client = useAtomValue(getClient);
  const [environment, setEnvironment] = useState("[]");
  const [output, setOutput] = useState<unknown | undefined>(undefined);

  return (
    <div>
      <Tabs
        tabs={[
          {
            id: "environment",
            title: "Environment Variables",
            content: (
              <div className="flex flex-col h-56 w-full">
                <div className="flex flex-1 border overflow-auto">
                  <CodeMirror
                    extensions={[basicSetup, json()]}
                    value={environment}
                    theme={{
                      "&": {
                        height: "100%",
                        width: "100%",
                      },
                    }}
                    onChange={(value) => {
                      setEnvironment(value);
                    }}
                  />
                </div>
              </div>
            ),
          },
          {
            id: "output",
            title: "Output",
            content: (
              <div className="flex flex-col h-56 w-full">
                <div className="flex flex-1 border  overflow-auto">
                  <CodeMirror
                    readOnly
                    extensions={[basicSetup, json()]}
                    value={JSON.stringify(output, null, 2)}
                    theme={{
                      "&": {
                        height: "100%",
                        width: "100%",
                      },
                    }}
                  />
                </div>
              </div>
            ),
          },
        ]}
      />
      <div className="mt-1 flex justify-end px-2">
        <Button
          className="mr-1"
          buttonType="primary"
          onClick={(e) => {
            e.preventDefault();
            try {
              if (!operation) {
                throw new Error("Must have operation to trigger invocation");
              }
              const invocation = client.invoke_instance(
                HasteHealthDeployOperation.Op,
                {},
                R4,
                "OperationDefinition",
                operation.id as id,
                {
                  code: getOperationCode(operation),
                  environment: JSON.parse(environment),
                },
              );

              Toaster.promise(invocation, {
                loading: "Invocation",
                success: (success) => {
                  setOutput(success);
                  return `Invocation succeeded`;
                },
                error: (error) => {
                  return getErrorMessage(error);
                },
              });
            } catch (e) {
              Toaster.error(`${e}`);
            }
          }}
        >
          Send
        </Button>
        <Button
          buttonType="secondary"
          onClick={(e) => {
            e.preventDefault();
            setOpen(false);
          }}
        >
          Cancel
        </Button>
      </div>
    </div>
  );
};

function OperationCodeEditor({
  operation,
  saved,
  value,
  setValue,
}: {
  operation: OperationDefinition | undefined;
  /** False while the operation is still being created. */
  saved: boolean;
  value: string;
  setValue: (value: string) => void;
}) {
  const client = useAtomValue(getClient);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="mb-2 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
        Edit the operation source code, then run it from the panel on the right:
        choose where it runs, fill in its input parameters and read its output.
      </div>
      {/* A fixed height, so the code scrolls inside its editor and the run
          panel keeps its size however long the code is. */}
      <div className="flex h-[calc(100vh-10rem)] min-h-[36rem] gap-3">
        <div className="flex min-w-0 flex-1 overflow-hidden rounded-md border border-slate-200">
          <CodeMirror
            extensions={extensions}
            value={value}
            theme={{
              "&": {
                height: "100%",
                width: "100%",
              },
            }}
            onChange={(value) => {
              setValue(value);
            }}
          />
        </div>
        {operation && (
          <div className="flex w-[28rem] shrink-0 flex-col overflow-hidden rounded-md border border-slate-200">
            <OperationInvocationPanel
              client={client}
              fhirVersion={R4}
              operationDefinition={operation}
              // The server runs the stored operation, so there is nothing
              // to run until it has been created.
              disabledReason={
                saved ? undefined : "Create the operation to run it."
              }
              runHint="Runs the saved operation: save first so your latest code and parameters are used."
            />
          </div>
        )}
      </div>
    </div>
  );
}

function OperationAuditEvents({ operationId }: { operationId: string }) {
  const client = useAtomValue(getClient);
  const [loading, setLoading] = useState(true);
  const [auditEvents, setAuditEvents] = useState<AuditEvent[]>([]);

  useEffect(() => {
    setLoading(true);
    client
      .search_type({}, R4, "AuditEvent", [
        { name: "entity", value: [operationId] },
      ])
      .then((response) => {
        setAuditEvents(response.resources);
        setLoading(false);
        return response;
      });
  }, [operationId, setAuditEvents]);

  return (
    <div className="space-y-2">
      <div className="rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
        Recent execution events for this operation.
      </div>
      <Table
        isLoading={loading}
        data={auditEvents || []}
        columns={[
          {
            id: "Outcome",
            content: "Outcome",
            selector: "$this.outcome",
            selectorType: "fhirpath",
          },
          {
            id: "Agent",
            content: "Agent",
            selector: "$this.agent.name",
            selectorType: "fhirpath",
          },
          {
            id: "Description",
            content: "Description",
            selector: "$this.outcomeDesc",
            selectorType: "fhirpath",
          },
        ]}
      />
    </div>
  );
}

interface OperationEditorProps extends AdditionalContent {
  resource: OperationDefinition | undefined;
  onChange: NonNullable<AdditionalContent["onChange"]>;
}

const DEFAULT_CODE = `
interface Context {
  request: {
    id?: string;
    resource?: string;
    parameters: unknown;
  }
}

export default async function(context: Context) {
  const sd = await fhir.read("StructureDefinition", context.request.parameters.parameter.filter(p => p.name === "id")[0].valueString);

  return {
      resourceType: 'Parameters',
      parameter: [
          {
              name: 'sd',
              resource: sd
          }
      ]
  };
}
`;

export default function OperationDefinitionView({
  id,
  resourceType,
  resource,
  actions,
  structureDefinition,
  onChange,
}: OperationEditorProps) {
  const code: string = getOperationCode(resource);

  useEffect(() => {
    if (id === "new" && resource === undefined) {
      onChange({
        resourceType: "OperationDefinition",
        extension: [
          {
            extension: [
              {
                url: "https://haste.health/Extension/custom-code-type",
                valueString: "text/typescript",
              },
            ],
            url: "https://haste.health/Extension/custom-code",
            valueString: DEFAULT_CODE,
          },
        ],
        name: "New Operation",
        status: "draft",
        kind: "operation",
        code: "new",
        system: true,
        type: false,
        instance: false,
        parameter: [
          {
            name: "sd",
            use: "out",
            min: 1,
            max: "1",
            type: "StructureDefinition",
          },
          {
            name: "id",
            use: "in",
            min: 1,
            max: "1",
            type: "string",
          },
        ],
      } as OperationDefinition);
    }
  }, [id, resource]);

  return (
    <ResourceEditorComponent
      id={id as id}
      actions={actions}
      structureDefinition={structureDefinition}
      resourceType={resourceType as ResourceType}
      resource={resource}
      onChange={onChange}
      rightTabs={[
        {
          id: "logs",
          title: "Logs",
          content: <OperationAuditEvents operationId={id as string} />,
        },
      ]}
      leftTabs={[
        {
          id: "code",
          title: "Code",
          content: (
            <OperationCodeEditor
              value={code}
              operation={resource}
              saved={id !== "new"}
              setValue={(v: string) =>
                onChange({
                  ...resource,
                  resourceType: "OperationDefinition",
                  extension: [
                    ...(resource?.extension?.filter(
                      (e) =>
                        e.url !== "https://haste.health/Extension/custom-code",
                    ) ?? []),
                    {
                      extension: [
                        {
                          url: "https://haste.health/Extension/custom-code-type",
                          valueString: "text/typescript",
                        },
                      ],
                      url: "https://haste.health/Extension/custom-code",
                      valueString: v,
                    },
                  ],
                } as OperationDefinition)
              }
            />
          ),
        },
      ]}
    />
  );
}
