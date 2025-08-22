import type { Meta, StoryObj } from "@storybook/react";

import { FHIROIDEditable } from "./oid";

const meta = {
  title: "Primitives/FHIROIDEditable",
  component: FHIROIDEditable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
} satisfies Meta<typeof FHIROIDEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: "urn:oid:1.2.3.4.5",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: "urn:oid:1.2.3.4.5",
    issue: "Issue",
    onChange: (value) => console.log(value),
  },
};

export const InvalidValue: Story = {
  args: {
    // @ts-ignore
    value: "bad-value",
    onChange: (value) => console.log(value),
  },
};
