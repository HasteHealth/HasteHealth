import type { Meta, StoryObj } from "@storybook/react";

import { Button } from "./button";

const meta = {
  title: "Base/Button",
  component: Button,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    buttonType: "primary",
    children: "Button",
    buttonSize: "large",
  },
};

export const Secondary: Story = {
  args: {
    buttonType: "secondary",
    children: "Button",
    buttonSize: "large",
  },
};

export const Danger: Story = {
  args: {
    buttonType: "danger",
    children: "Button",
    buttonSize: "small",
  },
};
