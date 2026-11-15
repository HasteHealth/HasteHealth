import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router";
import {
  BookOpenIcon,
  ChevronRightIcon,
  HomeIcon,
} from "@heroicons/react/24/outline";

import { Button, Toaster } from "@haste-health/components";
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
  ViewDefinition,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import ResourceEditorComponent from "../../components/ResourceEditor";
import { getClient } from "../../db/client";
import {
  fhirResourceDocsUrl,
  getErrorMessage,
  resourceInstancePath,
  resourceListPath,
} from "../../utilities";
import AccessPolicyView from "./AccessPolicy";
import IdentityProviderView from "./IdentityProvider";
import OperationDefinitionView from "./OperationDefinition";
import SubscriptionView from "./Subscription";
import QuestionnaireView from "./Questionnaire";
import ViewDefinitionEditor from "./ViewDefinition";

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
              resourceInstancePath(
                resourceType as string,
                (value as Resource).id as string,
              ),
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
              }).then(() => navigate(resourceListPath(resourceType as string)));
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
    case "ViewDefinition":
      return (
        <ViewDefinitionEditor
          id={id as id}
          resourceType={resourceType as ResourceType}
          actions={actions}
          resource={resource as ViewDefinition}
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
    <div className="flex flex-1 flex-col gap-6 overflow-auto text-[var(--ci-ink)]">
      <header className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-5 shadow-sm">
        <div className="mb-3 flex flex-wrap items-center gap-1 text-xs font-mono text-[var(--ci-muted)]">
          <button
            className="inline-flex items-center gap-1 rounded-none px-1 py-0.5 hover:bg-[var(--ci-surface-raised)] hover:text-[var(--ci-ink)] transition-colors"
            onClick={() => navigate(generatePath("/", {}))}
            type="button"
          >
            <HomeIcon className="h-3.5 w-3.5" />
            Home
          </button>
          <ChevronRightIcon className="h-3.5 w-3.5 text-[var(--ci-muted)] opacity-50" />
          <button
            className="inline-flex items-center gap-1 rounded-none px-1 py-0.5 hover:bg-[var(--ci-surface-raised)] hover:text-[var(--ci-ink)] transition-colors"
            onClick={() => navigate("/")}
            type="button"
          >
            Resources
          </button>
          <ChevronRightIcon className="h-3.5 w-3.5 text-[var(--ci-muted)] opacity-50" />
          <button
            className="rounded-none px-1 py-0.5 hover:bg-[var(--ci-surface-raised)] hover:text-[var(--ci-ink)] transition-colors"
            onClick={() => navigate(resourceListPath(displayResourceType))}
            type="button"
          >
            {displayResourceType}
          </button>
          <ChevronRightIcon className="h-3.5 w-3.5 text-[var(--ci-muted)] opacity-50" />
          <span className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-2 py-0.5 font-mono text-xs text-[var(--ci-signal-ink)]">
            {displayId}
          </span>
        </div>

        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <h1 className="text-xl font-mono font-semibold text-[var(--ci-ink)]">
              {displayResourceType} Editor
            </h1>
            <p className="text-xs text-[var(--ci-muted)]">
              {displayId === "new"
                ? `Create a new ${displayResourceType} and save it to your project.`
                : `Review and update ${displayResourceType}/${displayId}.`}
            </p>
            <div className="mt-2 flex flex-wrap items-center gap-2">
              <span className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-2 py-0.5 text-[11px] font-mono text-[var(--ci-muted)]">
                Status: {structureDefinition?.status ?? "unknown"}
              </span>
              <span className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-2 py-0.5 text-[11px] font-mono text-[var(--ci-muted)]">
                Version: {structureDefinition?.version ?? "n/a"}
              </span>
            </div>
          </div>
          <Button
            className="font-mono text-xs uppercase tracking-wider"
            buttonSize="small"
            buttonType="secondary"
            onClick={() =>
              window.open(
                fhirResourceDocsUrl(displayResourceType),
                "_blank",
                "noopener,noreferrer",
              )
            }
          >
            <div className="flex items-center justify-center">
              <BookOpenIcon className="mr-1 h-3.5 w-3.5 text-[var(--ci-signal)]" />
              <span>Docs</span>
            </div>
          </Button>
        </div>
      </header>

      <section className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-4 shadow-sm">
        <ResourceEditorTabs
          onStructureDefinitionChange={setStructureDefinition}
        />
      </section>
    </div>
  );
}
