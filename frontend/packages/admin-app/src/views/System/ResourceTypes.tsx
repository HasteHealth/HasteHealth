import { FHIRGenerativeSearchTable, Button } from "@oxidized-health/components";
import {
  AllResourceTypes,
  R4,
  Resource,
  ResourceType,
} from "@oxidized-health/fhir-types/versions";
import { useAtomValue } from "jotai";
import { useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";
import { getClient } from "../../db/client";
import { PlusIcon } from "@heroicons/react/24/outline";

export default function ResourceTypes() {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const [refresh, setRefresh] = useState<(() => void) | undefined>(undefined);
  const params = useParams();

  return (
    <>
      <div className="mt-4 flex items-center ">
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
            })
          );
        }}
        client={client}
        fhirVersion={R4}
        resourceType={params.resourceType as ResourceType<R4>}
      />
    </>
  );
}
