import type { Meta, StoryObj } from "@storybook/react";

import { FHIRTimeEditable } from "./time";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRTimeEditable",
  component: FHIRTimeEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRTimeEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: "12:00:00",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: "invalid-time",
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
