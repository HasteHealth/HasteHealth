import { json } from "@codemirror/lang-json";
import { ChevronDownIcon } from "@heroicons/react/24/outline";
import { basicSetup } from "codemirror";
import { useAtomValue } from "jotai";
import React, { useEffect, useMemo, useState } from "react";
import { useParams } from "react-router-dom";

import {
  Button,
  CodeMirror,
  DropDownMenu,
  FHIRGenerativeForm,
  MergeViewer,
  Modal,
  Table,
  Tabs,
} from "@haste-health/components";
import {
  BundleEntry,
  Resource,
  ResourceType,
  StructureDefinition,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../db/client";

const extensions = [basicSetup, json()];

function JSONEditor({
  value,
  setValue,
}: {
  value: string;
  setValue: (value: string) => void;
}) {
  return (
    <div className="flex flex-1 border overflow-auto">
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
  );
}

function ResourceHistory() {
  const client = useAtomValue(getClient);
  const { resourceType, id } = useParams();
  const [loading, setLoading] = useState(true);
  const [history, setHistory] = useState<BundleEntry[]>([]);
  const [diff, setDiff] = useState<[BundleEntry, BundleEntry] | undefined>(
    undefined
  );

  useEffect(() => {
    setLoading(true);
    client
      .history_instance({}, R4, resourceType as ResourceType, id as id)
      .then((response) => {
        setHistory(response);
        setLoading(false);
        return response;
      });
  }, [resourceType, id, setHistory]);

  return (
    <Modal
      size="x-large"
      ModalContent={() => (
        <MergeViewer
          extensions={extensions}
          oldValue={JSON.stringify(diff?.[1].resource, null, 2)}
          newValue={JSON.stringify(diff?.[0].resource, null, 2)}
        />
      )}
    >
      {(openModal) => (
        <Table
          isLoading={loading}
          data={history.map((d, i) => ({ ...d, index: i })) ?? []}
          onRowClick={(_row: unknown) => {
            const row = _row as { index: number };
            setDiff([
              history[row.index],
              history[row.index + 1] ?? history[row.index],
            ]);

            openModal(true);
          }}
          columns={[
            {
              id: "interaction",
              content: "Interaction",
              selector: "$this.request.method",
              selectorType: "fhirpath",
            },
            {
              id: "Version",
              content: "Version",
              selector: "$this.resource.meta.versionId",
              selectorType: "fhirpath",
            },
            {
              id: "Author",
              content: "Author",
              selector:
                "$this.resource.meta.extension.where(url='https://haste-health.app/author').valueReference.reference",
              selectorType: "fhirpath",
            },
            {
              id: "Updated at",
              content: "Updated at",
              selector: "$this.resource.meta.lastUpdated",
              selectorType: "fhirpath",
            },
          ]}
        />
      )}
    </Modal>
  );
}

export interface AdditionalContent {
  id: id;
  resourceType: ResourceType;
  resource: Resource | undefined;
  structureDefinition: StructureDefinition | undefined;
  onChange?: React.Dispatch<React.SetStateAction<Resource | undefined>>;
  actions: Parameters<typeof DropDownMenu>[0]["links"];
  leftTabs?: Parameters<typeof Tabs>[0]["tabs"];
  rightTabs?: Parameters<typeof Tabs>[0]["tabs"];
}
export default function ResourceEditorComponent({
  id,
  actions,
  resource,
  structureDefinition,
  onChange,
  leftTabs: leftSide = [],
  rightTabs: rightSide = [],
}: AdditionalContent) {
  const client = useAtomValue(getClient);
  const setValue = useMemo(
    () => (getResource: (r: Resource) => Resource) => {
      if (onChange) {
        onChange((resource) => {
          const newResource = getResource(
            resource
              ? resource
              : ({
                  resourceType: structureDefinition?.type,
                } as Resource)
          );
          return newResource;
        });
      }
    },
    [structureDefinition, onChange]
  );

  return (
    <Tabs
      tabs={[
        ...leftSide,
        ...[
          {
            id: "editor",
            title: "Editor",
            content: structureDefinition && (
              <FHIRGenerativeForm
                fhirVersion={R4}
                value={resource}
                structureDefinition={structureDefinition}
                setValue={setValue}
                client={client}
              />
            ),
          },
          {
            id: "json",
            title: "JSON",
            content: (
              <JSONEditor
                value={JSON.stringify(resource, null, 2)}
                setValue={(v) => {
                  try {
                    const resource = JSON.parse(v);
                    if (onChange) onChange(resource);
                  } catch (e) {
                    console.error(e);
                    return;
                  }
                }}
              />
            ),
          },
        ],
        ...(id !== "new"
          ? [
              {
                id: "history",
                title: "History",
                content: <ResourceHistory />,
              },
            ]
          : []),
        ...rightSide,
      ]}
      rightSide={
        <DropDownMenu links={actions}>
          <Button buttonType="secondary" buttonSize="small" onClick={() => {}}>
            <span className="flex items-center">
              <span>Actions</span> <ChevronDownIcon className="ml-1 w-3 h-3" />
            </span>
          </Button>
        </DropDownMenu>
      }
    />
  );
}
