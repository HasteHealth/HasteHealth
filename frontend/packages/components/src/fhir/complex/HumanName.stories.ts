import type { Meta, StoryObj } from "@storybook/react";

import { FHIRHumanNameEditable } from "./HumanName";

const meta = {
  title: "Complex/FHIRHumanNameEditable",
  component: FHIRHumanNameEditable,
  parameters: {
    layout: "centered",
  },

  tags: ["autodocs"],
} satisfies Meta<typeof FHIRHumanNameEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      given: ["Bob", "David"],
      family: "Smith",
    },
    label: "Name",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
