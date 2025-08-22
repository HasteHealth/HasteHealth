import { PlusIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import { useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";

import {
  Button,
  FHIRCodeEditable,
  FHIRGenerativeSearchTable,
  FHIRReferenceEditable,
  FHIRStringEditable,
  Modal,
  Toaster,
} from "@oxidized-health/components";
import { Reference, code, uri } from "@oxidized-health/fhir-types/r4/types";
import {
  AllResourceTypes,
  R4,
  Resource,
  ResourceType,
} from "@oxidized-health/fhir-types/versions";
import { OxidizedHealthInviteUser } from "@oxidized-health/generated-ops/r4";

import { getClient } from "../db/client";
import { getErrorMessage } from "../utilities";

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
          "https://oxidized-health.app/fhir/ValueSet/MembershipRole|4.0.1" as uri
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
                  .invoke_type(
                    OxidizedHealthInviteUser.Op,
                    {},
                    R4,
                    "Membership",
                    {
                      role,
                      email,
                      accessPolicy: accessPolicyRef,
                    }
                  )
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
                }
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

function ResourceTypeHeader({ refresh }: Readonly<{ refresh: () => void }>) {
  const params = useParams();
  const navigate = useNavigate();

  return (
    <div className="flex mb-4 justify-center items-center">
      <h2 className="text-left flex text-xl font-semibold mr-4 ">
        {params.resourceType}
      </h2>
      <div className="flex flex-1 justify-end">
        <div className="pr-4">
          {params.resourceType === "Membership" ? (
            <Modal
              modalTitle="Invite User"
              ModalContent={(setOpen) => (
                <InviteModal refresh={refresh} setOpen={setOpen} />
              )}
            >
              {(setopen) => (
                <Button
                  className="ml-2 font-medium"
                  buttonSize="small"
                  buttonType="secondary"
                  onClick={() => {
                    setopen(true);
                  }}
                >
                  Send Invite
                </Button>
              )}
            </Modal>
          ) : (
            <Button
              className="ml-2 font-medium"
              buttonSize="small"
              buttonType="secondary"
              onClick={() =>
                navigate(
                  generatePath("/resources/:resourceType/:id", {
                    resourceType: params.resourceType as string,
                    id: "new",
                  })
                )
              }
            >
              <div className="flex items-center justify-center ">
                <PlusIcon className="w-4 h-4 mr-1" /> <span>New</span>
              </div>
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

export default function ResourceTypeView() {
  const client = useAtomValue(getClient);
  const params = useParams();
  const navigate = useNavigate();
  const [refresh, setRefresh] = useState<(() => void) | undefined>(undefined);

  return (
    <div className="flex flex-col flex-1 w-full text-slate-700">
      {refresh && <ResourceTypeHeader refresh={refresh} />}
      <div className="overflow-auto">
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
              })
            );
          }}
          client={client}
          fhirVersion={R4}
          resourceType={params.resourceType as ResourceType<R4>}
        />
      </div>
    </div>
  );
}
