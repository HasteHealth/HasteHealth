import {
  ChevronRightIcon,
  HomeIcon,
  PlusIcon,
} from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";

import {
  Button,
  FHIRCodeEditable,
  FHIRGenerativeSearchTable,
  FHIRReferenceEditable,
  FHIRStringEditable,
  Modal,
  Toaster,
} from "@haste-health/components";
import {
  Reference,
  StructureDefinition,
  code,
  id,
  uri,
} from "@haste-health/fhir-types/r4/types";
import {
  AllResourceTypes,
  R4,
  Resource,
  ResourceType,
} from "@haste-health/fhir-types/versions";
import { HasteHealthInviteUser } from "@haste-health/generated-ops/r4";

import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

function InviteModal({
  refresh,
  setOpen,
}: Readonly<{ setOpen: (open: boolean) => void; refresh: () => void }>) {
  const client = useAtomValue(getClient);
  const [email, setEmail] = useState<string | undefined>();
  const [role, setRole] = useState<code | undefined>();
  const [accessPolicyRef, setAccessPolicyRef] = useState<
    Reference | undefined
  >();

  return (
    <div className="space-y-4 mt-4">
      <FHIRStringEditable
        label="Email"
        value={email}
        onChange={(email) => {
          setEmail(email);
        }}
      />
      <FHIRCodeEditable
        label="Role"
        system={
          "https://haste.health/fhir/ValueSet/MembershipRole|4.0.1" as uri
        }
        fhirVersion={R4}
        client={client}
        value={role}
        onChange={setRole}
      />
      <FHIRReferenceEditable
        label="AccessPolicyV2"
        resourceTypesAllowed={["AccessPolicyV2"]}
        fhirVersion={R4}
        client={client}
        value={accessPolicyRef}
        onChange={setAccessPolicyRef}
      />

      <div className="flex items-center">
        <div>
          <Button
            buttonType="secondary"
            onClick={() => {
              setOpen(false);
            }}
          >
            Cancel
          </Button>
        </div>
        <div className="flex grow justify-end">
          <Button
            onClick={() => {
              if (!email) {
                Toaster.error("Email is required");
                return;
              }
              if (!role) {
                Toaster.error("Role is required");
                return;
              }

              Toaster.promise(
                client
                  .invoke_type(HasteHealthInviteUser.Op, {}, R4, "Membership", {
                    role,
                    email,
                    accessPolicy: accessPolicyRef,
                  })
                  .then(() => {
                    refresh();
                    setOpen(false);
                  }),
                {
                  loading: "Uploading Bundle",
                  success: () => `Bundle was uploaded`,
                  error: (error) => {
                    return getErrorMessage(error);
                  },
                },
              );
            }}
          >
            Send
          </Button>
        </div>
      </div>
    </div>
  );
}

function ResourceTypeHeader({
  sd,
  refresh,
}: Readonly<{ sd: StructureDefinition | undefined; refresh: () => void }>) {
  const params = useParams();
  const navigate = useNavigate();
  const resourceType = params.resourceType as string;

  return (
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
          className="rounded px-1 py-0.5 hover:bg-slate-100 hover:text-slate-700"
          onClick={() => navigate(generatePath("/resources", {}))}
          type="button"
        >
          Resources
        </button>
        <ChevronRightIcon className="h-4 w-4 text-slate-400" />
        <span className="rounded bg-slate-100 px-2 py-0.5 text-xs font-medium text-slate-700">
          {resourceType}
        </span>
      </div>

      <div className="flex items-start justify-between gap-4">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold text-slate-900">
            {resourceType}
          </h1>
          <p className="text-sm text-slate-500">
            {sd?.snapshot?.element?.[0]?.definition ??
              `Browse and manage ${resourceType} resources.`}
          </p>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <span className="rounded-md border border-slate-200 bg-slate-50 px-2 py-1 text-xs font-medium text-slate-700">
              Status: {sd?.status ?? "unknown"}
            </span>
            <span className="rounded-md border border-slate-200 bg-slate-50 px-2 py-1 text-xs font-medium text-slate-700">
              Version: {sd?.version ?? "n/a"}
            </span>
          </div>
        </div>
        <Button
          className="font-medium"
          buttonSize="small"
          buttonType="secondary"
          onClick={() =>
            navigate(
              generatePath("/resources/:resourceType/:id", {
                resourceType,
                id: "new",
              }),
            )
          }
        >
          <div className="flex items-center justify-center">
            <PlusIcon className="mr-1 h-4 w-4" />{" "}
            <span>New {resourceType}</span>
          </div>
        </Button>
      </div>
    </header>
  );
}

export default function ResourceTypeView() {
  const client = useAtomValue(getClient);
  const params = useParams();
  const [sd, setSd] = useState<StructureDefinition | undefined>(undefined);
  const navigate = useNavigate();
  const [refresh, setRefresh] = useState<(() => void) | undefined>(undefined);

  useEffect(() => {
    client
      .read({}, R4, "StructureDefinition", params.resourceType as id)
      .then((res) => {
        setSd(res);
      })
      .catch(() => {
        Toaster.error("Failed to load structure definition for resource type.");
      });
  }, [client, params.resourceType]);

  return (
    <div className="flex w-full flex-1 flex-col gap-6 text-slate-700 overflow-auto">
      {refresh && <ResourceTypeHeader sd={sd} refresh={refresh} />}

      <section className="flex-grow rounded-lg border border-slate-200 bg-white p-3 shadow-sm">
        <div className="mb-3 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
          Click any row to view or edit that resource.
        </div>
        <FHIRGenerativeSearchTable
          key={params.resourceType}
          refresh={(refreshFnc) => {
            if (!refresh) {
              setRefresh(() => refreshFnc);
            }
          }}
          onRowClick={(row) => {
            navigate(
              generatePath("/resources/:resourceType/:id", {
                resourceType: params.resourceType as string,
                id: (row as Resource<R4, AllResourceTypes>).id as string,
              }),
            );
          }}
          client={client}
          fhirVersion={R4}
          resourceType={params.resourceType as ResourceType<R4>}
        />
      </section>
    </div>
  );
}
