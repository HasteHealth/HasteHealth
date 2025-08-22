import type { Meta, StoryObj } from "@storybook/react";

import { Select } from "./select";

const meta = {
  title: "Base/Select",
  component: Select,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
} satisfies Meta<typeof Select>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ClosedSelect: Story = {
  args: {
    onChange: (value) => console.log(value),
    value: "option1",
    options: [
      {
        label: "Option 1",
        value: "option1",
      },
      {
        label: "Option 2",
        value: "option2",
      },
      {
        label: "Option 3",
        value: "option3",
      },
    ],
  },
};

export const OpenSelect: Story = {
  args: {
    onChange: (value) => console.log(value),
    value: "option1",
    open: true,
    options: [
      {
        label: "Option 1",
        value: "option1",
      },
      {
        label: "Option 2",
        value: "option2",
      },
      {
        label: "Option 3",
        value: "option3",
      },
    ],
  },
};

export const OnError: Story = {
  args: {
    onChange: (value) => console.log(value),
    value: "option1",
    issue: "Bad value",
    open: true,
    options: [
      {
        label: "Option 1",
        value: "option1",
      },
      {
        label: "Option 2",
        value: "option2",
      },
      {
        label: "Option 3",
        value: "option3",
      },
    ],
  },
};
