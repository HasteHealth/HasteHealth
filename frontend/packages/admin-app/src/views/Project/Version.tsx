import { ChevronRightIcon, HomeIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useEffect, useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";

import { FHIRGenerativeForm, Loading } from "@haste-health/components";
import {
  Bundle,
  Resource,
  ResourceType,
  StructureDefinition,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";

function VersionContent() {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const { resourceType, id, versionId } = useParams<{
    resourceType: string;
    id: string;
    versionId: string;
  }>();

  const [resource, setResource] = useState<Resource | undefined>(undefined);
  const [structureDefinition, setStructureDefinition] = useState<
    StructureDefinition | undefined
  >(undefined);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);

    Promise.all([
      client.batch({}, R4, {
        resourceType: "Bundle",
        type: "batch",
        entry: [
          {
            request: {
              method: "GET",
              url: `${resourceType}/${id}/_history/${versionId}`,
            },
          },
          {
            request: {
              method: "GET",
              url: `StructureDefinition/${resourceType}`,
            },
          },
        ],
      } as Bundle),
    ])
      .then(([res]) => {
        setResource(res.entry?.[0].resource as Resource);
        setStructureDefinition(res.entry?.[1].resource as StructureDefinition);
      })
      .finally(() => setLoading(false));
  }, [client, resourceType, id, versionId]);

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
          <button
            className="inline-flex items-center gap-1 rounded px-1 py-0.5 hover:bg-slate-100 hover:text-slate-700"
            onClick={() => navigate(generatePath("/resources", {}))}
            type="button"
          >
            Resources
          </button>
          <ChevronRightIcon className="h-4 w-4 text-slate-400" />
          <button
            className="inline-flex items-center gap-1 rounded px-1 py-0.5 hover:bg-slate-100 hover:text-slate-700"
            onClick={() =>
              navigate(
                generatePath("/resources/:resourceType", {
                  resourceType: resourceType as ResourceType,
                }),
              )
            }
            type="button"
          >
            {resourceType}
          </button>
          <ChevronRightIcon className="h-4 w-4 text-slate-400" />
          <button
            className="inline-flex items-center gap-1 rounded px-1 py-0.5 hover:bg-slate-100 hover:text-slate-700"
            onClick={() =>
              navigate(
                generatePath("/resources/:resourceType/:id", {
                  resourceType: resourceType as ResourceType,
                  id: id as id,
                }),
              )
            }
            type="button"
          >
            {id}
          </button>
          <ChevronRightIcon className="h-4 w-4 text-slate-400" />
          <span className="rounded bg-slate-100 px-2 py-0.5 font-mono text-xs text-slate-700">
            v{versionId}
          </span>
        </div>

        <h1 className="text-2xl font-semibold text-slate-900">
          {resourceType} — Version {versionId}
        </h1>
        <p className="mt-1 text-sm text-slate-500">
          Read-only view of {resourceType}/{id} at version {versionId}.
        </p>
        <div className="mt-2">
          <span className="rounded-md border border-amber-200 bg-amber-50 px-2 py-1 text-xs font-medium text-amber-700">
            Read-only — historical version
          </span>
        </div>
      </header>

      <section className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        {loading ? (
          <div className="flex flex-1 items-center justify-center py-12 flex-col">
            <Loading />
            <div className="mt-1">Loading version...</div>
          </div>
        ) : resource && structureDefinition ? (
          <div className="relative">
            <div className="absolute inset-0 z-10 cursor-not-allowed" />
            <FHIRGenerativeForm
              fhirVersion={R4}
              value={resource}
              structureDefinition={structureDefinition}
              client={client}
            />
          </div>
        ) : (
          <div className="text-sm text-slate-500">
            Version not found or could not be loaded.
          </div>
        )}
      </section>
    </div>
  );
}

export default function VersionView() {
  return (
    <React.Suspense
      fallback={
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1">Loading...</div>
        </div>
      }
    >
      <VersionContent />
    </React.Suspense>
  );
}
