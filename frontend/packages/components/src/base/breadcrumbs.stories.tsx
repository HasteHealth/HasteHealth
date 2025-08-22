import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import { BreadCrumbs } from "./breadcrumbs";

const meta = {
  title: "Base/BreadCrumbs",
  component: BreadCrumbs,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
} satisfies Meta<typeof BreadCrumbs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    breadcrumbs: [
      <span>Home</span>,
      <span>Resources</span>,
      <span>Users</span>,
    ],
  },
};
