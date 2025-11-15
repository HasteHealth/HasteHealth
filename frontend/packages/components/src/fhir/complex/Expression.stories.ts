import type { Meta, StoryObj } from "@storybook/react";

import { Expression } from "@haste-health/fhir-types/r4/types";

import { FHIRExpressionEditable } from "./Expression";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRExpressionEditable",
  component: FHIRExpressionEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRExpressionEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      expression: "$this.test",
      language: "text/fhirpath",
    } as Expression,
    label: "FP Expression",
    onChange: (value) => console.log(value),
  },
};
