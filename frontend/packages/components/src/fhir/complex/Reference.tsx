import { MagnifyingGlassIcon } from "@heroicons/react/24/outline";
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
    /** Hide the current reference when another part of the row shows it. */
    showValue?: boolean;
    /** Label for the search button; defaults to "Search". */
    searchLabel?: string;
    /** Accessible name for the search button. */
    searchAriaLabel?: string;
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
      <span className="font-mono text-xs text-[var(--ci-muted)]">Resource not found</span>
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
    <button
      type="button"
      className="hover:bg-[var(--ci-surface-raised)] py-2 px-4 cursor-pointer border border-[var(--ci-line)] bg-[var(--ci-surface)] w-full text-left font-mono text-xs transition-colors"
      onClick={(_e) =>
        onChange?.call(this, {
          reference: `${resource.resourceType}/${resource.id}`,
        })
      }
    >
      <span className="font-semibold text-[var(--ci-ink)]">
        {resource.resourceType}/{resource.id}
      </span>
    </button>
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
      <div className="flex space-x-1 text-[var(--ci-ink)] font-mono text-xs">
        <div className="w-64">
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
  showValue = true,
  searchLabel = "Search",
  searchAriaLabel = "Search for a resource to reference",
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
              hideBorder
              disabled={disabled}
              label={label}
              issues={issue ? [issue] : []}
            >
              <div className="flex flex-wrap items-center gap-2">
                {showValue && value?.reference && (
                  <button
                    type="button"
                    className="text-[var(--ci-signal)] hover:underline cursor-pointer font-mono text-xs"
                    onClick={() => openDisplay(true)}
                  >
                    {value.reference}
                  </button>
                )}
                {!disabled && (
                  <button
                    type="button"
                    aria-label={searchAriaLabel}
                    className="inline-flex cursor-pointer items-center gap-1 font-mono text-[10px] uppercase tracking-wider text-[var(--ci-signal)] transition-colors hover:underline"
                    onClick={() => openSearch(true)}
                  >
                    <MagnifyingGlassIcon className="h-3.5 w-3.5" />
                    {searchLabel}
                  </button>
                )}
              </div>
            </InputContainer>
          )}
        </Modal>
      )}
    </Modal>
  );
};
