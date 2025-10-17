import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";

import { Toaster } from "@oxidized-health/components";
import {
  AccessPolicyV2,
  Bundle,
  IdentityProvider,
  OperationDefinition,
  Resource,
  ResourceType,
  StructureDefinition,
  Subscription,
  id,
} from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/versions";

import ResourceEditorComponent from "../../../components/ResourceEditor";
import { getClient } from "../../../db/client";
import { getErrorMessage } from "../../../utilities";
import AccessPolicyView from "./AccessPolicy";
import IdentityProviderView from "./IdentityProvider";

import OperationDefinitionView from "./OperationDefinition";
import SubscriptionView from "./Subscription";

function ResourceEditorTabs() {
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
                  resourceType, // Validate that resourceTypes align.
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
                  } as Resource
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
              }
            )
          );
        } catch (e) {
          Toaster.error(`${e}`);
        }
      },
    },
    {
      key: "Delete",
      className: "!text-red-600 hover:bg-red-600 hover:!text-white",
      label: "Delete",
      onClick: () => {
        const deletePromise = client.delete_instance(
          {},
          R4,
          resourceType as ResourceType,
          id as id
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
            })
          )
        );
      },
    },
  ];

  useEffect(() => {
    client
      .batch({}, R4, {
        type: "batch",
        resourceType: "Bundle",
        entry: [
          {
            request: {
              method: "GET",
              url: `${resourceType}/${id}`,
            },
          },
          {
            request: {
              method: "GET",
              url: `StructureDefinition/${resourceType}`,
            },
          },
        ],
      } as Bundle)
      .then((response) => {
        setResource(response.entry?.[0]?.resource);
        setStructureDefinition(
          response.entry?.[1]?.resource as StructureDefinition
        );
      });
  }, [resourceType, id]);

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
  return (
    <div className="flex flex-1 flex-col overflow-auto">
      <h3 className="text-slate-700 text-xl font-semibold mb-2">
        {resourceType} {id}
      </h3>
      <ResourceEditorTabs />
    </div>
  );
}
