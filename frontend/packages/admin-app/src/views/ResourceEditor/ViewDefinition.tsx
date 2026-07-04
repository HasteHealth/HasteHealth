import {
  Button,
  DropDownMenu,
  ViewDefinitionSqlRunner,
} from "@haste-health/components";
import { ChevronDownIcon } from "@heroicons/react/24/outline";
import { getClient } from "../../db/client";
import { useAtomValue } from "jotai";
import { basicSetup } from "codemirror";
import { useEffect } from "react";
import { R4 } from "@haste-health/fhir-types/versions";
import { ViewDefinition, instant } from "@haste-health/fhir-types/r4/types";
import { json } from "@codemirror/lang-json";
import { AdditionalContent } from "../../components/ResourceEditor";

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

interface ViewDefinitionEditorProps extends AdditionalContent {
  resource: ViewDefinition | undefined;
  onChange: NonNullable<AdditionalContent["onChange"]>;
}

export default function ViewDefinitionEditor(
  props: Readonly<ViewDefinitionEditorProps>,
) {
  useEffect(() => {
    if (!props.resource) {
      props.onChange(DEFAULT_VIEW_DEFINITION);
    }
  }, [props.resource]);
  const client = useAtomValue(getClient);

  if (!props.resource) {
    return <div />;
  }

  return (
    <div className="flex flex-1 flex-col">
      <div className="mb-3 flex justify-end">
        <DropDownMenu links={props.actions}>
          <Button buttonType="secondary" buttonSize="small" onClick={() => {}}>
            <span className="flex items-center">
              <span>Actions</span> <ChevronDownIcon className="ml-1 w-3 h-3" />
            </span>
          </Button>
        </DropDownMenu>
      </div>
      <ViewDefinitionSqlRunner
        client={client}
        viewDefinition={props.resource}
        setViewDefinition={props.onChange}
        editorExtensions={EDITOR_EXTENSIONS}
        defaultPageSize={10}
        fhirVersion={R4}
        since={"1980-01-01T00:00:00Z" as instant}
      />
    </div>
  );
}
