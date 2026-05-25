import {
  Questionnaire,
  QuestionnaireResponse,
} from "@haste-health/fhir-types/lib/generated/r4/types";
import ResourceEditorComponent, {
  AdditionalContent,
} from "../../components/ResourceEditor";
import { useState } from "react";
import { FHIRQuestionnaireRenderer } from "@haste-health/components";

interface QuestionnaireViewProps extends AdditionalContent {
  resource: Questionnaire | undefined;
  onChange: NonNullable<AdditionalContent["onChange"]>;
}

export default function QuestionnaireView({
  id,
  resourceType,
  resource,
  actions,
  structureDefinition,
  onChange,
}: Readonly<QuestionnaireViewProps>) {
  const [questionnaireResponse, setQuestionnaireResponse] = useState<
    QuestionnaireResponse | undefined
  >(undefined);
  return (
    <ResourceEditorComponent
      id={id}
      actions={actions}
      structureDefinition={structureDefinition}
      resourceType={resourceType}
      resource={resource}
      onChange={onChange}
      rightTabs={[
        {
          id: "form",
          title: "Form",
          content: (
            <FHIRQuestionnaireRenderer
              schema={resource as Questionnaire}
              value={questionnaireResponse}
              onChange={setQuestionnaireResponse}
            />
          ),
        },
      ]}
    />
  );
}
