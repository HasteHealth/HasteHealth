import type { Meta, StoryObj } from "@storybook/react";

import { CodeableConcept } from "@haste-health/fhir-types/r4/types";

import { FHIRCodeableConceptReadOnly } from "./CodeableConceptReadOnly";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRCodeableConceptReadOnly",
  component: FHIRCodeableConceptReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRCodeableConceptReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      coding: [
        { system: "test.com", code: "test" },
        { code: "testing2", display: "test2" },
      ],
    } as CodeableConcept,
  },
};

export const BadValue: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
