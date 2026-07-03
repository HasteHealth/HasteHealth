import type { Meta, StoryObj } from "@storybook/react-webpack5";
import React, { useEffect, useState } from "react";

import {
  Questionnaire,
  QuestionnaireResponse,
} from "@haste-health/fhir-types/r4/types";

import { FHIRQuestionnaireRenderer } from "./Questionnaire";

type StoryStateProps = {
  schema: Questionnaire;
  value?: QuestionnaireResponse;
};

const StatefulQuestionnaire = ({ schema, value }: StoryStateProps) => {
  const [current, setCurrent] = useState<QuestionnaireResponse | undefined>(
    value,
  );

  useEffect(() => {
    setCurrent(value);
  }, [value]);

  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-6 p-6">
      <FHIRQuestionnaireRenderer
        schema={schema}
        value={current}
        onChange={setCurrent}
      />
      <div className="rounded border border-slate-200 bg-slate-50 p-3">
        <div className="mb-2 text-xs font-semibold uppercase tracking-wide text-slate-600">
          QuestionnaireResponse Preview
        </div>
        <pre className="max-h-80 overflow-auto text-xs text-slate-800">
          {JSON.stringify(current, null, 2)}
        </pre>
      </div>
    </div>
  );
};

const meta = {
  title: "Resources/FHIRQuestionnaireRenderer",
  component: StatefulQuestionnaire,
  tags: ["autodocs"],
  parameters: {
    layout: "fullscreen",
  },
} satisfies Meta<typeof StatefulQuestionnaire>;

export default meta;
type Story = StoryObj<typeof meta>;

const sampleSchema = {
  resourceType: "Questionnaire",
  id: "intake-questionnaire",
  url: "http://example.org/fhir/Questionnaire/intake",
  status: "active",
  title: "Patient Intake",
  item: [
    {
      linkId: "section-1",
      text: "Basic Information",
      type: "group",
      item: [
        {
          linkId: "display-1",
          text: "Please answer the following questions.",
          type: "display",
        },
        {
          linkId: "name",
          text: "Full name",
          type: "string",
          required: true,
        },
        {
          linkId: "dob",
          text: "Date of birth",
          type: "date",
        },
        {
          linkId: "consent",
          text: "Consent to treatment",
          type: "boolean",
          required: true,
        },
        {
          linkId: "symptoms",
          text: "Current symptoms",
          type: "text",
        },
      ],
    },
    {
      linkId: "pain-level",
      text: "Pain level",
      type: "integer",
    },
    {
      linkId: "preferred-contact",
      text: "Preferred contact method",
      type: "choice",
      answerOption: [
        { valueString: "Phone" },
        { valueString: "Email" },
        { valueString: "SMS" },
      ],
      extension: [
        {
          url: "http://hl7.org/fhir/StructureDefinition/questionnaire-itemControl",
          valueCodeableConcept: {
            coding: [{ code: "radio-button" }],
          },
        },
      ],
    },
    {
      linkId: "allergies",
      text: "Known allergies",
      type: "open-choice",
      repeats: true,
      answerOption: [
        { valueString: "Penicillin" },
        { valueString: "Latex" },
        { valueString: "Peanuts" },
      ],
    },
    {
      linkId: "follow-up-url",
      text: "Portal profile URL",
      type: "url",
    },
  ],
} as Questionnaire;

const sampleResponse = {
  resourceType: "QuestionnaireResponse",
  status: "in-progress",
  questionnaire: "http://example.org/fhir/Questionnaire/intake",
  item: [
    {
      linkId: "section-1",
      item: [
        { linkId: "display-1" },
        { linkId: "name", answer: [{ valueString: "Jane Doe" }] },
        { linkId: "dob", answer: [{ valueDate: "1990-02-14" }] },
        { linkId: "consent", answer: [{ valueBoolean: true }] },
        { linkId: "symptoms", answer: [{ valueString: "Mild headache" }] },
      ],
    },
    {
      linkId: "pain-level",
      answer: [{ valueInteger: 3 }],
    },
    {
      linkId: "preferred-contact",
      answer: [{ valueString: "Email" }],
    },
    {
      linkId: "allergies",
      answer: [{ valueString: "Latex" }],
    },
    {
      linkId: "follow-up-url",
      answer: [{ valueUri: "https://patient.example.org/profile/jane" }],
    },
  ],
} as QuestionnaireResponse;

export const Primary: Story = {
  args: {
    schema: sampleSchema,
    value: sampleResponse,
  },
};

export const EmptyStart: Story = {
  args: {
    schema: sampleSchema,
  },
};
