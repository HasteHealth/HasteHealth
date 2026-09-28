import { json } from "@codemirror/lang-json";
import { ChevronDownIcon } from "@heroicons/react/24/outline";
import { basicSetup } from "codemirror";
import { useAtomValue } from "jotai";
import React, { useEffect, useState } from "react";
import { useParams } from "react-router";

import {
  Button,
  JSONResourceEditor,
  DropDownMenu,
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
const HISTORY_PAGE_SIZE = 25;

function ResourceHistory() {
  const client = useAtomValue(getClient);
  const { resourceType, id } = useParams();
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [history, setHistory] = useState<BundleEntry[]>([]);
  const [diff, setDiff] = useState<[BundleEntry, BundleEntry] | undefined>(
    undefined,
  );

  const loadHistory = (offset = 0) => {
    if (offset === 0) {
      setLoading(true);
    } else {
      setLoadingMore(true);
    }

    client
      .history_instance(
        {},
        R4,
        resourceType as ResourceType,
        id as id,
        `_offset=${offset}&_count=${HISTORY_PAGE_SIZE}`,
      )
      .then((response) => {
        setHistory((current) =>
          offset === 0 ? response : [...current, ...response],
        );
        // Assume that if we get full page their are more results.
        setHasMore(response.length === HISTORY_PAGE_SIZE);
      })
      .finally(() => {
        if (offset === 0) {
          setLoading(false);
        } else {
          setLoadingMore(false);
        }
      });
  };

  useEffect(() => {
    loadHistory(0);
  }, [resourceType, id, client]);

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
        <div>
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
                id: "resource",
                content: "Resource",
                selector: "$this.resource.type().type",
                selectorType: "fhirpath",
              },
              {
                id: "id",
                content: "ID",
                selector: "$this.resource.id",
                selectorType: "fhirpath",
              },
              {
                id: "interaction",
                content: "Interaction",
                selector: "$this.request.method",
                selectorType: "fhirpath",
              },
              {
                id: "version",
                content: "Version",
                selector: "$this.resource.meta.versionId",
                selectorType: "fhirpath",
              },
              {
                id: "Author",
                content: "Author",
                selector:
                  "$this.resource.meta.extension.where(url='https://haste.health/author').value.reference",
                selectorType: "fhirpath",
              },
              {
                id: "updated-at",
                content: "Updated at",
                selector: "$this.resource.meta.lastUpdated",
                selectorType: "fhirpath",
              },
            ]}
          />

          {hasMore && (
            <div className="mt-3 flex justify-center">
              <Button
                buttonType="secondary"
                buttonSize="small"
                onClick={() => loadHistory(history.length)}
                disabled={loadingMore}
              >
                {loadingMore ? "Loading..." : "Load more"}
              </Button>
            </div>
          )}
        </div>
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
  return (
    <Tabs
      tabs={[
        // JSON comes first: this is a developer tool, and the resource as the
        // server stores it is the thing to edit. Resource specific builders,
        // where they exist, are passed in as `leftTabs`/`rightTabs`.
        {
          id: "json",
          title: "JSON",
          content: (
            <JSONResourceEditor
              resource={resource}
              onChange={(next: Resource) => onChange?.(next)}
            />
          ),
        },
        ...leftSide,
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
