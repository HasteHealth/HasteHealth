import { PlusIcon } from "@heroicons/react/24/outline";
import { FHIRGenerativeSearchTable } from "@haste-health/components";
import {
  AllResourceTypes,
  R4,
  Resource,
  ResourceType,
} from "@haste-health/fhir-types/versions";
import { useAtomValue } from "jotai";
import { useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";
import { getClient } from "../../db/client";

const RESOURCE_DESCRIPTIONS: Record<string, string> = {
  User: "Manage user accounts and their properties.",
  IdentityProvider:
    "Configure external identity providers for SMART / OIDC authentication.",
};

function pluralizeResourceType(resourceType: string) {
  if (resourceType.endsWith("y")) {
    return `${resourceType.slice(0, -1)}ies`;
  }
  return `${resourceType}s`;
}

export default function ResourceTypes() {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const params = useParams();
  const [refresh, setRefresh] = useState<(() => void) | undefined>(undefined);

  const resourceType = params.resourceType ?? "";
  const title = resourceType
    ? pluralizeResourceType(resourceType)
    : "Resources";
  const description =
    RESOURCE_DESCRIPTIONS[resourceType] ??
    `Browse and manage ${resourceType} resources.`;

  return (
    <div className="flex w-full flex-col gap-6">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <h1 className="text-2xl font-semibold text-slate-900">{title}</h1>
            <p className="text-sm text-slate-500">{description}</p>
          </div>
          <button
            className="inline-flex shrink-0 items-center gap-2 rounded-md bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-500"
            onClick={() =>
              navigate(
                generatePath("/resources/:resourceType/:id", {
                  resourceType,
                  id: "new",
                }),
              )
            }
          >
            <PlusIcon className="h-4 w-4" />
            New {resourceType}
          </button>
        </div>
      </header>

      <section className="rounded-lg border border-slate-200 bg-white p-3 shadow-sm">
        <div className="mb-3 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
          Select a row to open the full editor for that resource.
        </div>
        <FHIRGenerativeSearchTable
          refresh={(refreshFnc) => {
            if (!refresh) {
              setRefresh(() => refreshFnc);
            }
          }}
          onRowClick={(row) => {
            navigate(
              generatePath("/resources/:resourceType/:id", {
                resourceType: (row as Resource<R4, AllResourceTypes>)
                  .resourceType,
                id: (row as Resource<R4, AllResourceTypes>).id as string,
              }),
            );
          }}
          client={client}
          fhirVersion={R4}
          resourceType={resourceType as ResourceType<R4>}
        />
      </section>
    </div>
  );
}
