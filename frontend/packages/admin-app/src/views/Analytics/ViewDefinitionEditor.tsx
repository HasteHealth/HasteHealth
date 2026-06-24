import { ViewDefinitionSqlRunner } from "@haste-health/components";
import { getClient } from "../../db/client";
import { useAtomValue } from "jotai";
import { R4 } from "@haste-health/fhir-types/versions";

export default function ViewDefinitionEditor() {
  const client = useAtomValue(getClient);
  return (
    <ViewDefinitionSqlRunner
      client={client}
      defaultPageSize={10}
      fhirVersion={R4}
      resources={[
        {
          resourceType: "Patient",
          name: [{ given: ["John"], family: "Doe" }],
        },
      ]}
    />
  );
}
