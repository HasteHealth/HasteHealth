import { ViewDefinitionSqlRunner } from "@haste-health/components";
import { getClient } from "../../db/client";
import { useAtomValue } from "jotai";
import { R4 } from "@haste-health/fhir-types/versions";
import { date } from "@haste-health/fhir-types/r4/types";
import { json } from "@codemirror/lang-json";
import { basicSetup } from "codemirror";

const EDITOR_EXTENSIONS = [basicSetup, json()];

export default function ViewDefinitionEditor() {
  const client = useAtomValue(getClient);
  return (
    <ViewDefinitionSqlRunner
      client={client}
      editorExtensions={EDITOR_EXTENSIONS}
      defaultPageSize={10}
      fhirVersion={R4}
      resources={[
        {
          resourceType: "Patient",
          name: [{ given: ["John", "Bob"], family: "Doe" }],
          birthDate: "1990-01-01" as date,
        },
      ]}
    />
  );
}
