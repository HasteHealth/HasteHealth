import { FHIRGenerativeSearchTable } from "@oxidized-health/components";
import { R4 } from "@oxidized-health/fhir-types/versions";
import { useAtomValue } from "jotai";
import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { getClient } from "../../db/client";

export default function Users() {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const [refresh, setRefresh] = useState<(() => void) | undefined>(undefined);

  return (
    <FHIRGenerativeSearchTable
      refresh={(refreshFnc) => {
        if (!refresh) {
          setRefresh(() => refreshFnc);
        }
      }}
      onRowClick={(row) => {
        alert("click");
      }}
      client={client}
      fhirVersion={R4}
      resourceType={"User"}
    />
  );
}
