import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";
import { ChevronRightIcon, HomeIcon } from "@heroicons/react/24/outline";

import { Toaster } from "@haste-health/components";
import {
  AccessPolicyV2,
  Bundle,
  IdentityProvider,
  OperationDefinition,
  Questionnaire,
  Resource,
  ResourceType,
  StructureDefinition,
  Subscription,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import ResourceEditorComponent from "../../components/ResourceEditor";
import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";
import AccessPolicyView from "./AccessPolicy";
import IdentityProviderView from "./IdentityProvider";
import OperationDefinitionView from "./OperationDefinition";
import SubscriptionView from "./Subscription";
import QuestionnaireView from "./Questionnaire";

function ResourceEditorTabs({
  onStructureDefinitionChange,
}: Readonly<{
  onStructureDefinitionChange: (sd: StructureDefinition | undefined) => void;
}>) {
  const client = useAtomValue(getClient);
  const [resource, setResource] = useState<Resource | undefined>(undefined);
  const [structureDefinition, setStructureDefinition] = useState<
    StructureDefinition | undefined
  >(undefined);
  const navigate = useNavigate();

  const { resourceType, id } = useParams();

  const actions = [
    {
      key: "Update",
      label: id === "new" ? "Create" : "Update",
      onClick: () => {
        try {
          const editPromise = (
            id === "new"
              ? client.create({}, R4, {
                  ...resource,
                  resourceType,
                } as Resource)
              : client.update(
                  {},
                  R4,
                  resourceType as ResourceType,
                  id as id,
                  {
                    ...resource,
                    resourceType,
                    id,
                  } as Resource,
                )
          ).then((response) => {
            setResource(response);
            return response;
          });

          Toaster.promise(editPromise, {
            loading: id === "new" ? "Creating Resource" : "Updating Resource",
            success: (success) =>
              `Updated ${(success as Resource).resourceType}`,
            error: (error) => {
              return getErrorMessage(error);
            },
          }).then((value) =>
            navigate(
              generatePath("/resources/:resourceType/:id", {
                resourceType: resourceType as string,
                id: (value as Resource).id as string,
              }),
              {
                replace: true,
              },
            ),
          );
        } catch (e) {
          Toaster.error(`${e}`);
        }
      },
    },
    ...(id !== "new"
      ? [
          {
            key: "Delete",
            className: "!text-red-600 hover:bg-red-600 hover:!text-white",
            label: "Delete",
            onClick: () => {
              const deletePromise = client.delete_instance(
                {},
                R4,
                resourceType as ResourceType,
                id as id,
              );
              Toaster.promise(deletePromise, {
                loading: "Deleting Resource",
                success: () => `Deleted ${resourceType}`,
                error: (error) => {
                  return getErrorMessage(error);
                },
              }).then(() =>
                navigate(
                  generatePath("/resources/:resourceType", {
                    resourceType: resourceType as string,
                  }),
                ),
              );
            },
          },
        ]
      : []),
  ];

  useEffect(() => {
    if (!resourceType || !id) {
      return;
    }

    const entries = [
      ...(id === "new"
        ? []
        : [
            {
              request: {
                method: "GET",
                url: `${resourceType}/${id}`,
              },
            },
          ]),
      {
        request: {
          method: "GET",
          url: `StructureDefinition/${resourceType}`,
        },
      },
    ];

    client
      .batch({}, R4, {
        type: "batch",
        resourceType: "Bundle",
        entry: entries,
      } as Bundle)
      .then((response) => {
        if (id === "new") {
          setResource(undefined);
          const sd = response.entry?.[0]?.resource as StructureDefinition;
          setStructureDefinition(sd);
          onStructureDefinitionChange(sd);
          return;
        }

        setResource(response.entry?.[0]?.resource);
        const sd = response.entry?.[1]?.resource as StructureDefinition;
        setStructureDefinition(sd);
        onStructureDefinitionChange(sd);
      });
  }, [resourceType, id, client, onStructureDefinitionChange]);

  switch (resourceType) {
    case "OperationDefinition":
      return (
        <OperationDefinitionView
          id={id as id}
          resourceType={resourceType as ResourceType}
          actions={actions}
          resource={resource as OperationDefinition}
          structureDefinition={structureDefinition}
          onChange={setResource}
        />
      );
    case "Subscription":
      return (
        <SubscriptionView
          id={id as id}
          resourceType={resourceType as ResourceType}
          actions={actions}
          resource={resource as Subscription}
          structureDefinition={structureDefinition}
          onChange={setResource}
        />
      );
    case "AccessPolicyV2":
      return (
        <AccessPolicyView
          id={id as id}
          resourceType={resourceType as ResourceType}
          actions={actions}
          resource={resource as AccessPolicyV2}
          structureDefinition={structureDefinition}
          onChange={setResource}
        />
      );
    case "IdentityProvider": {
      return (
        <IdentityProviderView
          id={id as id}
          resourceType={resourceType as ResourceType}
          actions={actions}
          resource={resource as IdentityProvider}
          structureDefinition={structureDefinition}
          onChange={setResource}
        />
      );
    }
    case "Questionnaire":
      return (
        <QuestionnaireView
          id={id as id}
          resourceType={resourceType as ResourceType}
          actions={actions}
          resource={resource as Questionnaire}
          structureDefinition={structureDefinition}
          onChange={setResource}
        />
      );
    default:
      return (
        <ResourceEditorComponent
          id={id as id}
          resourceType={resourceType as ResourceType}
          actions={actions}
          resource={resource}
          structureDefinition={structureDefinition}
          onChange={setResource}
        />
      );
  }
}

export default function ResourceEditor() {
  const { resourceType, id } = useParams();
  const navigate = useNavigate();
  const [structureDefinition, setStructureDefinition] = useState<
    StructureDefinition | undefined
  >(undefined);

  const displayResourceType = resourceType || "Resource";
  const displayId = id || "unknown";

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
            className="rounded px-1 py-0.5 hover:bg-slate-100 hover:text-slate-700"
            onClick={() =>
              navigate(
                generatePath("/resources/:resourceType", {
                  resourceType: displayResourceType,
                }),
              )
            }
            type="button"
          >
            {displayResourceType}
          </button>
          <ChevronRightIcon className="h-4 w-4 text-slate-400" />
          <span className="rounded bg-slate-100 px-2 py-0.5 font-mono text-xs text-slate-700">
            {displayId}
          </span>
        </div>

        <h1 className="text-2xl font-semibold text-slate-900">
          {displayResourceType} Editor
        </h1>
        <p className="mt-1 text-sm text-slate-500">
          {displayId === "new"
            ? `Create a new ${displayResourceType} and save it to your project.`
            : `Review and update ${displayResourceType}/${displayId}.`}
        </p>
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <span className="rounded-md border border-slate-200 bg-slate-50 px-2 py-1 text-xs font-medium text-slate-700">
            Status: {structureDefinition?.status ?? "unknown"}
          </span>
          <span className="rounded-md border border-slate-200 bg-slate-50 px-2 py-1 text-xs font-medium text-slate-700">
            Version: {structureDefinition?.version ?? "n/a"}
          </span>
        </div>
      </header>

      <section className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
        <ResourceEditorTabs
          onStructureDefinitionChange={setStructureDefinition}
        />
      </section>
    </div>
  );
}
