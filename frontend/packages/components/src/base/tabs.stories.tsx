import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import { Button } from "./button";
import { Tabs } from "./tabs";

const meta = {
  title: "Base/Tabs",
  component: Tabs,
  tags: ["autodocs"],
  parameters: {
    ayout: "fullscreen",
  },
} satisfies Meta<typeof Tabs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    tabs: [
      { id: 5, title: "Tab1", content: "Tab 1 content" },
      {
        id: 6,
        title: "Tab 2",
        content: <span className="font-bold">Tab 2 content</span>,
      },
      {
        id: 7,
        title: "Tab 3",
        content: <span className="font-medium underline">Tab 3 content</span>,
      },
    ],
  },
};

export const RightSide: Story = {
  args: {
    tabs: [
      { id: 5, title: "Tab1", content: "Tab 1 content" },
      {
        id: 6,
        title: "Tab 2",
        content: <span className="font-bold">Tab 2 content</span>,
      },
      {
        id: 7,
        title: "Tab 3",
        content: <span className="font-medium underline">Tab 3 content</span>,
      },
    ],
    rightSide: <Button buttonSize="small">Actions</Button>,
  },
};
