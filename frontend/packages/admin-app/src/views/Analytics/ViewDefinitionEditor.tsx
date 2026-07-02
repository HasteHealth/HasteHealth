import { ViewDefinitionSqlRunner } from "@haste-health/components";
import { getClient } from "../../db/client";
import { useAtomValue } from "jotai";
import { useState } from "react";
import { basicSetup } from "codemirror";

import { R4 } from "@haste-health/fhir-types/versions";
import { ViewDefinition, instant } from "@haste-health/fhir-types/r4/types";
import { json } from "@codemirror/lang-json";

const DEFAULT_VIEW_DEFINITION: ViewDefinition = {
  resourceType: "ViewDefinition",
  status: "draft",
  resource: "Patient",
  select: [
    {
      column: [
        {
          name: "id",
          path: "id",
          type: "http://hl7.org/fhirpath/System.String",
        },
        {
          name: "date of birth",
          path: "$this.birthDate",
          type: "date",
        },
      ],
    },
    {
      forEach: "$this.name",
      column: [
        {
          name: "name",
          path: "$this.given",
          type: "string",
          collection: true,
        },
        {
          name: "family",
          path: "$this.family",
          type: "string",
        },
      ],
    },
  ],
} as ViewDefinition;

const EDITOR_EXTENSIONS = [basicSetup, json()];

export default function ViewDefinitionEditor() {
  const client = useAtomValue(getClient);
  const [viewDefinition, setViewDefinition] = useState<ViewDefinition>(
    DEFAULT_VIEW_DEFINITION,
  );
  return (
    <div className="flex flex-1 flex-col">
      <ViewDefinitionSqlRunner
        client={client}
        viewDefinition={viewDefinition}
        setViewDefinition={setViewDefinition}
        editorExtensions={EDITOR_EXTENSIONS}
        defaultPageSize={10}
        fhirVersion={R4}
        since={"1980-01-01T00:00:00Z" as instant}
      />
    </div>
  );
}
