import type { Meta, StoryObj } from "@storybook/react";

import { FHIRBooleanEditable } from "./boolean";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRBooleanEditable",
  component: FHIRBooleanEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRBooleanEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const ExistingData: Story = {
  args: {
    value: true,
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
