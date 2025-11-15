import { MagnifyingGlassCircleIcon } from "@heroicons/react/24/outline";
import React from "react";

import {
  Bundle,
  Reference,
  Resource,
  StructureDefinition,
} from "@haste-health/fhir-types/r4/types";
import * as r4b from "@haste-health/fhir-types/r4b/types";
import { FHIR_VERSION, ResourceType } from "@haste-health/fhir-types/versions";

import { Loading, Select } from "../../base";
import { InputContainer } from "../../base/containers";
import { Modal } from "../../base/modal";
import { FHIRGenerativeSearchTable } from "../generative";
import { FHIRGenerativeForm } from "../generative/form";
import { ClientProps, EditableProps } from "../types";

export type FHIRReferenceEditableProps = EditableProps<Reference> &
  ClientProps & {
    disabled?: boolean;
    resourceTypesAllowed?: ResourceType<FHIR_VERSION>[];
  };

const ReferenceView = ({
  fhirVersion,
  value,
  client,
}: { value: Reference | undefined } & ClientProps) => {
  const [sd, setSd] = React.useState<StructureDefinition>();
  const [loading, setLoading] = React.useState<boolean>(true);
  const [resource, setResource] = React.useState<Resource>();
  React.useEffect(() => {
    setLoading(true);
    if (value?.reference) {
      const resourceType = value.reference.split("/")[0];
      const id = value.reference.split("/")[1];
      client
        .batch({}, fhirVersion, {
          resourceType: "Bundle",
          type: "batch",
          entry: [
            { request: { method: "GET", url: `${resourceType}/${id}` } },
            {
              request: {
                method: "GET",
                url: `StructureDefinition/${resourceType}`,
              },
            },
          ],
        } as Bundle)
        .then((batchResponse) => {
          if (
            batchResponse.entry?.[0].resource?.resourceType === resourceType ||
            batchResponse.entry?.[0].resource?.id === id
          ) {
            setResource(batchResponse.entry?.[0].resource as Resource);
          }
          setSd(batchResponse.entry?.[1].resource as StructureDefinition);
        })
        .then(() => setLoading(false))
        .catch((e) => {
          console.error(e);
        });
    }
  }, [value]);

  return loading ? (
    <div className="flex justify-center items-center">
      <Loading />
    </div>
  ) : resource && sd ? (
    <FHIRGenerativeForm
      fhirVersion={fhirVersion}
      client={client}
      value={resource}
      structureDefinition={sd}
    />
  ) : (
    <div className="flex justify-center items-center">
      <span className="font-semibold text-slate-600">Resource not found</span>
    </div>
  );
};

const SearchResult = ({
  resource,
  onChange,
}: {
  resource: Resource | r4b.Resource;
  onChange: FHIRReferenceEditableProps["onChange"];
}) => {
  return (
    <div
      className="hover:bg-orange-200 py-2 px-4 cursor-pointer border "
      onClick={(_e) =>
        onChange?.call(this, {
          reference: `${resource.resourceType}/${resource.id}`,
        })
      }
    >
      <span className="font-semibold text-slate-700">
        {resource.resourceType}/{resource.id}
      </span>
    </div>
  );
};

const ReferenceSearch = ({
  fhirVersion,
  client,
  onChange,
  resourceTypesAllowed,
}: ClientProps & {
  resourceTypesAllowed?: ResourceType<typeof fhirVersion>[];
  onChange: FHIRReferenceEditableProps["onChange"];
}) => {
  const [resourceType, setResourceType] = React.useState<
    ResourceType<typeof fhirVersion> | undefined
  >(resourceTypesAllowed?.[0]);

  return (
    <div>
      <div className="flex space-x-1 text-slate-700  ">
        <div className="w-36">
          <Select
            value={resourceType}
            onChange={(option) =>
              setResourceType(option.value as ResourceType<typeof fhirVersion>)
            }
            options={(resourceTypesAllowed ?? []).map((rt) => ({
              label: rt,
              value: rt,
            }))}
          />
        </div>
      </div>
      <div className="mt-4 h-96 space-y-2 overflow-auto">
        {resourceType && (
          <FHIRGenerativeSearchTable
            resourceType={resourceType}
            fhirVersion={fhirVersion}
            client={client}
            onRowClick={(row): void => {
              onChange?.call(this, {
                reference: `${(row as Resource).resourceType}/${
                  (row as Resource).id
                }`,
              });
            }}
          />
        )}
      </div>
    </div>
  );
};

export const FHIRReferenceEditable = ({
  value,
  disabled,
  onChange,
  issue,
  client,
  label,
  fhirVersion,
  resourceTypesAllowed,
}: FHIRReferenceEditableProps) => {
  return (
    <Modal
      size="large"
      ModalContent={(setOpen) => (
        <ReferenceSearch
          fhirVersion={fhirVersion}
          onChange={(ref) => {
            onChange?.call(this, ref);
            setOpen(false);
          }}
          client={client}
          resourceTypesAllowed={resourceTypesAllowed}
        />
      )}
    >
      {(openSearch) => (
        <Modal
          size="large"
          ModalContent={(_setOpen) => (
            <ReferenceView
              value={value}
              client={client}
              fhirVersion={fhirVersion}
            />
          )}
        >
          {(openDisplay) => (
            <InputContainer
              disabled={disabled}
              label={label}
              issues={issue ? [issue] : []}
            >
              <div className="flex space-x-1">
                <div onClick={() => openDisplay(true)}>
                  <a className="text-orange-400 hover:text-orange-500 cursor-pointer">
                    {value?.reference}
                  </a>
                </div>
                {!disabled && (
                  <span onClick={() => openSearch(true)}>
                    <a className="text-slate-700 hover:text-orange-700 cursor-pointer font-semibold">
                      {<MagnifyingGlassCircleIcon className="w-5 h-5" />}
                    </a>
                  </span>
                )}
              </div>
            </InputContainer>
          )}
        </Modal>
      )}
    </Modal>
  );
};
