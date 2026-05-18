import { ArrowPathIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";

import { Button, Table, Toaster } from "@haste-health/components";
import { BundleEntry, id } from "@haste-health/fhir-types/r4/types";
import { R4, ResourceType } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

const HISTORY_PAGE_SIZE = 25;

export default function TypeHistory() {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const { resourceType } = useParams();
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [history, setHistory] = useState<BundleEntry[]>([]);

  const loadHistory = (offset = 0) => {
    if (!resourceType) {
      setHistory([]);
      setHasMore(false);
      setLoading(false);
      return;
    }

    if (offset === 0) {
      setLoading(true);
    } else {
      setLoadingMore(true);
    }

    client
      .history_type(
        {},
        R4,
        resourceType as ResourceType<R4>,
        `_offset=${offset}&_count=${HISTORY_PAGE_SIZE}`,
      )
      .then((response) => {
        setHistory((current) =>
          offset === 0 ? response : [...current, ...response],
        );
        setHasMore(response.length === HISTORY_PAGE_SIZE);
      })
      .catch((error) => {
        Toaster.error(getErrorMessage(error));
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
  }, [client, resourceType]);

  return (
    <div className="flex w-full flex-1 flex-col gap-6 overflow-auto text-slate-700">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <h1 className="text-2xl font-semibold text-slate-900">
              {resourceType} History
            </h1>
            <p className="text-sm text-slate-500">
              Timeline of all changes for the {resourceType} resource type.
            </p>
          </div>
          <Button
            buttonType="secondary"
            buttonSize="small"
            className="font-medium"
            onClick={() => loadHistory(0)}
          >
            <span className="flex items-center gap-1">
              <ArrowPathIcon className="h-4 w-4" />
              Refresh
            </span>
          </Button>
        </div>
      </header>

      <section className="rounded-lg border border-slate-200 bg-white p-3 shadow-sm">
        <div className="mb-3 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
          Click any row to compare that revision with the previous one.
        </div>

        <Table
          isLoading={loading}
          onRowClick={(_row: unknown) => {
            const entry = _row as BundleEntry;

            navigate(
              generatePath("/resources/:resourceType/:id/_history/:versionId", {
                resourceType: entry.resource?.resourceType as ResourceType<R4>,
                id: entry.resource?.id as id,
                versionId: entry.resource?.meta?.versionId as id,
              }),
            );
          }}
          data={history.map((entry, index) => ({ ...entry, index }))}
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
      </section>
    </div>
  );
}
