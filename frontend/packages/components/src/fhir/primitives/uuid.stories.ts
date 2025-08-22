import type { Meta, StoryObj } from "@storybook/react";

import { FHIRUUIDEditable } from "./uuid";

const meta = {
  title: "Primitives/FHIRUUIDEditable",
  component: FHIRUUIDEditable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRUUIDEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: "d38ad80a-61b9-4c9b-a8e9-8c8b22659865",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: "d38ad80a-61b9-4c9b-a8e9-8c8b22659865",
    issue: "Issue",
    onChange: (value) => console.log(value),
  },
};

export const InvalidValue: Story = {
  args: {
    value: "1234-1234-1234-123",
    onChange: (value) => console.log(value),
  },
};
