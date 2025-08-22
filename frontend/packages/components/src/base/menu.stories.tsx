import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import { Button } from "./button";
import { DropDownMenu } from "./menu";

const meta = {
  title: "Base/Menu",
  component: DropDownMenu,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof DropDownMenu>;

export default meta;
type Story = StoryObj<typeof meta>;

export const DropdownMenuStory: Story = {
  args: {
    children: <Button>Menu</Button>,
    links: [
      { onClick: () => alert("settings"), label: "Settings" },
      { onClick: () => alert("logout"), label: "Sign out" },
    ],
  },
};
