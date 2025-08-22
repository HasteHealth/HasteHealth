import type { Meta, StoryObj } from "@storybook/react";

import { FHIRPositiveIntegerEditable } from "./positiveinteger";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRPositiveIntegerEditable",
  component: FHIRPositiveIntegerEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  argTypes: {
    value: { control: "number" },
  },
} satisfies Meta<typeof FHIRPositiveIntegerEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: 22.2,
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: 55,
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
