import type { Meta, StoryObj } from "@storybook/react";

import { Quantity } from "@oxidized-health/fhir-types/r4/types";

import { FHIRSimpleQuantityEditable } from "./SimpleQuantity";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRSimpleQuantityEditable",
  component: FHIRSimpleQuantityEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRSimpleQuantityEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      unit: "km",
      value: 55.2,
    } as Quantity,
    label: "Simple Quantity",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    // @ts-ignore
    value: "test",
    label: "Bad value",
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
