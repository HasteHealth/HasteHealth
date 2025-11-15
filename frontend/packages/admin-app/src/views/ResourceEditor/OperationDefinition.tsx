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
  Modal,
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
import { Operation } from "@haste-health/operation-execution";

import ResourceEditorComponent, {
  AdditionalContent,
} from "../../components/ResourceEditor";
import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

const extensions = [
  basicSetup,
  javascript(),
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
  ]),
];

function getOperationCode(operation: OperationDefinition | undefined): string {
  const code: string =
    operation?.extension?.find(
      (e) =>
        e.url ===
        "https://haste-health.github.io/fhir-operation-definition/operation-code"
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
                }
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
  value,
  setValue,
}: {
  operation: OperationDefinition | undefined;
  value: string;
  setValue: (value: string) => void;
}) {
  return (
    <div className="flex flex-1 flex-col overflow-auto">
      <div className="border flex-1 flex">
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
      <div className="flex justify-start py-2 space-x-4">
        <Modal
          modalTitle={`Deploy ${operation?.code}`}
          ModalContent={(setOpen) => (
            <DeployModal operation={operation} setOpen={setOpen} />
          )}
        >
          {(setOpen) => (
            <Button
              buttonType="primary"
              onClick={(e) => {
                e.preventDefault();
                setOpen(true);
              }}
            >
              Deploy
            </Button>
          )}
        </Modal>
        <Modal
          modalTitle={`Invoke ${operation?.code}`}
          ModalContent={(setOpen) => (
            <InvocationModal operation={operation} setOpen={setOpen} />
          )}
        >
          {(setOpen) => (
            <Button
              buttonType="secondary"
              onClick={(e) => {
                e.preventDefault();
                setOpen(true);
              }}
            >
              Invoke
            </Button>
          )}
        </Modal>
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
  );
}

interface OperationEditorProps extends AdditionalContent {
  resource: OperationDefinition | undefined;
  onChange: NonNullable<AdditionalContent["onChange"]>;
}

const InvocationModal = ({
  operation,
  setOpen,
}: {
  operation: OperationDefinition | undefined;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
  const client = useAtomValue(getClient);
  const [parameters, setParameters] = useState("{}");
  const [output, setOutput] = useState<unknown | undefined>(undefined);

  return (
    <div>
      <Tabs
        tabs={[
          {
            id: "input",
            title: "Input",
            content: (
              <div className="flex flex-col h-56 w-full">
                <div className="flex flex-1 border overflow-auto">
                  <CodeMirror
                    extensions={[basicSetup, json()]}
                    value={parameters}
                    theme={{
                      "&": {
                        height: "100%",
                        width: "100%",
                      },
                    }}
                    onChange={(value) => {
                      setParameters(value);
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
              const invocation = client.invoke_system(
                new Operation(operation),
                {},
                R4,
                JSON.parse(parameters)
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

export default function OperationDefinitionView({
  id,
  resourceType,
  resource,
  actions,
  structureDefinition,
  onChange,
}: OperationEditorProps) {
  const code: string = getOperationCode(resource);
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
              setValue={(v: string) =>
                onChange({
                  ...resource,
                  resourceType: "OperationDefinition",
                  extension: [
                    ...(resource?.extension?.filter(
                      (e) =>
                        e.url !==
                        "https://haste-health.github.io/fhir-operation-definition/operation-code"
                    ) || []),
                    {
                      url: "https://haste-health.github.io/fhir-operation-definition/operation-code",
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
