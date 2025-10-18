import { FHIRGenerativeSearchTable } from "@oxidized-health/components";
import {
  AllResourceTypes,
  R4,
  Resource,
} from "@oxidized-health/fhir-types/versions";
import { useAtomValue } from "jotai";
import { useState } from "react";
import { generatePath, useNavigate, useParams } from "react-router-dom";
import { getClient } from "../../db/client";

export default function IdentityProviders() {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const [refresh, setRefresh] = useState<(() => void) | undefined>(undefined);
  const params = useParams();

  return (
    <FHIRGenerativeSearchTable
      refresh={(refreshFnc) => {
        if (!refresh) {
          setRefresh(() => refreshFnc);
        }
      }}
      onRowClick={(row) => {
        navigate(
          generatePath("/edit/:resourceType/:id", {
            resourceType: (row as Resource<R4, AllResourceTypes>).resourceType,
            id: (row as Resource<R4, AllResourceTypes>).id as string,
          })
        );
      }}
      client={client}
      fhirVersion={R4}
      resourceType={"User"}
    />
  );
}
