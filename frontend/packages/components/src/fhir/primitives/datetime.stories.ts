import type { Meta, StoryObj } from "@storybook/react";

import { dateTime } from "@haste-health/fhir-types/r4/types";

import { FHIRDateTimeEditable } from "./datetime";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRDateTimeEditable",
  component: FHIRDateTimeEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  argTypes: {
    value: { control: "text" },
  },
} satisfies Meta<typeof FHIRDateTimeEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: "2017-01-01" as dateTime,
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: "invalid-date" as dateTime,
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
