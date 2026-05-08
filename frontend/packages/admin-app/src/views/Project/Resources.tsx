import { useAtomValue } from "jotai";
import React from "react";
import { generatePath, useNavigate } from "react-router-dom";
import { ChevronRightIcon, HomeIcon } from "@heroicons/react/24/outline";

import { Loading, Table } from "@haste-health/components";
import { CapabilityStatementRestResource } from "@haste-health/fhir-types/r4/types";

import { getCapabilities } from "../../db/capabilities";

const DisplayResources = () => {
  const navigate = useNavigate();
  const capabilities = useAtomValue(getCapabilities);
  const resources = capabilities?.rest?.[0].resource || [];

  return (
    <div className="flex flex-1 flex-col gap-6 overflow-auto">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="mb-3 flex flex-wrap items-center gap-1 text-sm text-slate-500">
          <button
            className="inline-flex items-center gap-1 rounded px-1 py-0.5 hover:bg-slate-100 hover:text-slate-700"
            onClick={() => navigate(generatePath("/", {}))}
            type="button"
          >
            <HomeIcon className="h-4 w-4" />
            Home
          </button>
          <ChevronRightIcon className="h-4 w-4 text-slate-400" />
          <span className="rounded bg-slate-100 px-2 py-0.5 text-xs font-medium text-slate-700">
            Resources
          </span>
        </div>

        <h1 className="text-2xl font-semibold text-slate-900">
          Supported Resources
        </h1>
        <p className="mt-1 text-sm text-slate-500">
          Browse every resource type exposed by your server capability
          statement. Select a row to open and manage that resource type.
        </p>
        <div className="mt-4 inline-flex rounded-md border border-brand-100 bg-brand-50 px-3 py-1 text-sm font-medium text-brand-700">
          {resources.length} resource types available
        </div>
      </header>

      <section className="rounded-lg border border-slate-200 bg-white shadow-sm">
        <Table
          data={resources}
          onRowClick={(row) => {
            navigate(
              generatePath("/resources/:resourceType", {
                resourceType: (row as CapabilityStatementRestResource).type,
              }),
            );
          }}
          columns={[
            {
              id: "Resource Type",
              content: "Resource Type",
              selector: "$this.type",
              selectorType: "fhirpath",
            },
            {
              id: "Profile",
              content: "Profile",
              selector: "$this.profile",
              selectorType: "fhirpath",
            },
            {
              id: "Interactions Supported",
              content: "Interactions Supported",
              selector: "$this.interaction.code",
              selectorType: "fhirpath",
            },
          ]}
        />
      </section>
    </div>
  );
};

export default function ResourcesView() {
  return (
    <React.Suspense
      fallback={
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1 ">Loading...</div>
        </div>
      }
    >
      <DisplayResources />
    </React.Suspense>
  );
}
