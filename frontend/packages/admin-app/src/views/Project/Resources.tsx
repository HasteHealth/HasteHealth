import { useAtomValue } from "jotai";
import React from "react";
import { generatePath, useNavigate } from "react-router-dom";

import { Loading, Table } from "@oxidized-health/components";
import { CapabilityStatementRestResource } from "@oxidized-health/fhir-types/r4/types";

import { getCapabilities } from "../../db/capabilities";

const DisplayResources = () => {
  const navigate = useNavigate();
  const capabilities = useAtomValue(getCapabilities);

  return (
    <div className="flex flex-col flex-1 overflow-auto">
      <h2 className="text-2xl font-semibold mb-2">Supported Resources</h2>
      <Table
        data={capabilities?.rest?.[0].resource || []}
        onRowClick={(row) => {
          navigate(
            generatePath("/resources/:resourceType", {
              resourceType: (row as CapabilityStatementRestResource).type,
            })
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
