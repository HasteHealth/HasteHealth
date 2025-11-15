import type { Meta, StoryObj } from "@storybook/react";

import { Quantity } from "@haste-health/fhir-types/r4/types";

import { FHIRQuantityReadOnly } from "./QuantityReadOnly";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRQuantityReadOnly",
  component: FHIRQuantityReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRQuantityReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      value: 15,
      unit: "mg",
    } as Quantity,
  },
};

export const BadValue: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
