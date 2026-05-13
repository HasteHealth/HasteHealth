import { ArrowPathIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";

import { Button, Table, Toaster } from "@haste-health/components";
import { BundleEntry } from "@haste-health/fhir-types/r4/types";
import { R4, ResourceType } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

export default function TypeHistory() {
  const client = useAtomValue(getClient);
  const { resourceType } = useParams();
  const [loading, setLoading] = useState(true);
  const [history, setHistory] = useState<BundleEntry[]>([]);

  const loadHistory = () => {
    if (!resourceType) {
      setHistory([]);
      setLoading(false);
      return;
    }

    setLoading(true);
    client
      .history_type({}, R4, resourceType as ResourceType<R4>)
      .then((response) => {
        setHistory(response);
      })
      .catch((error) => {
        Toaster.error(getErrorMessage(error));
      })
      .finally(() => {
        setLoading(false);
      });
  };

  useEffect(() => {
    loadHistory();
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
            onClick={loadHistory}
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
      </section>
    </div>
  );
}
