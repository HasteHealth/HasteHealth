import type { Meta, StoryObj } from "@storybook/react";

import { instant } from "@haste-health/fhir-types/r4/types";

import { FHIRInstantEditable } from "./instant";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRInstantEditable",
  component: FHIRInstantEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  argTypes: {
    value: { control: "text" },
  },
} satisfies Meta<typeof FHIRInstantEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: "2015-02-07T13:28:17.239+02:00" as instant,
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: "invalid-instant" as instant,
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
