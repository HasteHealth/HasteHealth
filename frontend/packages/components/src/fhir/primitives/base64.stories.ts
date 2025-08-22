import type { Meta, StoryObj } from "@storybook/react";

import { FHIRBase64BinaryEditable } from "./base64";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRBase64BinaryEditable",
  component: FHIRBase64BinaryEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRBase64BinaryEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const ExistingData: Story = {
  args: {
    value: "test data",
    onChange: (value) => console.log(value),
  },
};

export const EmptyData: Story = {
  args: {
    onChange: (value) => console.log(value),
  },
};
