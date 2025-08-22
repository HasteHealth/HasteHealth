import { json } from "@codemirror/lang-json";
import { basicSetup } from "codemirror";
import { useAtomValue } from "jotai";
import { useState } from "react";

import {
  Button,
  CodeMirror,
  FHIRCodeEditable,
  FHIRReferenceEditable,
  FHIRUriEditable,
  Modal,
  Tabs,
  Toaster,
} from "@oxidized-health/components";
import {
  AccessPolicyV2,
  Reference,
  ResourceType,
  code,
  id,
  uri,
} from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/versions";
import { OxidizedHealthEvaluatePolicy } from "@oxidized-health/generated-ops/lib/r4/ops";

import ResourceEditorComponent, {
  AdditionalContent,
} from "../../components/ResourceEditor";
import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

interface AccessPolicyV2EditorProps extends AdditionalContent {
  resource: AccessPolicyV2 | undefined;
  onChange: NonNullable<AdditionalContent["onChange"]>;
}

const AccessPolicyInvoke = ({
  accessPolicy,
  setOpen,
}: {
  accessPolicy: AccessPolicyV2 | undefined;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
  const client = useAtomValue(getClient);
  const [output, setOutput] = useState<unknown | undefined>(undefined);
  const [userReference, setUserReference] = useState<Reference | undefined>();
  const [requestType, setRequestType] = useState<code | undefined>();
  const [requestURL, setRequestURL] = useState<uri | undefined>();
  const [body, setBody] = useState<string | undefined>();

  return (
    <div>
      <Tabs
        tabs={[
          {
            id: "input",
            title: "Input",
            content: (
              <div className="flex flex-col h-56 w-full">
                <div className="flex flex-col flex-1 overflow-auto flex-grow space-y-2 px-1">
                  <div>
                    <FHIRReferenceEditable
                      label="User"
                      resourceTypesAllowed={["Membership"]}
                      fhirVersion={R4}
                      client={client}
                      value={userReference}
                      onChange={(ref) => {
                        if (ref) {
                          setUserReference(ref);
                        }
                      }}
                    />
                  </div>
                  <div className="flex flex-col flex-1 space-y-1">
                    <FHIRCodeEditable
                      label="Request"
                      client={client}
                      fhirVersion={R4}
                      system={
                        "http://hl7.org/fhir/ValueSet/http-operations" as uri
                      }
                      value={requestType}
                      onChange={(code) => setRequestType(code)}
                    />
                    <FHIRUriEditable
                      label="URL"
                      value={requestURL}
                      onChange={(url) => setRequestURL(url)}
                    />
                    <div>
                      <label>Body</label>
                      <CodeMirror
                        extensions={[basicSetup, json()]}
                        value={body}
                        theme={{
                          "&": {
                            height: "100%",
                            width: "100%",
                          },
                        }}
                        onChange={(value) => {
                          setBody(value);
                        }}
                      />
                    </div>
                  </div>
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
              const invocation = client.invoke_instance(
                OxidizedHealthEvaluatePolicy.Op,
                {},
                R4,
                "AccessPolicyV2",
                accessPolicy?.id as id,
                {
                  user: userReference,
                  request: {
                    resourceType: "Bundle",
                    type: "batch" as code,
                    entry: [
                      {
                        request: {
                          method: requestType?.toUpperCase() as code,
                          url: requestURL as uri,
                        },
                        resource: (() => {
                          try {
                            const resource = JSON.parse(body as string);
                            return resource;
                          } catch {
                            return undefined;
                          }
                        })(),
                      },
                    ],
                  },
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

export default function AccessPolicyView({
  id,
  resourceType,
  resource,
  actions,
  structureDefinition,
  onChange,
}: AccessPolicyV2EditorProps) {
  return (
    <div className="flex justify-center items-center py-2 px-1">
      <Modal
        modalTitle="Evaluate Policy"
        ModalContent={(setOpen) => (
          <AccessPolicyInvoke accessPolicy={resource} setOpen={setOpen} />
        )}
      >
        {(setOpen) => (
          <ResourceEditorComponent
            id={id as id}
            actions={[
              {
                key: "EvaluatePolicy",
                label: "Test policy",
                onClick: () => {
                  setOpen(true);
                },
              },
              ...actions,
            ]}
            structureDefinition={structureDefinition}
            resourceType={resourceType as ResourceType}
            resource={resource}
            onChange={onChange}
          />
        )}
      </Modal>
    </div>
  );
}
