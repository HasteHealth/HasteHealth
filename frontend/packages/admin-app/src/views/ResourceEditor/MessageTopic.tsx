import { json } from "@codemirror/lang-json";
import { basicSetup } from "codemirror";
import { useAtomValue } from "jotai";
import { useState } from "react";

import {
  Button,
  CodeMirror,
  Modal,
  Tabs,
  Toaster,
} from "@oxidized-health/components";
import {
  MessageTopic,
  ResourceType,
  id,
} from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/versions";
import { OxidizedHealthMessagePost } from "@oxidized-health/generated-ops/lib/r4/ops";

import ResourceEditorComponent, {
  AdditionalContent,
} from "../../components/ResourceEditor";
import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

interface MessageTopicEditorProps extends AdditionalContent {
  resource: MessageTopic | undefined;
  onChange: NonNullable<AdditionalContent["onChange"]>;
}

const SendMessageTopicModal = ({
  messageTopic,
  setOpen,
}: {
  messageTopic: MessageTopic | undefined;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
  const client = useAtomValue(getClient);
  const [parameters, setParameters] = useState("{}");
  const [output, setOutput] = useState<unknown | undefined>(undefined);

  return (
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

              <div className="mt-1 flex justify-end">
                <Button
                  className="mr-1"
                  buttonType="primary"
                  onClick={(e) => {
                    e.preventDefault();
                    try {
                      if (!messageTopic) {
                        Toaster.error("MessageTopic is not yet created.");
                        return;
                      }
                      const invocation = client.invoke_instance(
                        OxidizedHealthMessagePost.Op,
                        {},
                        R4,
                        "MessageTopic",
                        messageTopic.id as id,
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
  );
};

export default function MessageTopicView({
  id,
  resourceType,
  resource,
  actions,
  structureDefinition,
  onChange,
}: MessageTopicEditorProps) {
  return (
    <div className="flex justify-center items-center py-2 px-1">
      <Modal
        modalTitle="Send Message"
        ModalContent={(setOpen) => (
          <SendMessageTopicModal messageTopic={resource} setOpen={setOpen} />
        )}
      >
        {(setOpen) => (
          <ResourceEditorComponent
            id={id as id}
            actions={[
              {
                key: "SendMessage",
                label: "Send Message",
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
