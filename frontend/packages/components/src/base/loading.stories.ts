import type { Meta, StoryObj } from "@storybook/react";

import { Loading } from "./loading";

const meta = {
  title: "Base/Loading",
  component: Loading,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof Loading>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {},
};
