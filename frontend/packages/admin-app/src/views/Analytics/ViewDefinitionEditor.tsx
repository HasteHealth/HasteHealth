import { ViewDefinitionSqlRunner } from "@haste-health/components";
import { getClient } from "../../db/client";
import { useAtomValue } from "jotai";
import { R4 } from "@haste-health/fhir-types/versions";
import { instant } from "@haste-health/fhir-types/r4/types";
import { json } from "@codemirror/lang-json";
import { basicSetup } from "codemirror";

const EDITOR_EXTENSIONS = [basicSetup, json()];

export default function ViewDefinitionEditor() {
  const client = useAtomValue(getClient);
  return (
    <div className="flex overflow-auto">
      <ViewDefinitionSqlRunner
        client={client}
        editorExtensions={EDITOR_EXTENSIONS}
        defaultPageSize={10}
        fhirVersion={R4}
        since={"1980-01-01T00:00:00Z" as instant}
      />
    </div>
  );
}
